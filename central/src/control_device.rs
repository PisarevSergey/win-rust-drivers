use win::{AutoDeleteDevice, DRIVER_OBJECT, GUID, NTSTATUS, create_device_secure};

const CDO_CLASS: GUID = GUID {
    Data1: 0xf81d4fae,
    Data2: 0x7dec,
    Data3: 0x41d0,
    Data4: [0xa7, 0x65, 00, 0xa0, 0xc9, 0x1e, 0x6b, 0xf6],
};

pub fn create_device(
    driver_object: &mut DRIVER_OBJECT,
) -> core::result::Result<AutoDeleteDevice, NTSTATUS> {
    create_device_secure(
        driver_object,
        0,
        None, //device_name: core::option::Option<&mut UNICODE_STRING>,
        win::FILE_DEVICE_UNKNOWN,
        win::FILE_DEVICE_SECURE_OPEN,
        false,
        unsafe { &win::SDDL_DEVOBJ_SYS_ALL_ADM_ALL },
        Some(&CDO_CLASS),
    )
}
