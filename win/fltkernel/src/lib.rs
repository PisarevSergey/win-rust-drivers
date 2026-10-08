#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use core::ops::Deref;

#[macro_export]
macro_rules! c_str {
    ($s:literal) => {
        match core::ffi::CStr::from_bytes_with_nul(concat!($s, "\0").as_bytes()) {
            Ok(cstr) => cstr.as_ptr(),
            Err(_) => panic!("invalid format: {}", $s),
        }
    };
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! kd_print
{
    ($fmt:expr $(, $arg:expr)*) => {
        unsafe {
            DbgPrint($fmt $(, $arg)*);
        }
    }
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! kd_print {
    ($($tokens:tt)*) => {};
}

pub struct FltFileNameInformation {
    name: PFLT_FILE_NAME_INFORMATION,
}

impl Drop for FltFileNameInformation {
    fn drop(&mut self) {
        unsafe { FltReleaseFileNameInformation(self.name) };
    }
}

impl Deref for FltFileNameInformation {
    type Target = FLT_FILE_NAME_INFORMATION;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.name }
    }
}

pub fn query_file_name_information(
    cbd: &mut FLT_CALLBACK_DATA,
    file_name_options: FLT_FILE_NAME_OPTIONS,
) -> core::result::Result<FltFileNameInformation, NTSTATUS> {
    let mut fni: PFLT_FILE_NAME_INFORMATION = core::ptr::null_mut();

    let status =
        unsafe { FltGetFileNameInformation(&raw mut *cbd, file_name_options, &raw mut fni) };
    if status != 0 {
        kd_print!(
            c_str!("FltGetFileNameInformation failed with status 0x%x\n"),
            status
        );
        return Err(status);
    }

    Ok(FltFileNameInformation { name: fni })
}
