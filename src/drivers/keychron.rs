//! Keychron 1K "Link" receiver (`3434:d030` Type-A, `3434:d031` Type-C) with the older Keychron
//! mice (M1, M2, M3, M6, M7 ...).
//!
//! The receiver does not answer battery requests (feature reads stall). Instead it pushes input
//! report `0x54` on interface 1 when the mouse connects or wakes:
//! `54 e2 01 <link> 00 <battery %> 02 02 00 ...`.
//! Interface 3 (raw HID, unnumbered 32-byte reports) answers `b2` with the paired slots:
//! `b2 <n> [<vid lo> <vid hi> <pid lo> <pid hi> <connected>] * n`.

use super::Reading;

pub const EVENT_REPORT: u8 = 0x54;
const EV_STATUS: u8 = 0xe2;
pub const SLOT_QUERY: u8 = 0xb2;
const KEYCHRON_VID: u16 = 0x3434;

/// Parses a report read from interface 1 (`report[0]` is the report id).
pub fn parse_event(r: &[u8]) -> Option<Reading> {
    if r.len() < 8 || r[0] != EVENT_REPORT || r[1] != EV_STATUS || r[6] != 2 || r[7] != 2 {
        return None;
    }
    (r[5] <= 100).then_some(Reading {
        battery: r[5],
        charging: None,
    })
}

/// Parses the interface 3 slot reply: (paired mouse PID, connected) of the first Keychron slot.
pub fn parse_slots(r: &[u8]) -> Option<(u16, bool)> {
    if r.len() < 7 || r[0] != SLOT_QUERY {
        return None;
    }
    (0..r[1] as usize)
        .map(|i| 2 + 5 * i)
        .take_while(|&o| o + 4 < r.len())
        .find_map(|o| {
            let vid = u16::from_le_bytes([r[o], r[o + 1]]);
            (vid == KEYCHRON_VID).then(|| (u16::from_le_bytes([r[o + 2], r[o + 3]]), r[o + 4] != 0))
        })
}

/// Product name for the paired mouse's PID.
pub fn model_name(pid: u16) -> &'static str {
    match pid {
        0xd03f => "Keychron M6",
        0xd032 | 0xd033 => "Keychron M3",
        0xd035 => "Keychron M1",
        0xd03b => "Keychron M2",
        0xd03d => "Keychron M2 mini",
        0xd036 | 0xd03e => "Keychron M3 mini",
        0xd044 => "Keychron M7",
        _ => "Keychron mouse",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::hex;

    #[test]
    fn parses_events() {
        let r = parse_event(&hex(
            "54 e2 01 01 00 4c 02 02 00 00 00 00 00 00 00 00 00 00 00 00 00",
        ));
        assert_eq!(
            r,
            Some(Reading {
                battery: 76,
                charging: None
            })
        );
        let r = parse_event(&hex(
            "54 e2 01 02 00 4e 02 02 00 00 00 00 00 00 00 00 00 00 00 00 00",
        ));
        assert_eq!(r.unwrap().battery, 78);
    }

    #[test]
    fn ignores_other_reports() {
        for s in [
            "54 e2 01 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00",
            "54 e4 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00",
            "54 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00",
            "03 00 01 00 fa ff 00 00",
            "54 e2 01 01 00 65 02 02 00 00 00 00 00 00 00 00 00 00 00 00 00",
            "54 e2 01",
        ] {
            assert_eq!(parse_event(&hex(s)), None, "{s}");
        }
    }

    #[test]
    fn parses_slots() {
        assert_eq!(
            parse_slots(&hex("b2 01 34 34 3f d0 01 00")),
            Some((0xd03f, true))
        );
        assert_eq!(
            parse_slots(&hex("b2 01 34 34 3f d0 00 00")),
            Some((0xd03f, false))
        );
        assert_eq!(parse_slots(&hex("b2 00 00 00")), None);
        assert_eq!(parse_slots(&hex("b2 01 00 00 00 00 00 00")), None);
        assert_eq!(model_name(0xd03f), "Keychron M6");
        assert_eq!(model_name(0x1234), "Keychron mouse");
    }
}
