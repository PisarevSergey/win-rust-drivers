use crate::{
    _FLT_INSTANCE, FLT_CALLBACK_DATA_QUEUE, NTSTATUS, PFLT_CALLBACK_DATA_QUEUE_ACQUIRE,
    PFLT_CALLBACK_DATA_QUEUE_COMPLETE_CANCELED_IO, PFLT_CALLBACK_DATA_QUEUE_INSERT_IO,
    PFLT_CALLBACK_DATA_QUEUE_PEEK_NEXT_IO, PFLT_CALLBACK_DATA_QUEUE_RELEASE,
    PFLT_CALLBACK_DATA_QUEUE_REMOVE_IO,
};

pub fn try_create_cbd_queue(
    instance: &mut _FLT_INSTANCE,
    insert_io: PFLT_CALLBACK_DATA_QUEUE_INSERT_IO,
    remove_io: PFLT_CALLBACK_DATA_QUEUE_REMOVE_IO,
    peeknext_io: PFLT_CALLBACK_DATA_QUEUE_PEEK_NEXT_IO,
    acquire: PFLT_CALLBACK_DATA_QUEUE_ACQUIRE,
    release: PFLT_CALLBACK_DATA_QUEUE_RELEASE,
    complete_canceled_io: PFLT_CALLBACK_DATA_QUEUE_COMPLETE_CANCELED_IO,
) -> core::result::Result<FLT_CALLBACK_DATA_QUEUE, NTSTATUS> {
    let mut cbdq = FLT_CALLBACK_DATA_QUEUE::default();

    let status = unsafe {
        crate::FltCbdqInitialize(
            &raw mut *instance,
            &raw mut cbdq,
            insert_io,
            remove_io,
            peeknext_io,
            acquire,
            release,
            complete_canceled_io,
        )
    };
    if status != 0 {
        return Err(status);
    }

    Ok(cbdq)
}
