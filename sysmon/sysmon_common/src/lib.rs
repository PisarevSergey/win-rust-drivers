#![no_std]

use core::ffi::c_void;
use core::option::Option;

pub const CENTRAL_CDO_NAME: &[u16] = wchar::wchz!(r"\Device\central_cdo");
pub const CENTRAL_DOS_NAME: &[u16] = wchar::wchz!(r"\DosDevices\central_cdo");

pub const IOCTL_CENTRAL_PROCESS_START_STOP_NOTIFY: u32 =
    ctl_code(FILE_DEVICE_UNKNOWN, 0x800, METHOD_BUFFERED, FILE_WRITE_DATA);

/// kernel only, consumed by central
#[repr(C, packed)]
pub struct ProcessNotify<'a> {
    pub pid: *const c_void,
    pub event: ProcessEventType<'a>,
}

#[repr(C)]
pub enum ProcessEventType<'a> {
    Start(&'a ProcessStartInfo<'a>),
    Exit,
}

pub struct ProcessStartInfo<'a> {
    pub ppid: *const c_void,
    pub image_path: Option<&'a [u16]>,
    pub command_line: Option<&'a [u16]>,
}

const fn ctl_code(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    device_type << 16 | access << 14 | function << 2 | method
}

const FILE_DEVICE_UNKNOWN: u32 = 0x0000_0022;
const METHOD_BUFFERED: u32 = 0;
const FILE_WRITE_DATA: u32 = 0x0002;
