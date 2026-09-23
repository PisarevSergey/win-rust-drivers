#![no_std]

use core::ffi::c_void;
use core::panic::PanicInfo;

unsafe extern "C" {
    fn KeBugCheck(code: u32) -> !;
    fn FltRegisterFilter(
        driver: *mut c_void,
        registration: *const c_void,
        filter: *mut usize,
    ) -> i32;

}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { KeBugCheck(0xbadbad) }
}

#[unsafe(no_mangle)]
extern "system" fn DriverEntry(_driver: *mut c_void, _reg_path: *mut c_void) -> i32 {
    let mut filter: usize = 0;
    unsafe{
        FltRegisterFilter(core::ptr::null_mut(), core::ptr::null(), &raw mut filter)
    }
}
