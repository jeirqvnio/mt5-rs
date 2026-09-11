//! The two shapes a reply comes in, and the macro that reads one record.

use crate::error::Result;
use crate::wire::Cursor;

/// `field: reader` or `field: reader(arg)`, read in order from `$c`, each
/// labelled `"<prefix>.<field>"`. Prefix a name with `_` to read and drop it.
macro_rules! fields {
    ($c:ident, $prefix:literal, { $($field:ident : $method:ident $(($arg:expr))? ),+ $(,)? }) => {
        $(let $field = $c.$method($($arg,)? concat!($prefix, ".", stringify!($field)))?;)+
    };
}

/// A `u32` count and that many records, consuming the body exactly. An empty
/// body is the terminal's "nothing", not a truncation.
pub(crate) fn batch<T>(
    buf: &[u8],
    min: usize,
    what: &'static str,
    one: fn(&mut Cursor) -> Result<T>,
) -> Result<Vec<T>> {
    if buf.is_empty() {
        return Ok(Vec::new());
    }
    let mut c = Cursor::new(buf);
    let count = c.count(min, what)?;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push(one(&mut c)?);
    }
    c.expect_consumed(what)?;
    Ok(out)
}

/// One record and nothing else.
pub(crate) fn single<T>(
    buf: &[u8],
    what: &'static str,
    one: fn(&mut Cursor) -> Result<T>,
) -> Result<T> {
    let mut c = Cursor::new(buf);
    let record = one(&mut c)?;
    c.expect_consumed(what)?;
    Ok(record)
}

/// A bare `u32`: totals, the handshake build, acknowledgements.
pub fn count(buf: &[u8], what: &'static str) -> Result<u32> {
    Cursor::new(buf).u32(what)
}
