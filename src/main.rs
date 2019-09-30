use core::ptr::{self, NonNull};
use core::marker::PhantomData;
use std::ffi::CStr;
use std::io;

const SYSFS_ROOT: &str = "/sys/bus/usb/devices\0";
const USB_PREFIX: &str = "usb";

#[inline]
unsafe fn set_errno(errno: libc::c_int) {
    *libc::__errno_location() = errno
}

#[inline]
unsafe fn get_errno() -> libc::c_int {
    *libc::__errno_location()
}

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
        loop {
            unsafe { set_errno(0) };
            let entry = match NonNull::new( unsafe { libc::readdir(self.dir.as_ptr()) }) {
                Some(entry) => entry,
                None => return if unsafe { get_errno() } == 0 {
                    None
                } else {
                    Some(Err(io::Error::last_os_error()))
                }
            };
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
                None => return Some(Err(io::Error::last_os_error()))
            };
            return Some(Ok(Hub { path, _lifetime_of_path: PhantomData }))
        }
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

    pub fn devices(&self) -> io::Result<Devices<'hub>> {
        let path = unsafe { libc::strdup(self.path.as_ptr()) };
        let path = match NonNull::new(path) {
            Some(path) => path,
            None => return Err(io::Error::last_os_error())
        };
        Ok(Devices { path, _lifetime_of_path: PhantomData })
    }
}

pub struct Devices<'iter> {
    path: NonNull<libc::c_char>,
    _lifetime_of_path: PhantomData<&'iter ()>
}

impl Drop for Devices<'_> {
    fn drop(&mut self) {
        unsafe { libc::free(self.path.as_ptr() as *mut _) }
    }
}

impl<'iter> Iterator for Devices<'iter> {
    type Item = io::Result<Device<'iter>>;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

pub struct Device<'device> {
    path: NonNull<libc::c_char>,
    _lifetime_of_path: PhantomData<&'device ()>
}

impl Drop for Device<'_> {
    fn drop(&mut self) {
        unsafe { libc::free(self.path.as_ptr() as *mut _) }
    }
}

impl<'device> Device<'device> {
    fn path(&self) -> &'device CStr {
        unsafe { 
            CStr::from_ptr(self.path.as_ptr())
        }
    }
}

fn main() -> io::Result<()> {
    // sysfs usb lookup
    let mut all_hubs = Vec::new();
    let mut all_devs = Vec::new();
    for hub in hubs()? {
        let hub = hub?;
        for dev in hub.devices()? {
            let dev = dev?;
            all_devs.push(dev);
        }
        all_hubs.push(hub);
    }
    for hub in all_hubs {
        println!("{:?}", hub.path())
    }
    for dev in all_devs {
        println!("{:?}", dev.path())
    }
    Ok(())
}
