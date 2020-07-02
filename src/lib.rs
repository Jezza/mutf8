//!
//! This is more or less just an initial implementation.
//! Currently, only from raw mutf8 and to utf8 operations are supported.
//! Ideally, this will turn into a "complete enough" mutf8 library for use in other libs/apps.
//!

mod mutf8;

#[cfg(feature = "use-structs")]
mod str;

pub use mutf8::mutf8_to_utf8;
pub use mutf8::utf8_to_mutf8;

#[cfg(feature = "use-structs")]
pub use crate::str::MString;

#[cfg(feature = "use-structs")]
pub use crate::str::mstr;
