#![no_std]

extern crate alloc;
use alloc::{alloc::alloc, boxed::Box};

use core::{
    alloc::{GlobalAlloc, Layout},
    mem::{align_of, size_of},
    option::Option,
    panic::PanicInfo,
};

use fltkernel::{
    _FLT_VOLUME_PROPERTIES, DbgPrint, FILE_DEVICE_DISK_FILE_SYSTEM, FLT_CONTEXT_END,
    FLT_CONTEXT_REGISTRATION, FLT_FILESYSTEM_TYPE, FLT_FILTER_UNLOAD_FLAGS, FLT_INSTANCE_CONTEXT,
    FLT_INSTANCE_SETUP_FLAGS, FLT_OPERATION_REGISTRATION, FLT_REGISTRATION,
    FLT_REGISTRATION_VERSION, FltGetVolumeProperties, FltRegisterFilter, FltStartFiltering,
    FltUnregisterFilter, IRP_MJ_CREATE, KeBugCheck, NTSTATUS, PAGE_SIZE, PCFLT_RELATED_OBJECTS,
    PDRIVER_OBJECT, PFLT_FILTER, PFLT_VOLUME, PUNICODE_STRING, UCHAR, ULONG, c_str, kd_print,
};

mod cbd_queue;
mod create;
mod instance_context;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_FLT_DO_NOT_ATTACH: NTSTATUS = 0xC0_1C_00_0F_u32.cast_signed();
const POOL_FLAG_NON_PAGED: u64 = 0x00000000_00000040_u64;
const POOL_FLAG_UNINITIALIZED: u64 = 0x00000000_00000002_u64;
const IRP_MJ_OPERATION_END: UCHAR = 0x80_u8;
const INSTANCE_CONTEXT_POOL_TAG: u32 = u32::from_be_bytes(*b"Cpic");

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { KeBugCheck(0xbadbad) }
}

struct PoolAllocator {
    tag: u32,
}

#[global_allocator]
static ALLOCATOR: PoolAllocator = PoolAllocator { tag: 0x65_70_72_43 }; // tag Crpe

unsafe impl Sync for PoolAllocator {}

unsafe impl GlobalAlloc for PoolAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let Ok(size) = u64::try_from(layout.size()) else {
            return core::ptr::null_mut();
        };

        unsafe {
            fltkernel::ExAllocatePool2(
                POOL_FLAG_NON_PAGED | POOL_FLAG_UNINITIALIZED,
                size,
                self.tag,
            ) as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { fltkernel::ExFreePoolWithTag(ptr as fltkernel::PVOID, self.tag) };
    }
}

static mut FILTER: PFLT_FILTER = core::ptr::null_mut();

fn try_create<T: Default>(size: Option<usize>) -> Option<Box<T>> {
    let alignment = align_of::<T>();
    let size = size.unwrap_or(size_of::<T>());
    let Ok(la) = Layout::from_size_align(size, alignment) else {
        kd_print!(
            c_str!("failed to create memory layout for size %llx and alignment %llx\n"),
            size,
            alignment
        );
        return None;
    };

    let p = unsafe { alloc(la) };
    if p.is_null() {
        kd_print!(
            c_str!("failed to allocate %llx bytes with alignment %llx\n"),
            size,
            alignment
        );
        return None;
    }
    unsafe { core::ptr::write(p as *mut T, T::default()) };

    Some(unsafe { Box::from_raw(p as *mut T) })
}

#[unsafe(no_mangle)]
extern "system" fn DriverEntry(driver: PDRIVER_OBJECT, reg_path: PUNICODE_STRING) -> NTSTATUS {
    kd_print!(
        c_str!("enter DriverEntry; driver object %p; reg path %wZ\n"),
        driver,
        reg_path
    );

    let operations = [
        FLT_OPERATION_REGISTRATION {
            MajorFunction: IRP_MJ_CREATE as UCHAR,
            PreOperation: Some(create::pre_create),
            ..FLT_OPERATION_REGISTRATION::default()
        },
        FLT_OPERATION_REGISTRATION {
            MajorFunction: IRP_MJ_OPERATION_END,
            ..FLT_OPERATION_REGISTRATION::default()
        },
    ];

    let ctx_reg = [
        FLT_CONTEXT_REGISTRATION {
            ContextType: FLT_INSTANCE_CONTEXT as u16,
            ContextCleanupCallback: Some(instance_context::instance_context_cleanup),
            Size: core::mem::size_of::<instance_context::InstanceContext>() as u64,
            PoolTag: INSTANCE_CONTEXT_POOL_TAG,
            ..FLT_CONTEXT_REGISTRATION::default()
        },
        FLT_CONTEXT_REGISTRATION {
            ContextType: FLT_CONTEXT_END as u16,
            ..FLT_CONTEXT_REGISTRATION::default()
        },
    ];

    let mut flt_reg = FLT_REGISTRATION::default();
    flt_reg.Size = core::mem::size_of_val(&flt_reg) as u16;
    flt_reg.Version = FLT_REGISTRATION_VERSION as u16;
    flt_reg.FilterUnloadCallback = Some(unload);
    flt_reg.InstanceSetupCallback = Some(attach);
    flt_reg.OperationRegistration = operations.as_ptr();
    flt_reg.ContextRegistration = ctx_reg.as_ptr();

    let status = unsafe { FltRegisterFilter(driver, &raw const flt_reg, &raw mut FILTER) };
    if status != 0 {
        kd_print!(
            c_str!("FltRegisterFilter failed with status: 0x%x\n"),
            status
        );

        return status;
    }
    kd_print!(c_str!("FltRegisterFilter success\n"));

    let status = unsafe { FltStartFiltering(FILTER) };
    if status != 0 {
        kd_print!(
            c_str!("FltStartFiltering failed with status 0x%x\n"),
            status
        );

        unsafe { FltUnregisterFilter(FILTER) };

        return status;
    }

    kd_print!(c_str!("filter start success\n"));
    STATUS_SUCCESS
}

extern "C" fn unload(_flags: FLT_FILTER_UNLOAD_FLAGS) -> NTSTATUS {
    kd_print!(c_str!("start unloading\n"));

    unsafe {
        FltUnregisterFilter(FILTER);
    }

    kd_print!(c_str!("unloading complete\n"));

    STATUS_SUCCESS
}

unsafe extern "C" fn attach(
    flt_objects: PCFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_SETUP_FLAGS,
    volume_device_type: ULONG,
    _volume_filesystem_type: FLT_FILESYSTEM_TYPE,
) -> NTSTATUS {
    let flt_objects = unsafe { &*flt_objects };

    unsafe { report_volume(flt_objects.Volume) };

    if volume_device_type != FILE_DEVICE_DISK_FILE_SYSTEM {
        kd_print!(c_str!("not a local disk, do not attach\n"));
        return STATUS_FLT_DO_NOT_ATTACH;
    }

    let filter = unsafe { &mut (*(flt_objects.Filter)) };
    let mut instance_ctx = match fltkernel::try_create_context::<instance_context::InstanceContext>(
        filter,
        FLT_INSTANCE_CONTEXT as u16,
        None,
    ) {
        Ok(ic) => ic,
        Err(status) => {
            kd_print!(
                c_str!("failed to create instance context with status 0x%x\n"),
                status
            );
            return status;
        }
    };

    if let Err(status) = instance_ctx.init(
        unsafe { &mut *flt_objects.Instance }) {
        kd_print!(
            c_str!("instance context init failed with status 0x%x\n"),
            status
        );
        return STATUS_FLT_DO_NOT_ATTACH;
    }

    if let Err(status) = fltkernel::set_instance_context_if_none(
        unsafe { &mut *flt_objects.Instance },
        &instance_ctx,
    ) {
        kd_print!(
            c_str!("failed to set instance context with status 0x%x\n"),
            status
        );
        return STATUS_FLT_DO_NOT_ATTACH;
    }

    kd_print!(c_str!("attaching to volume\n"));
    STATUS_SUCCESS
}

unsafe fn report_volume(volume: PFLT_VOLUME) {
    let Ok(volume_info_buffer_size) = usize::try_from(PAGE_SIZE) else {
        kd_print!(c_str!("failed to convert 0x%x to usize\n"), PAGE_SIZE);
        return;
    };

    let Some(mut vol_prop) = try_create::<_FLT_VOLUME_PROPERTIES>(Some(volume_info_buffer_size))
    else {
        kd_print!(c_str!("failed to allocate _FLT_VOLUME_PROPERTIES buffer\n"));
        return;
    };

    let mut length_ret: ULONG = 0;
    let status = unsafe {
        FltGetVolumeProperties(
            volume,
            Box::as_mut_ptr(&mut vol_prop),
            PAGE_SIZE,
            &raw mut length_ret,
        )
    };
    if status != STATUS_SUCCESS {
        kd_print!(
            c_str!("FltGetVolumeProperties failed with status 0x%x; length returned %d\n"),
            status,
            length_ret
        );

        return;
    };

    kd_print!(
        c_str!(
            "attach for volume FileSystemDriverName: %wZ; FileSystemDeviceName: %wZ; RealDeviceName: %wZ;\n"
        ),
        &raw const vol_prop.FileSystemDriverName,
        &raw const vol_prop.FileSystemDeviceName,
        &raw const vol_prop.RealDeviceName
    );
}
