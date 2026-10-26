use fltkernel::{
    _FLT_INSTANCE, FLT_CALLBACK_DATA_QUEUE, KIRQL, NTSTATUS, PFLT_CALLBACK_DATA,
    PFLT_CALLBACK_DATA_QUEUE, PKIRQL, PLIST_ENTRY, PVOID, insert_tail_list, remove_entry_list,
};

use crate::{STATUS_SUCCESS, instance_context};

unsafe extern "C" fn insert_io(
    cbdq: PFLT_CALLBACK_DATA_QUEUE,
    cbd: PFLT_CALLBACK_DATA,
    _insert_context: PVOID,
) -> NTSTATUS {
    let ic = unsafe { &mut *(cbdq as *mut instance_context::InstanceContext) };
    insert_tail_list(&mut ic.pended_cbds, get_list_entry(cbd));

    STATUS_SUCCESS
}

unsafe extern "C" fn remove_io(_cbdq: PFLT_CALLBACK_DATA_QUEUE, cbd: PFLT_CALLBACK_DATA) {
    remove_entry_list(get_list_entry(cbd));
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

unsafe extern "C" fn complete_canceled_io(cbdq: PFLT_CALLBACK_DATA_QUEUE, cbd: PFLT_CALLBACK_DATA) {
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

fn get_list_entry(cbd: PFLT_CALLBACK_DATA) -> PLIST_ENTRY {
    unsafe { &raw mut ((*cbd).__bindgen_anon_1.__bindgen_anon_1.QueueLinks) }
}
