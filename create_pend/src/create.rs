use fltkernel::{
    _FLT_FILTER as FLT_FILTER, _FLT_PREOP_CALLBACK_STATUS_FLT_PREOP_PENDING as FLT_PREOP_PENDING,
    _FLT_PREOP_CALLBACK_STATUS_FLT_PREOP_SUCCESS_NO_CALLBACK as FLT_PREOP_SUCCESS_NO_CALLBACK,
    DbgPrint, FLT_CALLBACK_DATA, FLT_FILE_NAME_OPENED, FLT_FILE_NAME_QUERY_DEFAULT,
    FLT_PREOP_CALLBACK_STATUS, NTSTATUS, PCFLT_RELATED_OBJECTS, PFLT_CALLBACK_DATA, PVOID,
    UNICODE_STRING, c_str, kd_print,
};

use crate::{STATUS_SUCCESS, instance_context::InstanceContext};

pub unsafe extern "C" fn pre_create(
    data: PFLT_CALLBACK_DATA,
    flt_objects: PCFLT_RELATED_OBJECTS,
    _completion_context: *mut PVOID,
) -> FLT_PREOP_CALLBACK_STATUS {
    let cbd = unsafe { &mut *data };

    report_cbd(unsafe { &mut (*(*flt_objects).Filter) }, cbd);

    let mut ic = match fltkernel::get_instance_context::<InstanceContext>(unsafe {
        &mut (*(*flt_objects).Instance)
    }) {
        Ok(ic) => ic,
        Err(status) => {
            kd_print!(
                c_str!("get_instance_context failed with status 0x%x\n"),
                status
            );
            return FLT_PREOP_SUCCESS_NO_CALLBACK;
        }
    };

    if let Err(status) = ic.pend_cbd(data) {
        kd_print!(c_str!("pend_cbd failed with status x%x\n"), status);
        return FLT_PREOP_SUCCESS_NO_CALLBACK;
    };

    FLT_PREOP_PENDING
}

pub fn report_cbd(filter: &mut FLT_FILTER, cbd: &mut FLT_CALLBACK_DATA) {
    report_requestor_mode(cbd);
    report_file_path(cbd);
    report_extended_attributes(cbd);
    report_ecp_list(filter, cbd);
}

fn report_requestor_mode(cbd: &FLT_CALLBACK_DATA) {
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
}

fn report_file_path(cbd: &mut FLT_CALLBACK_DATA) {
    match fltkernel::query_file_name_information(
        cbd,
        FLT_FILE_NAME_QUERY_DEFAULT | FLT_FILE_NAME_OPENED,
    ) {
        Ok(fni) => {
            kd_print!(c_str!("create file: %wZ\n"), &raw const fni.Name)
        }
        Err(status) => {
            kd_print!(c_str!("query name failed with status 0x%x\n"), status)
        }
    };
}

fn report_extended_attributes(cbd: &FLT_CALLBACK_DATA) {
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

            ea_buffer = unsafe { ea_buffer.byte_add(ea_info.NextEntryOffset as usize) };
        }
    }
}

fn report_ecp_list(filter: &mut FLT_FILTER, cbd: &mut FLT_CALLBACK_DATA) {
    let Ok(ecp_list) = fltkernel::get_extra_create_parameters(filter, cbd) else {
        return;
    };

    for ecp in ecp_list {
        if let Ok(ecp_string) = guid_to_string(&ecp) {
            kd_print!(c_str!("ECP: %wZ\n"), &raw const *ecp_string.get_string());
        }
    }
}

fn guid_to_string(guid: &fltkernel::GUID) -> Result<GuidString, NTSTATUS> {
    let mut string = UNICODE_STRING::default();
    let status = unsafe { fltkernel::RtlStringFromGUID(&raw const *guid, &raw mut string) };
    if status != STATUS_SUCCESS {
        return Err(status);
    }

    Ok(GuidString { string: string })
}

struct GuidString {
    string: UNICODE_STRING,
}

impl GuidString {
    fn get_string(&self) -> &UNICODE_STRING {
        &self.string
    }
}

impl Drop for GuidString {
    fn drop(&mut self) {
        unsafe { fltkernel::RtlFreeUnicodeString(&raw mut self.string) };
    }
}
