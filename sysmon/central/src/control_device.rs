use win::{
    AutoDeleteDevice, DRIVER_OBJECT, GUID, IoCreateSymbolicLink, NTSTATUS, STATUS_SUCCESS,
    create_device_secure, kd_print_ln,
};

use sysmon_common::{CENTRAL_CDO_NAME, CENTRAL_DOS_NAME};

const CDO_CLASS: GUID = GUID {
    Data1: 0xf81d4fae,
    Data2: 0x7dec,
    Data3: 0x41d0,
    Data4: [0xa7, 0x65, 00, 0xa0, 0xc9, 0x1e, 0x6b, 0xf6],
};

pub fn try_create_device(
    driver_object: &mut DRIVER_OBJECT,
) -> core::result::Result<AutoDeleteDevice, NTSTATUS> {
    let mut cdo_name = win::to_unicode_string(CENTRAL_CDO_NAME);
    create_device_secure(
        driver_object,
        0,
        Some(&mut cdo_name),
        win::FILE_DEVICE_UNKNOWN,
        win::FILE_DEVICE_SECURE_OPEN,
        false,
        unsafe { &win::SDDL_DEVOBJ_SYS_ALL_ADM_ALL },
        Some(&CDO_CLASS),
    )
}

pub struct SymLink;

impl SymLink {
    pub fn try_create() -> core::result::Result<Self, NTSTATUS> {
        let mut dos_name = win::to_unicode_string(CENTRAL_DOS_NAME);
        let link = &raw mut dos_name;

        let mut cdo_name = win::to_unicode_string(CENTRAL_CDO_NAME);
        let name = &raw mut cdo_name;

        let status = unsafe { IoCreateSymbolicLink(link, name) };

        if status != STATUS_SUCCESS {
            kd_print_ln!(
                "IoCreateSymbolicLink failed %wZ -> %wZ with status 0x%x",
                link,
                name,
                status
            );
            return Err(status);
        }

        kd_print_ln!("IoCreateSymbolicLink success %wZ -> %wZ", link, name);
        Ok(Self {})
    }
}

impl Drop for SymLink {
    fn drop(&mut self) {
        let mut dos_name = win::to_unicode_string(CENTRAL_DOS_NAME);
        let link = &raw mut dos_name;
        kd_print_ln!("IoDeleteSymbolicLink %wZ", link);
        unsafe { win::IoDeleteSymbolicLink(link) };
    }
}
