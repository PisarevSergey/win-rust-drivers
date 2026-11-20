#![no_std]

pub const CENTRAL_CDO_NAME: &[u16] = wchar::wchz!(r"\Device\central_cdo");
pub const CENTRAL_DOS_NAME: &[u16] = wchar::wchz!(r"\DosDevices\central_cdo");

#[repr(C, packed)]
#[derive(Default)]
pub struct ProcessNotify {
    pub eprocess: u64,
    pub pid: u64,
    pub ps_create_notify_info: u64,
}
