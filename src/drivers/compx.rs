//! "Compx" 2.4 GHz receiver family (`248a:5b2f` "SSCYPL Wireless-Receiver", its wired twin
//! `248a:5b2e`, ...), used by Ajazz, Delux, Genesis, Inphic and other brands.
//!
//! Vendor interface 1, feature report `0x0C` (33 bytes with the report id). Command `0x20` asks
//! for the device status; it is read-only. Reply layout:
//! `rx[2]` = 0x20, `rx[6]` = payload length (0 = not ready), `rx[8..12]` = ASCII model tag,
//! `rx[0x12]` = battery %, `rx[0x13]` = 1 charging / 2 full / else discharging.

use super::Reading;

pub const REPORT_ID: u8 = 0x0c;
pub const REPORT_LEN: usize = 33;
const CMD_STATUS: u8 = 0x20;

/// The status request, ready for `HIDIOCSFEATURE`.
pub fn request(seq: u8) -> [u8; REPORT_LEN] {
    let mut b = [0u8; REPORT_LEN];
    b[..5].copy_from_slice(&[REPORT_ID, 0x01, CMD_STATUS, 0x00, seq]);
    b
}

/// Parses a `HIDIOCGFEATURE` reply into a reading and the model tag (e.g. `"M184"`).
pub fn parse(rx: &[u8]) -> Option<(Reading, String)> {
    if rx.len() < 0x14 || rx[0] != REPORT_ID || rx[2] != CMD_STATUS || rx[6] == 0 {
        return None;
    }
    let battery = rx[0x12];
    if battery > 100 {
        return None;
    }
    let tag = rx[8..12]
        .iter()
        .take_while(|&&c| c != 0)
        .filter(|c| c.is_ascii_graphic())
        .map(|&c| c as char)
        .collect();
    let charging = Some(matches!(rx[0x13], 1 | 2));
    Some((Reading { battery, charging }, tag))
}

/// Human name for a model tag. Unknown tags still get a usable name.
pub fn model_name(tag: &str) -> String {
    match tag {
        "M184" => "Ajazz AJ179P".into(), // the AJ179P reports firmware tag M184
        "M137" => "Genesis Zircon 500".into(),
        "" => "2.4 GHz mouse".into(),
        t => format!("Mouse {t}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::hex;

    // Live reply from an Ajazz AJ179P (firmware tag M184) on 248a:5b2f, 2026-11-20.
    const LIVE: &str = "0c 01 20 00 01 01 10 00 4d 31 38 34 33 01 00 95 11 ff 4d 00 \
                        ff 01 00 00 90 e5 31 38 ff c7 67 09 6d";

    #[test]
    fn request_layout() {
        let r = request(1);
        assert_eq!(r.len(), 33);
        assert_eq!(r[..7], [0x0c, 0x01, 0x20, 0x00, 0x01, 0x00, 0x00]);
        assert!(r[7..].iter().all(|&b| b == 0));
    }

    #[test]
    fn parses_live_reply() {
        let (r, tag) = parse(&hex(LIVE)).unwrap();
        assert_eq!(
            r,
            Reading {
                battery: 77,
                charging: Some(false)
            }
        );
        assert_eq!(tag, "M184");
        assert_eq!(model_name(&tag), "Ajazz AJ179P");
    }

    #[test]
    fn charging_states() {
        let mut rx = hex(LIVE);
        rx[0x13] = 1;
        assert_eq!(parse(&rx).unwrap().0.charging, Some(true));
        rx[0x13] = 2;
        assert_eq!(parse(&rx).unwrap().0.charging, Some(true));
    }

    #[test]
    fn rejects_bad_replies() {
        let good = hex(LIVE);
        let mut rx = good.clone();
        rx[2] = 0;
        assert_eq!(parse(&rx), None);
        let mut rx = good.clone();
        rx[6] = 0;
        assert_eq!(parse(&rx), None);
        let mut rx = good.clone();
        rx[0x12] = 101;
        assert_eq!(parse(&rx), None);
        assert_eq!(parse(&good[..0x13]), None);
    }

    #[test]
    fn unknown_tags() {
        assert_eq!(model_name("X999"), "Mouse X999");
        assert_eq!(model_name(""), "2.4 GHz mouse");
    }
}
