use win::{AutoDeleteDevice, DRIVER_OBJECT, DbgPrint, NTSTATUS, PCSTR, kd_print_ln};

use crate::control_device::{SymLink, try_create_device};

use core::option::Option;

pub struct Driver {
    #[allow(dead_code)]
    cdo: Option<AutoDeleteDevice>,
    #[allow(dead_code)]
    symlink: Option<SymLink>,
}

impl Driver {
    pub const fn new() -> Self {
        Self {
            cdo: None,
            symlink: None,
        }
    }

    pub fn try_create(driver: &mut DRIVER_OBJECT) -> core::result::Result<Self, NTSTATUS> {
        let cdo = try_create_device(driver)?;
        let symlink = SymLink::try_create()?;

        kd_print_ln!("driver create success");
        Ok(Driver {
            cdo: Some(cdo),
            symlink: Some(symlink),
        })
    }
}
