use fltkernel::{KIRQL, NTSTATUS, PFLT_CALLBACK_DATA, PFLT_CALLBACK_DATA_QUEUE, PKIRQL, PVOID};

pub unsafe extern "C" fn insert_io(
    cbdq: PFLT_CALLBACK_DATA_QUEUE,
    cbd: PFLT_CALLBACK_DATA,
    insert_context: PVOID,
) -> NTSTATUS {
    unimplemented!("to do");
    0
}

pub unsafe extern "C" fn remove_io(cbdq: PFLT_CALLBACK_DATA_QUEUE, cbd: PFLT_CALLBACK_DATA) {
    unimplemented!("to do");
}

pub unsafe extern "C" fn peeknext_io(
    cbdq: PFLT_CALLBACK_DATA_QUEUE,
    cbd: PFLT_CALLBACK_DATA,
    peek_context: PVOID,
) -> PFLT_CALLBACK_DATA {
    unimplemented!("to do");
    core::ptr::null_mut()
}

pub unsafe extern "C" fn acquire(cbdq: PFLT_CALLBACK_DATA_QUEUE, irql: PKIRQL) {
    unimplemented!("to do");
}

pub unsafe extern "C" fn release(cbdq: PFLT_CALLBACK_DATA_QUEUE, irql: KIRQL) {
    unimplemented!("to do");
}

pub unsafe extern "C" fn complete_canceled_io(
    cbdq: PFLT_CALLBACK_DATA_QUEUE,
    cbd: PFLT_CALLBACK_DATA,
) {
    unimplemented!("to do");
}
