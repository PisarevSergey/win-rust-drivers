#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(suspicious_runtime_symbol_definitions)]

#[cfg(feature = "wdm")]
mod wdm;

#[cfg(feature = "wdm")]
pub use wdm::*;

#[cfg(feature = "ntddk")]
mod ntddk;

#[cfg(feature = "ntddk")]
pub use ntddk::*;

#[cfg(feature = "fltkernel")]
mod fltkernel;

#[cfg(feature = "fltkernel")]
pub use fltkernel::*;

pub const STATUS_SUCCESS: NTSTATUS = 0;

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! kd_print_ln
{
    ($fmt:expr $(, $arg:expr)*) => {
        let c_fmt = concat!($fmt, "\n\0");
        unsafe {
            crate::DbgPrint(c_fmt.as_ptr() as crate::PCSTR $(, $arg)*);
        }
    }
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! kd_print_ln {
    ($($tokens:tt)*) => {};
}

pub struct AutoDeleteDevice {
    device: PDEVICE_OBJECT,
}

impl Drop for AutoDeleteDevice {
    fn drop(&mut self) {
        kd_print_ln!("call IoDeleteDevice for %p", self.device);
        unsafe { IoDeleteDevice(self.device) };
    }
}

pub fn create_device_secure(
    driver_object: &mut DRIVER_OBJECT,
    device_extension_size: ULONG,
    device_name: core::option::Option<&mut UNICODE_STRING>,
    device_type: ULONG,
    device_characteristics: ULONG,
    exclusive: bool,
    default_sddl: &UNICODE_STRING,
    device_class: core::option::Option<&GUID>,
) -> core::result::Result<AutoDeleteDevice, NTSTATUS> {
    let mut cdo = PDEVICE_OBJECT::default();
    let device_name = match device_name {
        Some(device_name) => &raw mut *device_name,
        None => core::ptr::null_mut(),
    };

    let device_class = match device_class {
        Some(device_class) => &raw const *device_class,
        None => core::ptr::null(),
    };

    let status = unsafe {
        WdmlibIoCreateDeviceSecure(
            &raw mut *driver_object,
            device_extension_size,
            device_name,
            device_type,
            device_characteristics,
            if exclusive {
                BOOLEAN::from(1)
            } else {
                BOOLEAN::from(0)
            },
            &raw const *default_sddl,
            device_class,
            &raw mut cdo,
        )
    };
    if status != STATUS_SUCCESS {
        return Err(status);
    }

    Ok(AutoDeleteDevice { device: cdo })
}

pub fn get_current_stack_location(irp: &mut IRP) -> &mut IO_STACK_LOCATION {
    unsafe {
        &mut *irp
            .Tail
            .Overlay
            .__bindgen_anon_2
            .__bindgen_anon_1
            .CurrentStackLocation
    }
}

pub fn get_irp_status(irp: &mut IRP) -> &mut NTSTATUS {
    unsafe { &mut irp.IoStatus.__bindgen_anon_1.Status }
}
