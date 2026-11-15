#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(feature = "wdm")]
mod wdm;

#[cfg(feature = "ntddk")]
mod ntddk;

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
            DbgPrint(c_fmt.as_ptr() as PCSTR $(, $arg)*);
        }
    }
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! kd_print_ln {
    ($($tokens:tt)*) => {};
}
