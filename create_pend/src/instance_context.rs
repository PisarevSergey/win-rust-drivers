use fltkernel::{
    _FLT_INSTANCE, DbgPrint, FLT_CALLBACK_DATA, FLT_CALLBACK_DATA_QUEUE, FLT_CONTEXT_TYPE,
    FltCbdqInsertIo, FltCbdqRemoveNextIo, LIST_ENTRY, NTSTATUS, PFLT_CALLBACK_DATA, PFLT_CONTEXT,
    c_str, initialize_list_head, kd_print,
};

use crate::{STATUS_SUCCESS, cbd_queue};

#[repr(C)]
#[derive(Default)]
pub struct InstanceContext {
    cbdq: FLT_CALLBACK_DATA_QUEUE,
    pended_cbds: LIST_ENTRY,
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
        initialize_list_head(&mut self.pended_cbds);

        Ok(0)
    }

    pub fn push_cbd(&mut self, cbd: PFLT_CALLBACK_DATA) -> Result<NTSTATUS, NTSTATUS> {
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

    pub fn pop_cbd(&mut self) -> Result<PFLT_CALLBACK_DATA, PFLT_CALLBACK_DATA> {
        let cbd = unsafe { FltCbdqRemoveNextIo(&mut self.cbdq, core::ptr::null_mut()) };
        if cbd.is_null() {
            return Err(core::ptr::null_mut::<FLT_CALLBACK_DATA>());
        }

        Ok(cbd)
    }
}

pub unsafe extern "C" fn instance_context_cleanup(
    context: PFLT_CONTEXT,
    _context_type: FLT_CONTEXT_TYPE,
) {
    unsafe { core::ptr::drop_in_place(context as *mut InstanceContext) };
}
