//! Reading little-endian scalars out of a response body, never past its end.

use crate::error::{Error, Result};

/// A read head over a response body. Every read is checked against what
/// remains and names the field it failed on, so a layout that no longer adds
/// up is an error rather than a plausible wrong number.
#[derive(Debug)]
pub struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// Start reading at the beginning of `buf`.
    pub fn new(buf: &'a [u8]) -> Self {
        Cursor { buf, pos: 0 }
    }

    /// Bytes not yet read.
    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    fn take(&mut self, n: usize, field: &'static str) -> Result<&'a [u8]> {
        let end = self.pos.saturating_add(n);
        let slice = self.buf.get(self.pos..end).ok_or_else(|| {
            Error::Protocol(format!(
                "`{field}`: needs {n} bytes at offset {} but {} remain",
                self.pos,
                self.remaining()
            ))
        })?;
        self.pos = end;
        Ok(slice)
    }

    fn array<const N: usize>(&mut self, field: &'static str) -> Result<[u8; N]> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.take(N, field)?);
        Ok(out)
    }

    /// One byte.
    pub fn u8(&mut self, field: &'static str) -> Result<u8> {
        Ok(self.array::<1>(field)?[0])
    }
    /// The terminal's booleans are one byte.
    pub fn bool(&mut self, field: &'static str) -> Result<bool> {
        Ok(self.u8(field)? != 0)
    }
    /// A little-endian `u16`.
    pub fn u16(&mut self, field: &'static str) -> Result<u16> {
        Ok(u16::from_le_bytes(self.array(field)?))
    }
    /// A little-endian `i32`.
    pub fn i32(&mut self, field: &'static str) -> Result<i32> {
        Ok(i32::from_le_bytes(self.array(field)?))
    }
    /// A little-endian `u32`.
    pub fn u32(&mut self, field: &'static str) -> Result<u32> {
        Ok(u32::from_le_bytes(self.array(field)?))
    }
    /// A little-endian `i64`.
    pub fn i64(&mut self, field: &'static str) -> Result<i64> {
        Ok(i64::from_le_bytes(self.array(field)?))
    }
    /// A little-endian `u64`.
    pub fn u64(&mut self, field: &'static str) -> Result<u64> {
        Ok(u64::from_le_bytes(self.array(field)?))
    }
    /// A little-endian IEEE 754 double.
    pub fn f64(&mut self, field: &'static str) -> Result<f64> {
        Ok(f64::from_le_bytes(self.array(field)?))
    }

    /// Consume `bytes` without keeping them; skipping is what keeps the
    /// fields after them aligned.
    pub fn skip(&mut self, bytes: usize, field: &'static str) -> Result<()> {
        self.take(bytes, field).map(|_| ())
    }

    /// `u32` unit count, then UTF-16LE units.
    pub fn string(&mut self, field: &'static str) -> Result<String> {
        let units = self.u32(field)? as usize;
        let bytes = units
            .checked_mul(2)
            .ok_or_else(|| Error::Protocol(format!("`{field}`: implausible length {units}")))?;
        Ok(utf16(self.take(bytes, field)?))
    }

    /// A fixed-width, NUL-padded UTF-16LE slot.
    pub fn fixed_string(&mut self, bytes: usize, field: &'static str) -> Result<String> {
        Ok(utf16(self.take(bytes, field)?))
    }

    /// Element count, checked against what the buffer could hold before any
    /// allocation trusts it.
    pub fn count(&mut self, min_element: usize, field: &'static str) -> Result<usize> {
        let count = self.u32(field)? as usize;
        let fit = self.remaining() / min_element.max(1);
        if count > fit {
            return Err(Error::Protocol(format!(
                "`{field}`: claims {count} elements but only {fit} fit in {} bytes",
                self.remaining()
            )));
        }
        Ok(count)
    }

    /// The whole body must be consumed: a slot width off by one byte leaves a
    /// remainder, and the values already read are then suspect.
    pub fn expect_consumed(&self, what: &'static str) -> Result<()> {
        self.expect_at(self.buf.len(), what)
    }

    /// Same check against a landmark inside the body.
    pub fn expect_at(&self, offset: usize, what: &'static str) -> Result<()> {
        if self.pos != offset {
            return Err(Error::Protocol(format!(
                "`{what}`: fields ended at {} but the layout says {offset} — this terminal \
                 build does not match",
                self.pos
            )));
        }
        Ok(())
    }
}

/// NUL-terminated UTF-16LE, at most `raw.len()` bytes of it.
pub fn utf16(raw: &[u8]) -> String {
    let units = raw
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&u| u != 0);
    char::decode_utf16(units)
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::wire::Writer;

    /// The second string is deliberately awkward: the accented letter needs
    /// more than one byte, and the final character sits above U+FFFF, so it
    /// reaches the wire as a surrogate pair and comes back only if the reader
    /// reassembles one.
    #[test]
    fn strings_round_trip_and_short_buffers_fail_by_name() {
        let body = Writer::new()
            .string("EURUSD")
            .string("café 𝄞")
            .u32(7)
            .into_bytes();
        let mut c = Cursor::new(&body);
        assert_eq!(c.string("a").unwrap(), "EURUSD");
        assert_eq!(c.string("b").unwrap(), "café 𝄞");
        assert_eq!(c.u32("c").unwrap(), 7);
        c.expect_consumed("body").unwrap();
        let err = Cursor::new(&[1, 2, 3]).f64("price").unwrap_err();
        assert!(err.to_string().contains("price"), "{err}");
    }

    #[test]
    fn an_absurd_count_is_rejected_before_allocating() {
        let body = Writer::new().u32(u32::MAX).into_bytes();
        assert!(Cursor::new(&body).count(8, "n").is_err());
    }
}
