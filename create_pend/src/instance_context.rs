use fltkernel::{
    _FLT_INSTANCE, DbgPrint, FLT_CALLBACK_DATA, FLT_CALLBACK_DATA_QUEUE, FLT_CONTEXT_TYPE,
    FltCbdqInsertIo, FltCbdqRemoveNextIo, LIST_ENTRY, NTSTATUS, PFLT_CALLBACK_DATA,
    PFLT_CALLBACK_DATA_QUEUE_ACQUIRE, PFLT_CALLBACK_DATA_QUEUE_COMPLETE_CANCELED_IO,
    PFLT_CALLBACK_DATA_QUEUE_INSERT_IO, PFLT_CALLBACK_DATA_QUEUE_PEEK_NEXT_IO,
    PFLT_CALLBACK_DATA_QUEUE_RELEASE, PFLT_CALLBACK_DATA_QUEUE_REMOVE_IO, PFLT_CONTEXT, c_str,
    create_list_head, kd_print,
};

use crate::STATUS_SUCCESS;

#[repr(C)]
pub struct InstanceContext {
    cbdq: FLT_CALLBACK_DATA_QUEUE,
    pended_cbds: LIST_ENTRY,
}

impl Default for InstanceContext {
    fn default() -> Self {
        Self {
            cbdq: FLT_CALLBACK_DATA_QUEUE::default(),
            pended_cbds: create_list_head(),
        }
    }
}

impl InstanceContext {
    pub fn init(
        &mut self,
        instance: &mut _FLT_INSTANCE,
        insert_io: PFLT_CALLBACK_DATA_QUEUE_INSERT_IO,
        remove_io: PFLT_CALLBACK_DATA_QUEUE_REMOVE_IO,
        peeknext_io: PFLT_CALLBACK_DATA_QUEUE_PEEK_NEXT_IO,
        acquire: PFLT_CALLBACK_DATA_QUEUE_ACQUIRE,
        release: PFLT_CALLBACK_DATA_QUEUE_RELEASE,
        complete_canceled_io: PFLT_CALLBACK_DATA_QUEUE_COMPLETE_CANCELED_IO,
    ) -> Result<NTSTATUS, NTSTATUS> {
        let cbdq = match fltkernel::try_create_cbd_queue(
            instance,
            insert_io,
            remove_io,
            peeknext_io,
            acquire,
            release,
            complete_canceled_io,
        ) {
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
