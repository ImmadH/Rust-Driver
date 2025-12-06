#![no_std]

#[cfg(not(test))]
extern crate wdk_panic;

#[cfg(not(test))]
use wdk_alloc::WdkAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WdkAllocator = WdkAllocator;

use core::{mem::zeroed, ptr::null_mut};

use wdk::println;
use wdk_sys::{
    NTSTATUS, PUNICODE_STRING, PDRIVER_OBJECT, PDEVICE_OBJECT, PIRP,
    UNICODE_STRING,
    PDRIVER_INITIALIZE,
    STATUS_SUCCESS,
    FILE_DEVICE_UNKNOWN,
    FILE_DEVICE_SECURE_OPEN,
    DO_BUFFERED_IO,
    DO_DEVICE_INITIALIZING,
    IRP_MJ_CREATE,
    IRP_MJ_CLOSE,
    IRP_MJ_DEVICE_CONTROL,
    IO_NO_INCREMENT,
};

use wdk_sys::ntddk::{
    RtlInitUnicodeString,
    IoCreateDevice,
    IoCreateSymbolicLink,
    IofCompleteRequest,
};

extern "system" {
    fn IoCreateDriver(
        DriverName: PUNICODE_STRING,
        InitializationFunction: PDRIVER_INITIALIZE,
    ) -> NTSTATUS;
}


static DEVICE_NAME: &[u16] = &[
    '\\' as u16, 'D' as u16, 'e' as u16, 'v' as u16,
    'i' as u16, 'c' as u16, 'e' as u16, '\\' as u16,
    'k' as u16, 'm' as u16, 0,
];

static SYMBOLIC_LINK_NAME: &[u16] = &[
    '\\' as u16, 'D' as u16, 'o' as u16, 's' as u16,
    'D' as u16, 'e' as u16, 'v' as u16, 'i' as u16,
    'c' as u16, 'e' as u16, 's' as u16, '\\' as u16,
    'k' as u16, 'm' as u16, 0,
];

static DRIVER_NAME: &[u16] = &[
    '\\' as u16, 'D' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, '\\' as u16,
    'r' as u16, 'u' as u16, 's' as u16, 't' as u16, '_' as u16, 'd' as u16, 'r' as u16, 'i' as u16,
    'v' as u16, 'e' as u16, 'r' as u16, 0,
];



unsafe fn finish_device_setup(
    driver_object: PDRIVER_OBJECT,
    device_object: PDEVICE_OBJECT,
) {
    (*device_object).Flags |= DO_BUFFERED_IO;

    (*driver_object).MajorFunction[IRP_MJ_CREATE as usize] =
        Some(driver_create);
    (*driver_object).MajorFunction[IRP_MJ_CLOSE as usize] =
        Some(driver_close);
    (*driver_object).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        Some(driver_device_control);

    (*device_object).Flags &= !DO_DEVICE_INITIALIZING;

    println!("[NAMWARE] Driver initialized successfully.");
}

//DRIVER HANDLERS
pub unsafe extern "C" fn driver_create(
    _device: PDEVICE_OBJECT,
    irp: PIRP,
) -> NTSTATUS {
    (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*irp).IoStatus.Information = 0;
    IofCompleteRequest(irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

pub unsafe extern "C" fn driver_close(
    _device: PDEVICE_OBJECT,
    irp: PIRP,
) -> NTSTATUS {
    (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*irp).IoStatus.Information = 0;
    IofCompleteRequest(irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

pub unsafe extern "C" fn driver_device_control(
    _device: PDEVICE_OBJECT,
    irp: PIRP,
) -> NTSTATUS {
    (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*irp).IoStatus.Information = 0;
    IofCompleteRequest(irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}



unsafe fn create_device(driver_object: PDRIVER_OBJECT) -> NTSTATUS {
    let mut device_name: UNICODE_STRING = zeroed();
    RtlInitUnicodeString(&mut device_name, DEVICE_NAME.as_ptr());

    let mut device_object: PDEVICE_OBJECT = null_mut();

    let status_dev: NTSTATUS = IoCreateDevice(
        driver_object,
        0,
        &mut device_name, 
        FILE_DEVICE_UNKNOWN,
        FILE_DEVICE_SECURE_OPEN,
        0,
        &mut device_object,
    );

    if status_dev != STATUS_SUCCESS  {
        println!("Failed to create device object");
        return status_dev;
    }

    //now create link
    let mut symbolic_link: UNICODE_STRING = zeroed();
    RtlInitUnicodeString(&mut symbolic_link, SYMBOLIC_LINK_NAME.as_ptr());

    let status_sym = IoCreateSymbolicLink(&mut symbolic_link, &mut device_name);
    if status_sym != STATUS_SUCCESS {
        println!("Failed to create device link");
        return status_sym;
    }

    //set device flag
    finish_device_setup(driver_object, device_object);


    STATUS_SUCCESS
}


// "real" initializer used by IoCreateDriver 
pub unsafe extern "C" fn driver_initialize(
    driver: PDRIVER_OBJECT,
    registry_path: PUNICODE_STRING,
) -> NTSTATUS {
    let _ = registry_path;

    println!("[NAMWARE] driver_initialize called");
    let status = create_device(driver);
    if status != STATUS_SUCCESS {
        println!("Failed Driver Initialization");
        return status;
    }

    STATUS_SUCCESS
}


// KdMapper will call this "entry point"
#[unsafe(export_name = "DriverEntry")]
pub unsafe extern "system" fn driver_entry(
    _driver: PDRIVER_OBJECT,
    _registry_path: PUNICODE_STRING,
) -> NTSTATUS {
    println!("[NAMWARE] Hello From The Kernal");

    let mut driver_name: UNICODE_STRING = zeroed();
    RtlInitUnicodeString(&mut driver_name, DRIVER_NAME.as_ptr());

    let status = IoCreateDriver(&mut driver_name, Some(driver_initialize));
    if status != STATUS_SUCCESS {
        println!("IoCreateDriver failed");
    }

    status
}
