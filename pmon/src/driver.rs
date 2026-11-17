use crate::process_notify::ProcessCreateNotify;
use win::NTSTATUS;

pub struct Driver {
    #[allow(dead_code)]
    proc_create_notify: core::option::Option<ProcessCreateNotify>,
}

impl Driver {
    pub const fn new() -> Self {
        Driver {
            proc_create_notify: None,
        }
    }

    pub fn try_create() -> core::result::Result<Self, NTSTATUS> {
        let pn = match ProcessCreateNotify::try_create() {
            Ok(pn) => pn,
            Err(status) => return Err(status),
        };

        Ok(Self {
            proc_create_notify: Some(pn),
        })
    }
}
