#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(feature = "wdm")]
pub mod wdm;

#[cfg(feature = "ntddk")]
pub mod ntddk;

#[cfg(feature = "fltkernel")]
mod fltkernel;

#[cfg(feature = "fltkernel")]
pub use fltkernel::*;
