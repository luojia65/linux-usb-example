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
    type Item = io::Result<Hub<'iter>>;

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
            let path = unsafe { libc::strdup(&entry.as_ref().d_name as *const _) };
            let path = match NonNull::new(path) {
                Some(path) => path,
                None => panic!("strdup returned null, this is a bug")
            };
            return Some(Ok(Hub { path, _lifetime_of_path: PhantomData }))
        }
        None
    }
}

pub struct Hub<'hub> {
    path: NonNull<libc::c_char>,
    _lifetime_of_path: PhantomData<&'hub ()>
}

impl Drop for Hub<'_> {
    fn drop(&mut self) {
        unsafe { libc::free(self.path.as_ptr() as *mut _) }
    }
}

impl<'hub> Hub<'hub> {
    fn path(&self) -> &'hub CStr {
        unsafe { 
            CStr::from_ptr(self.path.as_ptr())
        }
    }
}

fn main() -> io::Result<()> {
    // sysfs usb lookup
    let mut vec = Vec::new();
    for hub in hubs()? {
        vec.push(hub);
        // println!("{}", hub.name().to_str().unwrap())
    }
    for hub in vec {
        println!("{}", hub?.path().to_str().unwrap())
    }
    Ok(())
}
