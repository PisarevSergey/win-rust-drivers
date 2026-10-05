#![no_std]

use core::panic::PanicInfo;

use fltkernel::{
    DbgPrint, FLT_FILTER_UNLOAD_FLAGS, FLT_REGISTRATION, FLT_REGISTRATION_VERSION,
    FltRegisterFilter, FltUnregisterFilter, KeBugCheck, NTSTATUS, PDRIVER_OBJECT, PFLT_FILTER,
    PUNICODE_STRING, kd_print, c_str
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { KeBugCheck(0xbadbad) }
}

static mut FILTER: PFLT_FILTER = core::ptr::null_mut();

#[unsafe(no_mangle)]
extern "system" fn DriverEntry(driver: PDRIVER_OBJECT, reg_path: PUNICODE_STRING) -> NTSTATUS {
    kd_print!(c_str!("create_pend: enter DriverEntry; driver object %p; reg path %wZ\n"), driver, reg_path);

    let mut flt_reg = FLT_REGISTRATION::default();
    flt_reg.Size = core::mem::size_of_val(&flt_reg) as u16;
    flt_reg.Version = FLT_REGISTRATION_VERSION as u16;
    flt_reg.FilterUnloadCallback = Some(unload);

    let status = unsafe { FltRegisterFilter(driver, &raw const flt_reg, &raw mut FILTER) };
    kd_print!(c_str!("create_pend: FltRegisterFilter status: 0x%x\n"), status);

    status
}

extern "C" fn unload(_flags: FLT_FILTER_UNLOAD_FLAGS) -> NTSTATUS {
    kd_print!(c_str!("create_pend: start unloading\n"));

    unsafe {
        FltUnregisterFilter(FILTER);
    }

    kd_print!(c_str!("create_pend: unloading complete\n"));

    0
}
