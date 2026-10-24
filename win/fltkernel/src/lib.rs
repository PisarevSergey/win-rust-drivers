#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod callback_data_queue;

const STATUS_NOT_FOUND: NTSTATUS = 0xc000_0225_u32.cast_signed();

use _FLT_SET_CONTEXT_OPERATION_FLT_SET_CONTEXT_KEEP_IF_EXISTS as FLT_SET_CONTEXT_KEEP_IF_EXISTS;
use _POOL_TYPE_NonPagedPoolNx as NonPagedPoolNx;

use core::ops::{Deref, DerefMut};

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

pub struct ExtraCreateParameters<'a> {
    ecp_list: &'a mut ECP_LIST,
    filter: &'a mut _FLT_FILTER,
    current_ecp_context: PVOID,
}

impl<'a> ExtraCreateParameters<'a> {
    fn new(filter: &'a mut _FLT_FILTER, list: &'a mut ECP_LIST) -> Self {
        Self {
            ecp_list: list,
            filter: filter,
            current_ecp_context: core::ptr::null_mut(),
        }
    }
}

impl<'a> Iterator for ExtraCreateParameters<'a> {
    type Item = GUID;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_ecp_type = GUID::default();
        let mut next_ecp_ctx_size = ULONG::default();

        let status = unsafe {
            FltGetNextExtraCreateParameter(
                &raw mut *self.filter,
                &raw mut *self.ecp_list,
                self.current_ecp_context,
                &raw mut next_ecp_type,
                &raw mut self.current_ecp_context,
                &raw mut next_ecp_ctx_size,
            )
        };
        if status == 0 {
            return Some(next_ecp_type);
        } else if status == STATUS_NOT_FOUND {
            // empty ECP list or last ECP
        } else {
            kd_print!(
                c_str!("FltGetNextExtraCreateParameter failed with status 0x%x\n"),
                status
            );
        }

        None
    }
}

pub fn get_extra_create_parameters<'a>(
    filter: &'a mut _FLT_FILTER,
    cbd: &'a mut FLT_CALLBACK_DATA,
) -> core::result::Result<ExtraCreateParameters<'a>, NTSTATUS> {
    let mut ecp_list: PECP_LIST = core::ptr::null_mut();
    let status = unsafe {
        FltGetEcpListFromCallbackData(&raw mut *filter, &raw mut *cbd, &raw mut ecp_list)
    };
    if status != 0 {
        kd_print!(
            c_str!("FltGetEcpListFromCallbackData failed with status 0x%x\n"),
            status
        );
        return Err(status);
    }

    if ecp_list.is_null() {
        return Err(0);
    }

    Ok(ExtraCreateParameters::new(filter, unsafe {
        &mut *ecp_list
    }))
}

pub struct FltContext<T> {
    ctx: *mut T,
}

impl<T> Drop for FltContext<T> {
    fn drop(&mut self) {
        unsafe { FltReleaseContext(self.ctx as PFLT_CONTEXT) };
    }
}

impl<T> Deref for FltContext<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ctx }
    }
}

impl<T> DerefMut for FltContext<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ctx }
    }
}

pub fn try_create_context<T: Default>(
    filter: &mut _FLT_FILTER,
    ctx_type: FLT_CONTEXT_TYPE,
    context_size: core::option::Option<SIZE_T>,
) -> core::result::Result<FltContext<T>, NTSTATUS> {
    let mut ctx_raw_mem = core::ptr::null_mut::<T>();
    let alloc_size = context_size.unwrap_or(core::mem::size_of::<T>() as SIZE_T);
    let status = unsafe {
        FltAllocateContext(
            &raw mut *filter,
            ctx_type,
            alloc_size,
            NonPagedPoolNx,
            &raw mut ctx_raw_mem as *mut PFLT_CONTEXT,
        )
    };
    if status != 0 {
        return Err(status);
    }

    unsafe { core::ptr::write(ctx_raw_mem as *mut T, T::default()) };
    Ok(FltContext { ctx: ctx_raw_mem })
}

pub fn set_instance_context_if_none<T>(
    instance: &mut _FLT_INSTANCE,
    context: &FltContext<T>,
) -> core::result::Result<NTSTATUS, NTSTATUS> {
    let status = unsafe {
        FltSetInstanceContext(
            &raw mut *instance,
            FLT_SET_CONTEXT_KEEP_IF_EXISTS,
            context.ctx as PFLT_CONTEXT,
            core::ptr::null_mut(),
        )
    };

    if status != 0 {
        kd_print!(
            c_str!("FltSetInstanceContext failed with status 0x%x\n"),
            status
        );
        return Err(status);
    }

    Ok(status)
}

pub fn initialize_list_head(head: &mut LIST_ENTRY) {
    head.Flink = core::ptr::from_mut(head);
    head.Blink = core::ptr::from_mut(head);
}

pub use crate::callback_data_queue::try_create_cbd_queue;
