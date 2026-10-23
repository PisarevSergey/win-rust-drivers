use fltkernel::{
    FLT_CALLBACK_DATA_QUEUE, FLT_CONTEXT_TYPE, LIST_ENTRY, PFLT_CONTEXT, create_list_head,
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

pub unsafe extern "C" fn instance_context_cleanup(
    context: PFLT_CONTEXT,
    _context_type: FLT_CONTEXT_TYPE,
) {
    unsafe { core::ptr::drop_in_place(context as *mut InstanceContext) };
}
