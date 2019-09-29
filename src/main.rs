use core::ptr::NonNull;
use std::ffi::CStr;

const SYSFS_ROOT: &str = "/sys/bus/usb/devices\0";
const USB_PREFIX: &str = "usb";

fn main() {
    // sysfs usb lookup
    let dir = NonNull::new(
        unsafe { libc::opendir(SYSFS_ROOT.as_ptr() as *const _) }
    ).expect("Error occurred; use errno");
    while let Some(entry) = NonNull::new(
        unsafe { libc::readdir(dir.as_ptr()) }
    ) {
        if unsafe { libc::strncmp(
            USB_PREFIX.as_ptr() as *const _,
            &entry.as_ref().d_name as *const _,
            3
        ) } != 0 {
            continue;
        }
        let name = unsafe { 
            CStr::from_ptr(&entry.as_ref().d_name as *const _)
        };
        println!("{}", name.to_string_lossy());
    }
    unsafe { libc::closedir(dir.as_ptr()) };
}
