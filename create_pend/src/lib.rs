#![no_std]

use core::ffi::c_void;
use core::panic::PanicInfo;

use fltkernel::{KeBugCheck, FltRegisterFilter, PFLT_FILTER};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { KeBugCheck(0xbadbad) }
}

#[unsafe(no_mangle)]
extern "system" fn DriverEntry(_driver: *mut c_void, _reg_path: *mut c_void) -> i32 {
    let mut filter: PFLT_FILTER = core::ptr::null_mut();
    unsafe{
        FltRegisterFilter(core::ptr::null_mut(), core::ptr::null(), &raw mut filter)
    }
}
