#![no_std]

mod driver;
mod process_notify;

use core::panic::PanicInfo;

use win::{
    DbgPrint, NTSTATUS, PCSTR, PDRIVER_OBJECT, PUNICODE_STRING, STATUS_SUCCESS, kd_print_ln,
};

use crate::driver::Driver;

static mut DRIVER: Driver = Driver::new();

#[unsafe(no_mangle)]
extern "system" fn DriverEntry(driver: PDRIVER_OBJECT, reg_path: PUNICODE_STRING) -> NTSTATUS {
    kd_print_ln!("DriverEntry: driver - %p; reg path: %wZ", driver, reg_path);

    let driver = unsafe { &mut *driver };
    driver.DriverUnload = Some(unload);

    match Driver::try_create() {
        Ok(drv) => unsafe { DRIVER = drv },
        Err(status) => return status,
    };

    kd_print_ln!("driver start success");
    STATUS_SUCCESS
}

unsafe extern "C" fn unload(driver_object: PDRIVER_OBJECT) {
    kd_print_ln!("unloading driver %p", driver_object);
    unsafe { core::ptr::drop_in_place(&raw mut DRIVER) };
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { win::KeBugCheck(0xbadbad) }
}
