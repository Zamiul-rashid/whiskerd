//! Protocol families. Each one is a pure parser (unit-tested with captured bytes);
//! the I/O around it lives in `hidraw.rs`.

pub mod compx;
pub mod keychron;

/// One battery reading from a mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reading {
    pub battery: u8,
    /// `None` when the protocol does not say.
    pub charging: Option<bool>,
}

#[cfg(test)]
pub fn hex(s: &str) -> Vec<u8> {
    s.split_whitespace()
        .map(|b| u8::from_str_radix(b, 16).unwrap())
        .collect()
}
