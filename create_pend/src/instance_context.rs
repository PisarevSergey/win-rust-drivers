use fltkernel::{
    _FLT_INSTANCE, _KWAIT_REASON_Executive as Executive, _MODE_KernelMode as KernelMode,
    _WAIT_TYPE_WaitAny as WaitAny, DbgPrint, Event, FALSE, FLT_CALLBACK_DATA,
    FLT_CALLBACK_DATA_QUEUE, FLT_CONTEXT_TYPE, FLT_PREOP_SUCCESS_NO_CALLBACK, FltCbdqDisable,
    FltCbdqInsertIo, FltCbdqRemoveNextIo, KeWaitForMultipleObjects, LIST_ENTRY, NTSTATUS,
    NotificationEvent, PFLT_CALLBACK_DATA, PFLT_CONTEXT, PLARGE_INTEGER, PVOID,
    PsTerminateSystemThread, PushLock, RefObject, STATUS_SUCCESS, STATUS_WAIT_0, Semaphore,
    ZwWaitForSingleObject, c_str, create_thread, get_object_from_handle, initialize_list_head,
    kd_print,
};

mod cbd_queue;

#[repr(C)]
#[derive(Default)]
pub struct InstanceContext {
    cbdq: FLT_CALLBACK_DATA_QUEUE,
    cbds_lock: PushLock,
    pended_cbds: LIST_ENTRY,
    cbd_ready: Semaphore,
    exit_thread: Event,
    worker_thread: core::option::Option<RefObject>,
}

impl InstanceContext {
    pub fn init(&mut self, instance: &mut _FLT_INSTANCE) -> Result<NTSTATUS, NTSTATUS> {
        let cbdq = match cbd_queue::try_create_cbd_queue(instance) {
            Ok(cbdq) => cbdq,
            Err(status) => {
                kd_print!(
                    c_str!("try_create_cbd_queue failed with status 0x%x\n"),
                    status
                );
                return Err(status);
            }
        };
        self.cbdq = cbdq;
        self.cbds_lock = PushLock::new();
        initialize_list_head(&mut self.pended_cbds);
        self.cbd_ready.init(0, i32::MAX);
        self.exit_thread.init(NotificationEvent, false);

        let thread = match create_thread(Some(cbd_completer_worker), &raw mut *self as PVOID) {
            Ok(thread) => thread,
            Err(status) => return Err(status),
        };
        kd_print!(c_str!("thread started\n"));

        let worker_thread = match get_object_from_handle(
            &thread,
            fltkernel::SYNCHRONIZE,
            unsafe { Some(fltkernel::PsThreadType) },
            fltkernel::KernelMode as i8,
        ) {
            Ok(thread) => thread,
            Err(status) => {
                kd_print!(
                    c_str!("get_object_from_handle failed with status 0x%x\n"),
                    status
                );

                self.stop();

                unsafe {
                    ZwWaitForSingleObject(
                        thread.get_native_handle(),
                        FALSE as u8,
                        PLARGE_INTEGER::default(),
                    )
                };

                return Err(status);
            }
        };
        self.worker_thread = Some(worker_thread);

        Ok(0)
    }

    pub fn stop(&mut self) {
        self.disable_queue();

        self.exit_thread.set();

        if let Some(ref mut thread) = self.worker_thread {
            thread.wait();
        }
    }

    pub fn pend_cbd(&mut self, cbd: PFLT_CALLBACK_DATA) -> Result<NTSTATUS, NTSTATUS> {
        let status = unsafe {
            FltCbdqInsertIo(
                &raw mut self.cbdq,
                cbd,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            )
        };
        match status {
            STATUS_SUCCESS => {
                return Ok(STATUS_SUCCESS);
            }
            _ => {
                kd_print!(
                    c_str!("FltCbdqInsertIo failed for CBD 0x%p with status 0x%x\n"),
                    cbd,
                    status
                );
                return Err(status);
            }
        }
    }

    pub fn extract_pended_cbd(&mut self) -> Result<PFLT_CALLBACK_DATA, PFLT_CALLBACK_DATA> {
        let cbd = unsafe { FltCbdqRemoveNextIo(&mut self.cbdq, core::ptr::null_mut()) };
        if cbd.is_null() {
            return Err(core::ptr::null_mut::<FLT_CALLBACK_DATA>());
        }

        Ok(cbd)
    }

    fn disable_queue(&mut self) {
        unsafe { FltCbdqDisable(&raw mut self.cbdq) };
    }
}

pub unsafe extern "C" fn instance_context_cleanup(
    context: PFLT_CONTEXT,
    _context_type: FLT_CONTEXT_TYPE,
) {
    unsafe { core::ptr::drop_in_place(context as *mut InstanceContext) };
}

unsafe extern "C" fn cbd_completer_worker(start_context: PVOID) {
    kd_print!(c_str!("enter thread\n"));
    let ic = unsafe { &mut *(start_context as *mut InstanceContext) };

    loop {
        let mut wait_objects = [unsafe { ic.cbd_ready.get_native_semaphore() }, unsafe {
            ic.exit_thread.get_native_event()
        }];
        let status = unsafe {
            KeWaitForMultipleObjects(
                2,
                wait_objects.as_mut_ptr(),
                WaitAny,
                Executive,
                KernelMode as i8,
                FALSE as u8,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            )
        };

        while let Ok(cbd) = ic.extract_pended_cbd() {
            unsafe {
                fltkernel::FltCompletePendedPreOperation(
                    cbd,
                    FLT_PREOP_SUCCESS_NO_CALLBACK,
                    core::ptr::null_mut(),
                )
            };
        }

        if status != STATUS_WAIT_0 {
            break;
        }
    }

    kd_print!(c_str!("exiting thread\n"));
    unsafe { PsTerminateSystemThread(STATUS_SUCCESS) };
}
