#![no_std]

use core::option::Option;
use core::panic::PanicInfo;

use fltkernel::{
    FLT_FILTER_UNLOAD_FLAGS, FLT_REGISTRATION, FLT_REGISTRATION_VERSION, FltRegisterFilter,
    FltUnregisterFilter, KeBugCheck, NTSTATUS, PDRIVER_OBJECT, PFLT_FILTER, PUNICODE_STRING,
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { KeBugCheck(0xbadbad) }
}

static mut FILTER: PFLT_FILTER = core::ptr::null_mut();

#[unsafe(no_mangle)]
extern "system" fn DriverEntry(driver: PDRIVER_OBJECT, _reg_path: PUNICODE_STRING) -> NTSTATUS {
    let mut flt_reg = FLT_REGISTRATION::default();
    flt_reg.Size = core::mem::size_of_val(&flt_reg) as u16;
    flt_reg.Version = FLT_REGISTRATION_VERSION as u16;
    flt_reg.FilterUnloadCallback = Some(unload);

    unsafe { FltRegisterFilter(driver, &raw const flt_reg, &raw mut FILTER) }
}

extern "C" fn unload(_flags: FLT_FILTER_UNLOAD_FLAGS) -> NTSTATUS {
    unsafe {
        FltUnregisterFilter(FILTER);
    }

    0
}
