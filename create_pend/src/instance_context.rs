use fltkernel::{
    _FLT_INSTANCE, DbgPrint, FLT_CALLBACK_DATA_QUEUE, FLT_CONTEXT_TYPE, LIST_ENTRY, NTSTATUS,
    PFLT_CALLBACK_DATA_QUEUE_ACQUIRE, PFLT_CALLBACK_DATA_QUEUE_COMPLETE_CANCELED_IO,
    PFLT_CALLBACK_DATA_QUEUE_INSERT_IO, PFLT_CALLBACK_DATA_QUEUE_PEEK_NEXT_IO,
    PFLT_CALLBACK_DATA_QUEUE_RELEASE, PFLT_CALLBACK_DATA_QUEUE_REMOVE_IO, PFLT_CONTEXT, c_str,
    create_list_head, kd_print,
};

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
    ) -> NTSTATUS {
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
                return status;
            }
        };
        self.cbdq = cbdq;

        0
    }
}

pub unsafe extern "C" fn instance_context_cleanup(
    context: PFLT_CONTEXT,
    _context_type: FLT_CONTEXT_TYPE,
) {
    unsafe { core::ptr::drop_in_place(context as *mut InstanceContext) };
}
