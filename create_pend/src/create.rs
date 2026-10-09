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

    let iopb = unsafe { &*cbd.Iopb };
    let params = unsafe { iopb.Parameters.Create };

    let mut ea_buffer = params.EaBuffer;
    if !ea_buffer.is_null() && params.EaLength != 0 {
        loop {
            let ea_info = ea_buffer as fltkernel::PFILE_FULL_EA_INFORMATION;
            let ea_info = unsafe { &mut *ea_info };

            let ea_name = fltkernel::ANSI_STRING {
                Length: u16::from(ea_info.EaNameLength),
                MaximumLength: u16::from(ea_info.EaNameLength),
                Buffer: ea_info.EaName.as_mut_ptr(),
            };

            kd_print!(
                c_str!("EA name: %Z, EA params at: %p, EA length: %d\n"),
                &raw const ea_name,
                params.EaBuffer,
                params.EaLength
            );

            if ea_info.NextEntryOffset == 0 {
                break;
            }

            ea_buffer = ea_buffer.byte_add(ea_info.NextEntryOffset as usize);
        }
    }

    FLT_PREOP_SUCCESS_NO_CALLBACK
}
