#![no_std]

use core::panic::PanicInfo;

use win::{NTSTATUS, PDRIVER_OBJECT, PUNICODE_STRING, STATUS_SUCCESS};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { win::KeBugCheck(0xbadbad) }
}

#[unsafe(no_mangle)]
extern "system" fn DriverEntry(driver: PDRIVER_OBJECT, reg_path: PUNICODE_STRING) -> NTSTATUS {
    STATUS_SUCCESS
}
