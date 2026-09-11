//! Bytes on the wire.
//!
//! ```text
//! request   u32 len (= 4 + body)  u32 command  body
//! response  u32 len (= 8 + body)  u32 command  u32 status  body
//! ```
//!
//! Strings are UTF-16LE: `u32` unit count then units in requests, NUL-padded
//! fixed slots inside records. Every read is bounds-checked and names the
//! field it failed on.

mod cursor;
mod endpoint;
mod frame;
mod token;
mod writer;

pub use cursor::{utf16, Cursor};
pub use endpoint::{pipe_name_for, Endpoint, Stream};
pub use frame::{read_frame, request, MAX_FRAME};
pub use token::{expect_token, send_token};
pub use writer::Writer;
