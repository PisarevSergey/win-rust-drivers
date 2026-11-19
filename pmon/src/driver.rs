use crate::process_notify::ProcessCreateNotify;
use win::{
    AutoHandle, FILE_NON_DIRECTORY_FILE, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    FILE_SYNCHRONOUS_IO_NONALERT, GENERIC_READ, GENERIC_WRITE, HANDLE, IO_STATUS_BLOCK, NTSTATUS,
    OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE, OBJECT_ATTRIBUTES, PWCH, STATUS_SUCCESS,
    UNICODE_STRING, kd_print_ln,
};

use core::option::Option;
use core::result::Result;

const CDO_NAME_RAW: &[u16] = wchar::wchz!(r"\Device\central_cdo");
const CDO_NAME: UNICODE_STRING = UNICODE_STRING {
    Length: 2 * CDO_NAME_RAW.len() as u16,
    MaximumLength: 2 * CDO_NAME_RAW.len() as u16,
    Buffer: CDO_NAME_RAW.as_ptr() as PWCH,
};

pub struct Driver {
    central: Option<AutoHandle>,
    #[allow(dead_code)]
    proc_create_notify: Option<ProcessCreateNotify>,
}

impl Driver {
    pub const fn new() -> Self {
        Driver {
            central: None,
            proc_create_notify: None,
        }
    }

    pub fn try_create() -> Result<Self, NTSTATUS> {
        let central = open_central()?;
        let pn = ProcessCreateNotify::try_create()?;

        Ok(Self {
            central: Some(central),
            proc_create_notify: Some(pn),
        })
    }
}

pub fn open_central() -> Result<AutoHandle, NTSTATUS> {
    let mut cdo_name = CDO_NAME;

    let mut oa = OBJECT_ATTRIBUTES::default();
    oa.Length = core::mem::size_of_val(&oa) as u32;
    oa.ObjectName = &raw mut cdo_name;
    oa.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;

    let mut iosb = IO_STATUS_BLOCK::default();
    let mut central_handle = HANDLE::default();
    let status = unsafe {
        win::ZwOpenFile(
            &raw mut central_handle,
            GENERIC_WRITE,
            &raw mut oa,
            &raw mut iosb,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
        )
    };
    if status != STATUS_SUCCESS {
        kd_print_ln!(
            "open_central: ZwOpenFile failed for %wZ with status 0x%x",
            &raw const cdo_name,
            status
        );
        return Err(status);
    }

    Ok(unsafe { AutoHandle::from_raw_handle(central_handle) })
}
