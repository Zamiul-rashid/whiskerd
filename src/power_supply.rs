//! Peripheral batteries the kernel already reads (`/sys/class/power_supply`, scope "Device"):
//! Logitech via hid-logitech-hidpp, HID devices that report battery strength, and others.

use std::fs;
use std::path::Path;

use crate::state::DeviceState;

fn read(p: &Path) -> Option<String> {
    fs::read_to_string(p).ok().map(|s| s.trim().to_string())
}

fn kind_of(supply: &Path) -> &'static str {
    // An input device under the same parent with relative axes is a mouse.
    let Ok(rd) = fs::read_dir(supply.join("device/input")) else {
        return "device";
    };
    for e in rd.flatten() {
        if let Some(rel) = read(&e.path().join("capabilities/rel")) {
            if rel != "0" {
                return "mouse";
            }
        }
        if let Some(key) = read(&e.path().join("capabilities/key")) {
            if key.split(' ').count() >= 4 {
                return "keyboard";
            }
        }
    }
    "device"
}

/// Current peripheral batteries, sorted by id.
pub fn scan(class_dir: &Path, now: u64) -> Vec<DeviceState> {
    let Ok(rd) = fs::read_dir(class_dir) else {
        return Vec::new();
    };
    let mut out: Vec<DeviceState> = rd
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if read(&p.join("scope"))?.as_str() != "Device"
                || read(&p.join("type"))?.as_str() != "Battery"
            {
                return None;
            }
            let entry = e.file_name().to_string_lossy().into_owned();
            let name = read(&p.join("model_name"))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| entry.clone());
            let mut d = DeviceState::new(format!("ps:{entry}"), name, kind_of(&p));
            d.connected = read(&p.join("present")).map_or(true, |v| v == "1");
            d.battery = read(&p.join("capacity"))
                .and_then(|c| c.parse::<u8>().ok())
                .filter(|&c| c <= 100);
            d.charging = match read(&p.join("status")).as_deref() {
                Some("Charging") | Some("Full") => Some(true),
                Some("Discharging") | Some("Not charging") => Some(false),
                _ => None,
            };
            d.updated = d.battery.map(|_| now);
            Some(d)
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn supply(root: &Path, name: &str, files: &[(&str, &str)]) {
        let d = root.join(name);
        fs::create_dir_all(&d).unwrap();
        for (f, v) in files {
            if let Some(parent) = Path::new(f).parent() {
                fs::create_dir_all(d.join(parent)).unwrap();
            }
            fs::write(d.join(f), format!("{v}\n")).unwrap();
        }
    }

    #[test]
    fn reads_peripheral_batteries_only() {
        let root: PathBuf =
            std::env::temp_dir().join(format!("whiskerd-ps-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        supply(
            &root,
            "BAT1",
            &[("scope", "System"), ("type", "Battery"), ("capacity", "76")],
        );
        supply(&root, "ACAD", &[("type", "Mains")]);
        supply(
            &root,
            "hidpp_battery_0",
            &[
                ("scope", "Device"),
                ("type", "Battery"),
                ("capacity", "55"),
                ("status", "Discharging"),
                ("model_name", "MX Master 3"),
                ("device/input/input30/capabilities/rel", "1943"),
            ],
        );
        let devs = scan(&root, 100);
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(devs.len(), 1);
        let d = &devs[0];
        assert_eq!(
            (d.id.as_str(), d.name.as_str(), d.kind),
            ("ps:hidpp_battery_0", "MX Master 3", "mouse")
        );
        assert_eq!(
            (d.battery, d.charging, d.connected, d.updated),
            (Some(55), Some(false), true, Some(100))
        );
    }
}
