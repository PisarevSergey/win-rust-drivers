use crate::process_notify::ProcessCreateNotify;
use win::{
    AutoRefObject, DEVICE_OBJECT, FILE_WRITE_DATA, NTSTATUS, PDEVICE_OBJECT, PFILE_OBJECT, PVOID,
    PWCH, STATUS_SUCCESS, UNICODE_STRING, kd_print_ln,
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
    central: Option<AutoRefObject<DEVICE_OBJECT>>,
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

pub fn open_central() -> Result<AutoRefObject<DEVICE_OBJECT>, NTSTATUS> {
    let mut cdo_name = CDO_NAME;

    let mut fo = PFILE_OBJECT::default();
    let mut dev_object = PDEVICE_OBJECT::default();
    let status = unsafe {
        win::IoGetDeviceObjectPointer(
            &raw mut cdo_name,
            FILE_WRITE_DATA,
            &raw mut fo,
            &raw mut dev_object,
        )
    };
    if status != STATUS_SUCCESS {
        kd_print_ln!(
            "IoGetDeviceObjectPointer failed for %wZ with status 0x%x",
            &raw const cdo_name,
            status
        );
        return Err(status);
    }
    let _fo = unsafe { AutoRefObject::from_raw_ptr(fo) };

    unsafe { win::ObfReferenceObject(dev_object as PVOID) };
    Ok(unsafe { AutoRefObject::from_raw_ptr(dev_object) })
}
