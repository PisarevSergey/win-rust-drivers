#![no_std]

mod driver;
mod process_notify;

use core::panic::PanicInfo;

use win::{
    kd_print_ln, DbgPrint, NTSTATUS, PCSTR, PDRIVER_OBJECT, PUNICODE_STRING, STATUS_SUCCESS,
};

#[unsafe(no_mangle)]
extern "system" fn DriverEntry(driver: PDRIVER_OBJECT, reg_path: PUNICODE_STRING) -> NTSTATUS {
    kd_print_ln!("DriverEntry: driver - %p; reg path: %wZ", driver, reg_path);

    let driver = unsafe { &mut *driver };
    driver.DriverUnload = Some(unload);

    if let Err(status) = driver::get_driver().init() {
        kd_print_ln!("driver init failed with status 0x%x", status);
        return status;
    };

    kd_print_ln!("driver start success");
    STATUS_SUCCESS
}

unsafe extern "C" fn unload(driver_object: PDRIVER_OBJECT) {
    kd_print_ln!("unloading driver %p", driver_object);
    driver::destroy_driver();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { win::KeBugCheck(0xbadbad) }
}
