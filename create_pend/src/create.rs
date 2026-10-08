use fltkernel::{
    DbgPrint, FLT_FILE_NAME_OPENED, FLT_FILE_NAME_QUERY_DEFAULT, FLT_PREOP_CALLBACK_STATUS,
    PCFLT_RELATED_OBJECTS, PFLT_CALLBACK_DATA, PVOID, c_str, kd_print,
};

use fltkernel::_FLT_PREOP_CALLBACK_STATUS_FLT_PREOP_SUCCESS_NO_CALLBACK as FLT_PREOP_SUCCESS_NO_CALLBACK;

pub unsafe extern "C" fn pre_create(
    data: PFLT_CALLBACK_DATA,
    _flt_objects: PCFLT_RELATED_OBJECTS,
    _completion_context: *mut PVOID,
) -> FLT_PREOP_CALLBACK_STATUS {
    let cbd = unsafe { &*data };
    match fltkernel::MODE::from(cbd.RequestorMode) {
        fltkernel::_MODE_KernelMode => {
            kd_print!(c_str!("create from kernel mode\n"))
        }
        fltkernel::_MODE_UserMode => {
            kd_print!(c_str!("create from user mode\n"))
        }
        mode => {
            kd_print!(c_str!("unexpected requestor mode %d\n"), mode)
        }
    }

    match fltkernel::query_file_name_information(
        unsafe { &mut *data },
        FLT_FILE_NAME_QUERY_DEFAULT | FLT_FILE_NAME_OPENED,
    ) {
        Ok(fni) => {
            kd_print!(
                c_str!("[create_pend]: create file: %wZ\n"),
                &raw const fni.Name
            )
        }
        Err(status) => {
            kd_print!(
                c_str!("[create_pend]: query name failed with status 0x%x\n"),
                status
            )
        }
    };

    FLT_PREOP_SUCCESS_NO_CALLBACK
}
