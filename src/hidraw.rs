//! hidraw I/O: feature-report ioctls and the two read-only queries.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::drivers::{compx, keychron, Reading};

const IOC_READ_WRITE: u64 = 3;

fn ioc(nr: u8, len: usize) -> u64 {
    (IOC_READ_WRITE << 30) | ((len as u64) << 16) | ((b'H' as u64) << 8) | nr as u64
}

/// `HIDIOCSFEATURE(len)`
pub fn hidioc_sfeature(len: usize) -> u64 {
    ioc(0x06, len)
}

/// `HIDIOCGFEATURE(len)`
pub fn hidioc_gfeature(len: usize) -> u64 {
    ioc(0x07, len)
}

fn ioctl_buf(f: &File, req: u64, buf: *mut u8) -> io::Result<()> {
    // `as _`: the request type is c_ulong on glibc and c_int on musl.
    let rc = unsafe { libc::ioctl(f.as_raw_fd(), req as _, buf) };
    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn open(devnode: &Path, write: bool) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(write)
        .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(devnode)
}

/// Waits up to `ms` for `f` to become readable. Ok(false) on timeout.
pub fn wait_readable(f: &File, ms: i32) -> io::Result<bool> {
    let mut p = libc::pollfd {
        fd: f.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let rc = unsafe { libc::poll(&mut p, 1, ms) };
    match rc {
        -1 => Err(io::Error::last_os_error()),
        0 => Ok(false),
        _ if p.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 => {
            Err(io::Error::from_raw_os_error(libc::ENODEV))
        }
        _ => Ok(true),
    }
}

/// Compx status query. Ok(None) when the receiver answered but had no valid reading.
pub fn compx_query(devnode: &Path, seq: u8) -> io::Result<Option<(Reading, String)>> {
    let f = open(devnode, true)?;
    let mut req = compx::request(seq);
    ioctl_buf(&f, hidioc_sfeature(compx::REPORT_LEN), req.as_mut_ptr())?;
    for _ in 0..2 {
        sleep(Duration::from_millis(30));
        let mut rx = [0u8; compx::REPORT_LEN];
        rx[0] = compx::REPORT_ID;
        ioctl_buf(&f, hidioc_gfeature(compx::REPORT_LEN), rx.as_mut_ptr())?;
        if let Some(r) = compx::parse(&rx) {
            return Ok(Some(r));
        }
    }
    Ok(None)
}

/// Keychron Link slot query on interface 3: (paired mouse PID, connected).
pub fn keychron_slots(iface3: &Path) -> io::Result<Option<(u16, bool)>> {
    let mut f = open(iface3, true)?;
    let mut buf = [0u8; 64];
    while f.read(&mut buf).is_ok() {} // drop stale reports
    let mut req = [0u8; 33];
    req[1] = keychron::SLOT_QUERY;
    f.write_all(&req)?;
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        let left = deadline
            .saturating_duration_since(Instant::now())
            .as_millis() as i32;
        if left == 0 || !wait_readable(&f, left)? {
            return Ok(None);
        }
        match f.read(&mut buf) {
            Ok(n) if n > 0 && buf[0] == keychron::SLOT_QUERY => {
                return Ok(keychron::parse_slots(&buf[..n]))
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ioctl_numbers() {
        assert_eq!(hidioc_sfeature(33), 0xC021_4806);
        assert_eq!(hidioc_gfeature(33), 0xC021_4807);
        assert_eq!(hidioc_gfeature(21), 0xC015_4807);
    }
}
