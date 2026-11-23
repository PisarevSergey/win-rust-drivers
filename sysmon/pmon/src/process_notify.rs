use core::slice;

use win::{
    FALSE, HANDLE, NTSTATUS, PEPROCESS, PPS_CREATE_NOTIFY_INFO, PsSetCreateProcessNotifyRoutineEx,
    STATUS_SUCCESS, TRUE, kd_print_ln,
};

use crate::driver;

use sysmon_common::{ProcessEventType, ProcessNotify, ProcessStartInfo};

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
    _process: PEPROCESS,
    pid: HANDLE,
    create_info: PPS_CREATE_NOTIFY_INFO,
) {
    kd_print_ln!(
        "pmon::process_notify: pid - %p; create info - %p",
        pid,
        create_info
    );

    if create_info.is_null() {
        let pn = ProcessNotify {
            pid: pid,
            event: ProcessEventType::Exit,
        };

        driver::get_driver().send_process_event_to_central(&pn);
        return;
    };

    let create_info = unsafe { &*create_info };
    let img_path = if create_info.ImageFileName.is_null() {
        None
    } else {
        let img = unsafe { &*create_info.ImageFileName };
        Some(unsafe { slice::from_raw_parts(img.Buffer, (img.Length / 2) as usize) })
    };

    let command_line = if create_info.CommandLine.is_null() {
        None
    } else {
        let command_line = unsafe { &*create_info.CommandLine };
        Some(unsafe {
            slice::from_raw_parts(command_line.Buffer, (command_line.Length / 2) as usize)
        })
    };

    let si = ProcessStartInfo {
        ppid: create_info.ParentProcessId,
        image_path: img_path,
        command_line: command_line,
    };
    let pn = ProcessNotify {
        pid: pid,
        event: ProcessEventType::Start(&si),
    };

    driver::get_driver().send_process_event_to_central(&pn);
}
