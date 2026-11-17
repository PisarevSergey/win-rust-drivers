use win::{
    AutoDeleteDevice, DRIVER_OBJECT, DbgPrint, IO_NO_INCREMENT, IRP, IRP_MJ_CLEANUP, IRP_MJ_CLOSE,
    IRP_MJ_CREATE, IRP_MJ_DEVICE_CONTROL, IofCompleteRequest, NTSTATUS, PCSTR, PDEVICE_OBJECT,
    PIRP, STATUS_SUCCESS, ULONG_PTR, get_current_stack_location, get_irp_status, kd_print_ln,
};

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

        driver.MajorFunction[IRP_MJ_CREATE as usize] = Some(on_create);
        driver.MajorFunction[IRP_MJ_CLEANUP as usize] = Some(on_cleanup);
        driver.MajorFunction[IRP_MJ_CLOSE as usize] = Some(on_close);
        driver.MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(on_ioctl);

        kd_print_ln!("driver create success");
        Ok(Driver {
            cdo: Some(cdo),
            symlink: Some(symlink),
        })
    }
}

unsafe extern "C" fn on_ioctl(_device_object: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    let irp = unsafe { &mut *irp };

    let current_sl = get_current_stack_location(irp);
    let code = unsafe { current_sl.Parameters.DeviceIoControl.IoControlCode };

    kd_print_ln!("ioctl 0x%x for device %p", code, _device_object);

    complete_irp_success(irp)
}

unsafe extern "C" fn on_create(_device_object: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    kd_print_ln!("create for device %p", _device_object);

    let irp = unsafe { &mut *irp };
    complete_irp_success(irp)
}

unsafe extern "C" fn on_cleanup(_device_object: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    kd_print_ln!("cleanup for device %p", _device_object);

    let irp = unsafe { &mut *irp };
    complete_irp_success(irp)
}

unsafe extern "C" fn on_close(_device_object: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    kd_print_ln!("close for device %p", _device_object);

    let irp = unsafe { &mut *irp };
    complete_irp_success(irp)
}

fn complete_irp_success(irp: &mut IRP) -> NTSTATUS {
    complete_irp_with_params(irp, STATUS_SUCCESS, 0)
}

fn complete_irp_with_params(irp: &mut IRP, status: NTSTATUS, info: ULONG_PTR) -> NTSTATUS {
    *get_irp_status(irp) = status;
    irp.IoStatus.Information = info;

    unsafe { IofCompleteRequest(irp, IO_NO_INCREMENT as i8) };

    status
}
