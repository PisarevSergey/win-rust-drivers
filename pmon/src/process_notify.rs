use win::{
    DbgPrint, FALSE, HANDLE, NTSTATUS, PCSTR, PEPROCESS, PPS_CREATE_NOTIFY_INFO,
    PsSetCreateProcessNotifyRoutineEx, STATUS_SUCCESS, TRUE, kd_print_ln,
};

struct ProcessCreateNotify {}

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
    _pid: HANDLE,
    _create_info: PPS_CREATE_NOTIFY_INFO,
) {
    kd_print_ln!(
        "process_notify: pid - %p; create info - %p",
        _pid,
        _create_info
    );
}
