use fltkernel::{
    _FLT_INSTANCE, FLT_CALLBACK_DATA_QUEUE, KIRQL, NTSTATUS, PFLT_CALLBACK_DATA,
    PFLT_CALLBACK_DATA_QUEUE, PKIRQL, PVOID,
};

use crate::instance_context;

unsafe extern "C" fn insert_io(
    cbdq: PFLT_CALLBACK_DATA_QUEUE,
    cbd: PFLT_CALLBACK_DATA,
    insert_context: PVOID,
) -> NTSTATUS {
    let ic = cbdq as *mut instance_context::InstanceContext;
    unimplemented!("to do");
    0
}

unsafe extern "C" fn remove_io(cbdq: PFLT_CALLBACK_DATA_QUEUE, cbd: PFLT_CALLBACK_DATA) {
    unimplemented!("to do");
}

unsafe extern "C" fn peeknext_io(
    cbdq: PFLT_CALLBACK_DATA_QUEUE,
    cbd: PFLT_CALLBACK_DATA,
    peek_context: PVOID,
) -> PFLT_CALLBACK_DATA {
    unimplemented!("to do");
    core::ptr::null_mut()
}

unsafe extern "C" fn acquire(cbdq: PFLT_CALLBACK_DATA_QUEUE, irql: PKIRQL) {
    unimplemented!("to do");
}

unsafe extern "C" fn release(cbdq: PFLT_CALLBACK_DATA_QUEUE, irql: KIRQL) {
    unimplemented!("to do");
}

unsafe extern "C" fn complete_canceled_io(
    cbdq: PFLT_CALLBACK_DATA_QUEUE,
    cbd: PFLT_CALLBACK_DATA,
) {
    unimplemented!("to do");
}

pub fn try_create_cbd_queue(
    instance: &mut _FLT_INSTANCE,
) -> Result<FLT_CALLBACK_DATA_QUEUE, NTSTATUS> {
    fltkernel::try_create_cbd_queue(
        instance,
        Some(insert_io),
        Some(remove_io),
        Some(peeknext_io),
        Some(acquire),
        Some(release),
        Some(complete_canceled_io),
    )
}
