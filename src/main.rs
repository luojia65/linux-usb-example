use core::ptr::{self, NonNull};
use core::marker::PhantomData;
use std::ffi::CStr;
use std::io;

const SYSFS_ROOT: &str = "/sys/bus/usb/devices\0";
const USB_PREFIX: &str = "usb";

pub fn hubs<'iter>() -> io::Result<Hubs<'iter>> {
    let dir = NonNull::new(
        unsafe { libc::opendir(SYSFS_ROOT.as_ptr() as *const _) }
    );
    if let Some(dir) = dir {
        Ok(Hubs { dir, _lifetime_of_dir: PhantomData })
    } else {
        Err(io::Error::last_os_error())
    }
}

pub struct Hubs<'iter> {
    dir: ptr::NonNull<libc::DIR>,
    _lifetime_of_dir: PhantomData<&'iter ()>
}

impl Drop for Hubs<'_> {
    fn drop(&mut self) {
        unsafe { libc::closedir(self.dir.as_ptr()) };
    }
}

impl<'iter> Iterator for Hubs<'iter> {
    type Item = Hub<'iter>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entry) = NonNull::new(
            unsafe { libc::readdir(self.dir.as_ptr()) }
        ) {
            if unsafe { libc::strncmp(
                USB_PREFIX.as_ptr() as *const _,
                &entry.as_ref().d_name as *const _,
                3
            ) } != 0 {
                continue;
            }
            return Some(Hub { entry, _lifetime_of_entry: PhantomData })
        }
        None
    }
}

pub struct Hub<'hub> {
    entry: NonNull<libc::dirent>,
    _lifetime_of_entry: PhantomData<&'hub ()>
}

impl<'hub> Hub<'hub> {
    pub fn name(&self) -> &'hub CStr {
        unsafe { 
            CStr::from_ptr(&self.entry.as_ref().d_name as *const _)
        }
    }
}

fn main() {
    // sysfs usb lookup
    for hub in hubs().unwrap() {
        println!("{}", hub.name().to_str().unwrap())
    }
}
