use crate::process_notify::ProcessCreateNotify;
use win::{
    AutoHandle, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SYNCHRONOUS_IO_NONALERT, GENERIC_WRITE,
    HANDLE, IO_STATUS_BLOCK, NTSTATUS, OBJECT_ATTRIBUTES, STATUS_SUCCESS, UNICODE_STRING,
    kd_print_ln,
};

use core::option::Option;
use core::result::Result;

pub struct Driver {
    #[allow(dead_code)]
    proc_create_notify: Option<ProcessCreateNotify>,
}

impl Driver {
    pub const fn new() -> Self {
        Driver {
            proc_create_notify: None,
        }
    }

    pub fn try_create() -> Result<Self, NTSTATUS> {
        let pn = match ProcessCreateNotify::try_create() {
            Ok(pn) => pn,
            Err(status) => return Err(status),
        };

        Ok(Self {
            proc_create_notify: Some(pn),
        })
    }
}

pub fn open_central() -> Result<AutoHandle, NTSTATUS> {
    let mut oa = OBJECT_ATTRIBUTES::default();
    oa.Length = core::mem::size_of_val(&oa) as u32;
    oa.Attributes = win::OBJ_KERNEL_HANDLE;

    const CDO_NAME_RAW: &[u16] = wchar::wchz!(r"\Device\central_cdo");

    let mut cdo_name = UNICODE_STRING::default();
    unsafe { win::RtlInitUnicodeString(&raw mut cdo_name, CDO_NAME_RAW.as_ptr()) };
    oa.ObjectName = &raw mut cdo_name;

    let mut iosb = IO_STATUS_BLOCK::default();
    let mut central = HANDLE::default();
    let status = unsafe {
        win::ZwOpenFile(
            &raw mut central,
            GENERIC_WRITE,
            &raw mut oa,
            &raw mut iosb,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_SYNCHRONOUS_IO_NONALERT,
        )
    };
    if status != STATUS_SUCCESS {
        kd_print_ln!(
            "ZwOpenFile failed for %wZ with status 0x%x",
            &raw const cdo_name,
            status
        );
        return Err(status);
    }

    Ok(unsafe { AutoHandle::from_raw_handle(central) })
}
