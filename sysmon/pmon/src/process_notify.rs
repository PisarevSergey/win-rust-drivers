use win::{
    kd_print_ln, PsSetCreateProcessNotifyRoutineEx, FALSE, HANDLE, NTSTATUS, PEPROCESS,
    PPS_CREATE_NOTIFY_INFO, STATUS_SUCCESS, TRUE,
};

use crate::driver;

use sysmon_common::ProcessNotify;

pub struct ProcessCreateNotify {}

impl ProcessCreateNotify {
    pub fn try_create() -> core::result::Result<Self, NTSTATUS> {
        let status =
            unsafe { PsSetCreateProcessNotifyRoutineEx(Some(process_notify), FALSE as u8) };
        if status != STATUS_SUCCESS {
            kd_print_ln!(
                "PsSetCreateProcessNotifyRoutineEx failed with status 0x%x",
                status
            );
            return Err(status);
        }

        kd_print_ln!("PsSetCreateProcessNotifyRoutineEx reg notify success");
        Ok(Self {})
    }
}

impl Drop for ProcessCreateNotify {
    fn drop(&mut self) {
        let _status =
            unsafe { PsSetCreateProcessNotifyRoutineEx(Some(process_notify), TRUE as u8) };
        kd_print_ln!(
            "PsSetCreateProcessNotifyRoutineEx unreg notify status 0x%x",
            _status
        );
    }
}

unsafe extern "C" fn process_notify(
    process: PEPROCESS,
    pid: HANDLE,
    create_info: PPS_CREATE_NOTIFY_INFO,
) {
    let mut pn = ProcessNotify {
        eprocess: process as u64,
        pid: pid as u64,
        ps_create_notify_info: create_info as u64,
    };

    kd_print_ln!(
        "process_notify: pid - 0x%llx; create info - 0x%llx",
        pn.pid,
        pn.ps_create_notify_info
    );

    driver::get_driver().send_process_event_tdo_central(&mut pn);
}
