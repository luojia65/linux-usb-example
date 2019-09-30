use core::ptr::{self, NonNull};
use core::marker::PhantomData;
use core::mem;
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

pub fn devices<'iter>() -> io::Result<Devices<'iter>> {
    let dir = NonNull::new(
        unsafe { libc::opendir(SYSFS_ROOT.as_ptr() as *const _) }
    );
    if let Some(dir) = dir {
        Ok(Devices { dir, _lifetime_of_dir: PhantomData })
    } else {
        Err(io::Error::last_os_error())
    }
}

pub struct Devices<'iter> {
    dir: ptr::NonNull<libc::DIR>,
    _lifetime_of_dir: PhantomData<&'iter ()>
}

impl Drop for Devices<'_> {
    fn drop(&mut self) {
        unsafe { libc::closedir(self.dir.as_ptr()) };
    }
}

impl<'iter> Iterator for Devices<'iter> {
    type Item = io::Result<Device<'iter>>;

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
            let is_hub = unsafe { libc::strncmp(
                USB_PREFIX.as_ptr() as *const _,
                &entry.as_ref().d_name as *const _,
                3
            ) } == 0;
            let is_device = unsafe { libc::strchr( 
                &entry.as_ref().d_name as *const _,
                b':' as _
            ) } != core::ptr::null_mut(); 
            if !is_hub && !is_device {
                continue;
            }
            let path = unsafe { libc::strdup(&entry.as_ref().d_name as *const _) };
            let path = match NonNull::new(path) {
                Some(path) => path,
                None => return Some(Err(io::Error::last_os_error()))
            };
            return Some(Ok(Device { path, _lifetime_of_path: PhantomData }))
        }
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
    // active interface descriptor
    pub fn interface_descriptor(&self) -> io::Result<InterfaceDescriptor> {
        let dst = mem::MaybeUninit::<InterfaceDescriptor>::uninit();
        let fd = unsafe { libc::open(
            self.path.as_ptr() as *const _,
            libc::O_RDONLY,
        ) };
        if fd == 0 {
            return Err(io::Error::last_os_error())
        }
        unsafe { libc::read(fd, dst.as_ptr() as _, mem::size_of::<InterfaceDescriptor>()) };
        Ok(unsafe { dst.assume_init() })
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[repr(C)]
pub struct DeviceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub bcd_usb: u16,
    pub device_class: u8,
    pub device_sub_class: u8,
    pub device_protocol: u8,
    pub max_packet_size_0: u8,
    pub id_vendor: u16,
    pub id_product: u16,
    pub bcd_device: u16,
    pub manufacturer: u8,
    pub product: u8,
    pub serial_number: u8,
    pub num_configurations: u8,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[repr(C)]
pub struct InterfaceDescriptor {
    pub length: u8,
    pub descriptor_type: u8, 
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub index_interface: u8,
}

fn main() -> io::Result<()> {
    // sysfs usb lookup
    let mut all_devs = Vec::new();
    for dev in devices()? {
        let dev = dev?;
        all_devs.push(dev);
    }
    for dev in all_devs {
        println!("{:?}", dev.interface_descriptor())
    }
    Ok(())
}
