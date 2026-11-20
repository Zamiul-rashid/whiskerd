//! Peak-hold for voltage-based battery levels.
//!
//! Some mice (the Ajazz AJ179P among them) turn battery voltage straight into a percentage. Under
//! load the voltage sags, so the number slides while the mouse is in use and jumps back after a
//! rest. Showing the highest reading of the last 45 minutes keeps the level close to the resting
//! value. Only readings inside the window are kept, so memory stays bounded.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::drivers::Reading;

/// How far back the highest reading is taken from.
pub const WINDOW: Duration = Duration::from_secs(45 * 60);
/// Hard cap on stored readings. One per 5-minute poll is 9 or 10; 30-second retries after a
/// sleeping mouse wakes can add a few more.
const MAX_SAMPLES: usize = 16;

#[derive(Debug, Default)]
pub struct PeakHold {
    samples: VecDeque<(Instant, u8)>,
    charging: Option<bool>,
}

impl PeakHold {
    /// Records a reading taken at `at` and returns the reading to show.
    pub fn push(&mut self, r: Reading, at: Instant) -> Reading {
        // A change in charge state makes older levels meaningless.
        if r.charging != self.charging {
            self.samples.clear();
            self.charging = r.charging;
        }
        while self
            .samples
            .front()
            .is_some_and(|&(t, _)| at.saturating_duration_since(t) >= WINDOW)
        {
            self.samples.pop_front();
        }
        self.samples.push_back((at, r.battery));
        while self.samples.len() > MAX_SAMPLES {
            self.samples.pop_front();
        }
        if r.charging == Some(true) {
            return r; // the level climbs while charging: the latest is the truth
        }
        let peak = self
            .samples
            .iter()
            .map(|&(_, b)| b)
            .max()
            .unwrap_or(r.battery);
        Reading { battery: peak, ..r }
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.samples.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rd(battery: u8, charging: bool) -> Reading {
        Reading {
            battery,
            charging: Some(charging),
        }
    }

    const MIN: Duration = Duration::from_secs(60);

    #[test]
    fn shows_the_highest_reading_in_the_window() {
        let t0 = Instant::now();
        let mut p = PeakHold::default();
        assert_eq!(p.push(rd(77, false), t0).battery, 77);
        assert_eq!(p.push(rd(75, false), t0 + 5 * MIN).battery, 77);
        assert_eq!(p.push(rd(70, false), t0 + 10 * MIN).battery, 77);
        // a rested mouse can read higher again; that becomes the new peak
        assert_eq!(p.push(rd(78, false), t0 + 15 * MIN).battery, 78);
    }

    #[test]
    fn readings_older_than_the_window_are_dropped() {
        let t0 = Instant::now();
        let mut p = PeakHold::default();
        p.push(rd(77, false), t0);
        assert_eq!(p.push(rd(70, false), t0 + 40 * MIN).battery, 77);
        // 77 is now 45 minutes old: gone
        assert_eq!(p.push(rd(68, false), t0 + 45 * MIN).battery, 70);
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn memory_stays_bounded() {
        let t0 = Instant::now();
        let mut p = PeakHold::default();
        // a reading every 30 seconds for an hour
        for i in 0..120u32 {
            p.push(rd(60, false), t0 + Duration::from_secs(30) * i);
            assert!(p.len() <= MAX_SAMPLES);
        }
    }

    #[test]
    fn charging_shows_the_latest_and_resets_on_change() {
        let t0 = Instant::now();
        let mut p = PeakHold::default();
        p.push(rd(80, false), t0);
        // plugged in: the latest reading, not the old peak
        assert_eq!(p.push(rd(40, true), t0 + MIN).battery, 40);
        assert_eq!(p.push(rd(95, true), t0 + 30 * MIN).battery, 95);
        // unplugged: the high reading taken while charging must not stick
        assert_eq!(p.push(rd(90, false), t0 + 31 * MIN).battery, 90);
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn keeps_the_charge_state_of_the_reading() {
        let mut p = PeakHold::default();
        let shown = p.push(
            Reading {
                battery: 50,
                charging: None,
            },
            Instant::now(),
        );
        assert_eq!(shown.charging, None);
    }
}
