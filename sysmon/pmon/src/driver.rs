use crate::process_notify::ProcessCreateNotify;
use win::{
    kd_print_ln, AutoHandle, FILE_NON_DIRECTORY_FILE, FILE_SHARE_DELETE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, FILE_SYNCHRONOUS_IO_NONALERT, GENERIC_WRITE, HANDLE, IO_STATUS_BLOCK,
    NTSTATUS, OBJECT_ATTRIBUTES, OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE, PWCH, STATUS_SUCCESS,
    UNICODE_STRING,
};

use core::option::Option;
use core::result::Result;

use sysmon_common::CENTRAL_CDO_NAME;

static mut DRIVER: Driver = Driver::new();

pub fn get_driver() -> &'static mut Driver {
    unsafe { &mut *&raw mut DRIVER }
}

pub fn destroy_driver() {
    unsafe { core::ptr::drop_in_place(&raw mut DRIVER) };
}

const CDO_NAME: UNICODE_STRING = UNICODE_STRING {
    Length: 2 * CENTRAL_CDO_NAME.len() as u16,
    MaximumLength: 2 * CENTRAL_CDO_NAME.len() as u16,
    Buffer: CENTRAL_CDO_NAME.as_ptr() as PWCH,
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

    pub fn init(&mut self) -> Result<NTSTATUS, NTSTATUS> {
        self.central = Some(open_central()?);
        self.proc_create_notify = Some(ProcessCreateNotify::try_create()?);

        Ok(STATUS_SUCCESS)
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
