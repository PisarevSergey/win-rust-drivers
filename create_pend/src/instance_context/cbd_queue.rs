use fltkernel::{
    _FLT_INSTANCE, _FLT_PREOP_CALLBACK_STATUS_FLT_PREOP_COMPLETE as FLT_PREOP_COMPLETE,
    FLT_CALLBACK_DATA, FLT_CALLBACK_DATA_QUEUE, FltCompletePendedPreOperation, KIRQL, NTSTATUS,
    PFLT_CALLBACK_DATA, PFLT_CALLBACK_DATA_QUEUE, PKIRQL, PLIST_ENTRY, PVOID, insert_tail_list,
    remove_entry_list,
};

use crate::{STATUS_CANCELLED, STATUS_SUCCESS, instance_context};

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
    _peek_context: PVOID,
) -> PFLT_CALLBACK_DATA {
    let ic = unsafe { &*(cbdq as *const instance_context::InstanceContext) };

    let next_entry = if cbd.is_null() {
        ic.pended_cbds.Flink
    } else {
        unsafe { (*get_list_entry(cbd)).Flink }
    };

    if core::ptr::addr_eq(next_entry, &ic.pended_cbds) {
        return core::ptr::null_mut();
    }

    let offset = core::mem::offset_of!(
        FLT_CALLBACK_DATA,
        __bindgen_anon_1.__bindgen_anon_1.QueueLinks
    );

    let next_entry = unsafe { next_entry.byte_sub(offset) };

    next_entry as PFLT_CALLBACK_DATA
}

unsafe extern "C" fn acquire(cbdq: PFLT_CALLBACK_DATA_QUEUE, _irql: PKIRQL) {
    let ic = unsafe { &mut *(cbdq as *mut instance_context::InstanceContext) };
    ic.cbds_lock.acquire_exclusive();
}

unsafe extern "C" fn release(cbdq: PFLT_CALLBACK_DATA_QUEUE, _irql: KIRQL) {
    let ic = unsafe { &mut *(cbdq as *mut instance_context::InstanceContext) };
    ic.cbds_lock.release();
}

unsafe extern "C" fn complete_canceled_io(
    _cbdq: PFLT_CALLBACK_DATA_QUEUE,
    cbd: PFLT_CALLBACK_DATA,
) {
    let cbd = unsafe { &mut *cbd };

    cbd.IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
    cbd.IoStatus.Information = 0;

    unsafe { FltCompletePendedPreOperation(cbd, FLT_PREOP_COMPLETE, core::ptr::null_mut()) };
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
