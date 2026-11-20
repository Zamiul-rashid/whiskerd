//! What the daemon knows about each device, and the JSON file it publishes.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::drivers::Reading;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceState {
    pub id: String,
    pub name: String,
    /// "mouse", "keyboard" or "device".
    pub kind: &'static str,
    pub battery: Option<u8>,
    pub charging: Option<bool>,
    pub connected: bool,
    /// Unix time of the last reading.
    pub updated: Option<u64>,
}

impl DeviceState {
    pub fn new(id: String, name: String, kind: &'static str) -> Self {
        DeviceState {
            id,
            name,
            kind,
            battery: None,
            charging: None,
            connected: false,
            updated: None,
        }
    }

    pub fn apply(&mut self, r: Reading, now: u64) {
        self.battery = Some(r.battery);
        self.charging = r.charging;
        self.connected = true;
        self.updated = Some(now);
    }
}

fn json_str(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn json_opt<T: ToString>(v: Option<T>) -> String {
    v.map_or_else(|| "null".into(), |v| v.to_string())
}

pub fn render_json(devs: &[DeviceState]) -> String {
    let mut s = String::from("{\"devices\":[");
    for (i, d) in devs.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str("{\"id\":");
        json_str(&mut s, &d.id);
        s.push_str(",\"name\":");
        json_str(&mut s, &d.name);
        s.push_str(",\"kind\":");
        json_str(&mut s, d.kind);
        s.push_str(&format!(
            ",\"battery\":{},\"charging\":{},\"connected\":{},\"updated\":{}}}",
            json_opt(d.battery),
            json_opt(d.charging),
            d.connected,
            json_opt(d.updated)
        ));
    }
    s.push_str("]}\n");
    s
}

/// Writes `<path>.tmp` then renames it over `path`, so readers never see a partial file.
pub fn write_atomic(path: &Path, contents: &str) -> io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path)
}

pub fn runtime_path(xdg_runtime_dir: Option<&str>, uid: u32) -> PathBuf {
    match xdg_runtime_dir {
        Some(d) if !d.is_empty() => Path::new(d).join("whiskerd.json"),
        _ => PathBuf::from(format!("/run/user/{uid}/whiskerd.json")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_two_devices() {
        let mut a = DeviceState::new("248a:5b2f@3-1".into(), "Ajazz AJ179P".into(), "mouse");
        a.apply(
            Reading {
                battery: 77,
                charging: Some(false),
            },
            1795188749,
        );
        let k = DeviceState::new("3434:d031@3-5.3".into(), "Keychron mouse".into(), "mouse");
        assert_eq!(
            render_json(&[a, k]),
            "{\"devices\":[\
             {\"id\":\"248a:5b2f@3-1\",\"name\":\"Ajazz AJ179P\",\"kind\":\"mouse\",\"battery\":77,\"charging\":false,\"connected\":true,\"updated\":1795188749},\
             {\"id\":\"3434:d031@3-5.3\",\"name\":\"Keychron mouse\",\"kind\":\"mouse\",\"battery\":null,\"charging\":null,\"connected\":false,\"updated\":null}\
             ]}\n"
        );
    }

    #[test]
    fn render_escapes_name() {
        let d = DeviceState::new("x".into(), "a\"b\\c\n".into(), "device");
        assert!(render_json(&[d]).contains("\"name\":\"a\\\"b\\\\c\\u000a\""));
    }

    #[test]
    fn render_empty() {
        assert_eq!(render_json(&[]), "{\"devices\":[]}\n");
    }

    #[test]
    fn write_atomic_replaces() {
        let dir = std::env::temp_dir().join(format!("whiskerd-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("whiskerd.json");
        write_atomic(&p, "one").unwrap();
        write_atomic(&p, "two").unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "two");
        assert!(!dir.join("whiskerd.json.tmp").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn runtime_path_uses_xdg() {
        assert_eq!(
            runtime_path(Some("/run/user/1000"), 1000),
            PathBuf::from("/run/user/1000/whiskerd.json")
        );
    }

    #[test]
    fn runtime_path_falls_back() {
        assert_eq!(
            runtime_path(None, 1000),
            PathBuf::from("/run/user/1000/whiskerd.json")
        );
        assert_eq!(
            runtime_path(Some(""), 42),
            PathBuf::from("/run/user/42/whiskerd.json")
        );
    }
}
