//! Hotplug notifications. Listens to udev's netlink group (not the raw kernel group), so by the
//! time we hear about a new hidraw node udev has already applied the `uaccess` ACL.

use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

const UDEV_GROUP: u32 = 2;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Changes {
    pub hidraw: bool,
    pub power_supply: bool,
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}

/// What a single uevent message is about.
pub fn classify(msg: &[u8]) -> Changes {
    Changes {
        hidraw: contains(msg, b"SUBSYSTEM=hidraw\0"),
        power_supply: contains(msg, b"SUBSYSTEM=power_supply\0"),
    }
}

pub fn open() -> io::Result<OwnedFd> {
    unsafe {
        let fd = libc::socket(
            libc::AF_NETLINK,
            libc::SOCK_DGRAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
            libc::NETLINK_KOBJECT_UEVENT,
        );
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let fd = OwnedFd::from_raw_fd(fd);
        let mut addr: libc::sockaddr_nl = std::mem::zeroed();
        addr.nl_family = libc::AF_NETLINK as libc::sa_family_t;
        addr.nl_groups = UDEV_GROUP;
        let rc = libc::bind(
            fd.as_raw_fd(),
            &addr as *const libc::sockaddr_nl as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t,
        );
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(fd)
    }
}

/// Reads every pending message.
pub fn drain(fd: &OwnedFd) -> Changes {
    let mut out = Changes::default();
    let mut buf = [0u8; 8192];
    loop {
        let n = unsafe { libc::recv(fd.as_raw_fd(), buf.as_mut_ptr().cast(), buf.len(), 0) };
        if n <= 0 {
            return out;
        }
        let c = classify(&buf[..n as usize]);
        out.hidraw |= c.hidraw;
        out.power_supply |= c.power_supply;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_events() {
        let m = b"libudev\0....ACTION=add\0DEVPATH=/devices/x/hidraw/hidraw11\0SUBSYSTEM=hidraw\0DEVNAME=/dev/hidraw11\0";
        assert_eq!(
            classify(m),
            Changes {
                hidraw: true,
                power_supply: false
            }
        );
        let m = b"ACTION=change\0SUBSYSTEM=power_supply\0POWER_SUPPLY_CAPACITY=50\0";
        assert_eq!(
            classify(m),
            Changes {
                hidraw: false,
                power_supply: true
            }
        );
        let m = b"ACTION=add\0SUBSYSTEM=input\0";
        assert_eq!(classify(m), Changes::default());
        // Prefix match must not fire on longer subsystem names.
        assert_eq!(classify(b"SUBSYSTEM=hidrawx\0"), Changes::default());
    }
}
