use fltkernel::{FLT_CALLBACK_DATA_QUEUE, FLT_CONTEXT_TYPE, PFLT_CONTEXT};

#[derive(Default)]
pub struct InstanceContext {
    cbdq: FLT_CALLBACK_DATA_QUEUE,
}

pub unsafe extern "C" fn instance_context_cleanup(
    context: PFLT_CONTEXT,
    _context_type: FLT_CONTEXT_TYPE,
) {
    unsafe { core::ptr::drop_in_place(context as *mut InstanceContext) };
}
