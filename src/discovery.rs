//! Finds supported receivers by walking `/sys/class/hidraw`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::devices::{self, Receiver};

#[derive(Debug, PartialEq, Eq)]
pub struct Found {
    pub rx: &'static Receiver,
    /// `/dev/hidrawN` of the vendor interface.
    pub devnode: PathBuf,
    /// Canonical sysfs dir of the USB device (shared by all its interfaces).
    pub usb_dev: PathBuf,
}

impl Found {
    /// Stable id: VID:PID plus the USB port path, e.g. `3434:d031@3-5.3`.
    pub fn id(&self) -> String {
        let port = self
            .usb_dev
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        format!("{:04x}:{:04x}@{}", self.rx.vid, self.rx.pid, port)
    }
}

struct Node {
    devnode: PathBuf,
    vid: u16,
    pid: u16,
    iface: u8,
    usb_dev: PathBuf,
}

fn read_hid_id(uevent: &Path) -> Option<(u16, u16)> {
    let text = fs::read_to_string(uevent).ok()?;
    let id = text.lines().find_map(|l| l.strip_prefix("HID_ID="))?;
    let mut parts = id.split(':').map(|p| u32::from_str_radix(p, 16).ok());
    let (bus, vid, pid) = (parts.next()??, parts.next()??, parts.next()??);
    (bus == 3).then_some((vid as u16, pid as u16)) // 3 = BUS_USB
}

fn nodes(class_dir: &Path, dev_dir: &Path) -> Vec<Node> {
    let Ok(rd) = fs::read_dir(class_dir) else {
        return Vec::new();
    };
    let mut out: Vec<Node> = rd
        .flatten()
        .filter_map(|e| {
            let hid = e.path().join("device");
            let (vid, pid) = read_hid_id(&hid.join("uevent"))?;
            let iface_txt = fs::read_to_string(hid.join("../bInterfaceNumber")).ok()?;
            let iface = u8::from_str_radix(iface_txt.trim(), 16).ok()?;
            let usb_dev = fs::canonicalize(hid.join("../..")).ok()?;
            Some(Node {
                devnode: dev_dir.join(e.file_name()),
                vid,
                pid,
                iface,
                usb_dev,
            })
        })
        .collect();
    out.sort_by(|a, b| a.devnode.cmp(&b.devnode));
    out
}

/// All supported receivers currently plugged in, sorted by devnode.
pub fn scan(class_dir: &Path, dev_dir: &Path) -> Vec<Found> {
    nodes(class_dir, dev_dir)
        .into_iter()
        .filter_map(|n| {
            let rx = devices::find(n.vid, n.pid, n.iface)?;
            Some(Found {
                rx,
                devnode: n.devnode,
                usb_dev: n.usb_dev,
            })
        })
        .collect()
}

/// The hidraw node of another interface on the same USB device.
pub fn sibling_iface(
    found: &Found,
    class_dir: &Path,
    dev_dir: &Path,
    iface: u8,
) -> Option<PathBuf> {
    nodes(class_dir, dev_dir)
        .into_iter()
        .find(|n| n.usb_dev == found.usb_dev && n.iface == iface)
        .map(|n| n.devnode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    struct Tree {
        root: PathBuf,
    }

    impl Tree {
        fn new(tag: &str) -> Tree {
            let root =
                std::env::temp_dir().join(format!("whiskerd-sysfs-{tag}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(root.join("class")).unwrap();
            Tree { root }
        }
        /// Adds hidraw<n> for `vid:pid` interface `iface` on USB device `port`.
        fn add(&self, n: u32, port: &str, iface: u8, vid: u16, pid: u16) {
            let intf = self
                .root
                .join(format!("devices/usb/{port}/{port}:1.{iface}"));
            let hid = intf.join(format!("0003:{vid:04X}:{pid:04X}.{n:04X}"));
            fs::create_dir_all(&hid).unwrap();
            fs::write(intf.join("bInterfaceNumber"), format!("{iface:02x}\n")).unwrap();
            fs::write(
                hid.join("uevent"),
                format!(
                    "DRIVER=hid-generic\nHID_ID=0003:0000{vid:04X}:0000{pid:04X}\nHID_NAME=x\n"
                ),
            )
            .unwrap();
            let class = self.root.join(format!("class/hidraw{n}"));
            fs::create_dir_all(&class).unwrap();
            symlink(&hid, class.join("device")).unwrap();
        }
        fn class(&self) -> PathBuf {
            self.root.join("class")
        }
    }

    impl Drop for Tree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn real_layout(t: &Tree) {
        for (n, i) in [(10, 0), (11, 1), (12, 2), (13, 3)] {
            t.add(n, "3-5.3", i, 0x3434, 0xd031);
        }
        for (n, i) in [(6, 0), (7, 1), (8, 2)] {
            t.add(n, "3-1", i, 0x248a, 0x5b2f);
        }
        t.add(0, "3-2", 0, 0xfffe, 0x0011);
        t.add(1, "3-2", 1, 0xfffe, 0x0011);
    }

    #[test]
    fn finds_both_receivers() {
        let t = Tree::new("both");
        real_layout(&t);
        let found = scan(&t.class(), Path::new("/dev"));
        let nodes: Vec<_> = found.iter().map(|f| f.devnode.clone()).collect();
        assert_eq!(
            nodes,
            vec![
                PathBuf::from("/dev/hidraw11"),
                PathBuf::from("/dev/hidraw7")
            ]
        );
        assert_eq!(found[0].rx.pid, 0xd031);
        assert_eq!(found[0].id(), "3434:d031@3-5.3");
        assert_eq!(found[1].id(), "248a:5b2f@3-1");
    }

    #[test]
    fn ignores_other_vendor() {
        let t = Tree::new("other");
        t.add(0, "3-2", 1, 0xfffe, 0x0011);
        assert!(scan(&t.class(), Path::new("/dev")).is_empty());
    }

    #[test]
    fn sibling_finds_iface3() {
        let t = Tree::new("sib");
        real_layout(&t);
        let found = scan(&t.class(), Path::new("/dev"));
        let kc = found.iter().find(|f| f.rx.vid == 0x3434).unwrap();
        assert_eq!(
            sibling_iface(kc, &t.class(), Path::new("/dev"), 3),
            Some(PathBuf::from("/dev/hidraw13"))
        );
        assert_eq!(sibling_iface(kc, &t.class(), Path::new("/dev"), 9), None);
    }

    #[test]
    fn missing_class_dir_is_empty() {
        assert!(scan(Path::new("/nonexistent/whiskerd"), Path::new("/dev")).is_empty());
    }
}
