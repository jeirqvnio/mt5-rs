//! Building a request body in wire order.

/// Builds a request body by appending fields in wire order.
#[derive(Debug, Default)]
pub struct Writer(Vec<u8>);

impl Writer {
    /// An empty body.
    pub fn new() -> Self {
        Writer(Vec::with_capacity(64))
    }
    /// The finished body.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
    /// A one-byte boolean, which is the width the terminal uses.
    pub fn bool(mut self, v: bool) -> Self {
        self.0.push(u8::from(v));
        self
    }
    /// A little-endian `u32`.
    pub fn u32(mut self, v: u32) -> Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    /// A little-endian `i64`.
    pub fn i64(mut self, v: i64) -> Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    /// A little-endian `u64`.
    pub fn u64(mut self, v: u64) -> Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    /// A little-endian IEEE 754 double.
    pub fn f64(mut self, v: f64) -> Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    /// `u32` unit count, then UTF-16LE units.
    pub fn string(mut self, s: &str) -> Self {
        let units: Vec<u16> = s.encode_utf16().collect();
        self.0
            .extend_from_slice(&(units.len() as u32).to_le_bytes());
        self.0.extend(units.iter().flat_map(|u| u.to_le_bytes()));
        self
    }
    /// A fixed-width NUL-padded slot. One unit is kept for the terminator, so
    /// a 64-byte slot holds 31 characters; longer input is truncated rather
    /// than allowed to run into the next field. A slot too small to hold even
    /// a terminator is written as padding alone.
    pub fn fixed_string(mut self, s: &str, slot_bytes: usize) -> Self {
        let start = self.0.len();
        self.0.resize(start + slot_bytes, 0);
        // Saturating: a slot of 0 or 1 byte has room for no characters at all,
        // and `slot_bytes / 2 - 1` would wrap to a count that writes past the
        // end of the buffer just resized for it.
        let room = (slot_bytes / 2).saturating_sub(1);
        for (i, unit) in s.encode_utf16().take(room).enumerate() {
            self.0[start + i * 2..start + i * 2 + 2].copy_from_slice(&unit.to_le_bytes());
        }
        self
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::wire::Cursor;

    #[test]
    fn fixed_slots_truncate_and_stop_at_nul() {
        let body = Writer::new()
            .fixed_string(&"x".repeat(100), 64)
            .into_bytes();
        assert_eq!(body.len(), 64);
        assert_eq!(Cursor::new(&body).fixed_string(64, "s").unwrap().len(), 31);
    }

    /// A slot with no room for a terminator must write padding, not wrap its
    /// character count and run off the end of the buffer it just sized.
    #[test]
    fn a_slot_too_small_for_a_terminator_writes_only_padding() {
        for slot in [0, 1, 2] {
            let body = Writer::new().fixed_string("abc", slot).into_bytes();
            assert_eq!(body.len(), slot, "slot of {slot}");
            assert!(
                body.iter().all(|&b| b == 0),
                "a slot of {slot} must be padding alone"
            );
        }
        // Four bytes hold one character and its terminator.
        let body = Writer::new().fixed_string("abc", 4).into_bytes();
        assert_eq!(Cursor::new(&body).fixed_string(4, "s").unwrap(), "a");
    }
}
