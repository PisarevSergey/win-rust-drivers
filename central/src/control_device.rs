use win::{
    AutoDeleteDevice, DRIVER_OBJECT, GUID, NTSTATUS, RtlInitUnicodeString, UNICODE_STRING,
    create_device_secure,
};

const CDO_CLASS: GUID = GUID {
    Data1: 0xf81d4fae,
    Data2: 0x7dec,
    Data3: 0x41d0,
    Data4: [0xa7, 0x65, 00, 0xa0, 0xc9, 0x1e, 0x6b, 0xf6],
};

const CDO_NAME: &[u16] = wchar::wchz!(r"\Device\central_cdo");

pub fn create_device(
    driver_object: &mut DRIVER_OBJECT,
) -> core::result::Result<AutoDeleteDevice, NTSTATUS> {
    let mut cdo_name = UNICODE_STRING::default();
    unsafe { RtlInitUnicodeString(&raw mut cdo_name, CDO_NAME.as_ptr()) };

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
