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


pub mod error {
	use std::fmt::{Display, Formatter, Result as FResult};
	use std::string::FromUtf8Error;
	use std::str::Utf8Error;

	// pub type Result<T, E = Error> = std::result::Result<T, E>;
	pub type Result<T, E = Error> = std::result::Result<T, E>;

	#[derive(Debug)]
	pub enum Error {
		EndOfInput(Mode, Expected, Position),
		InvalidUtf8 {
			bytes: Option<Vec<u8>>,
			error: Utf8Error,
		},
	}

	impl Display for Error {
		fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
			match self {
				Self::EndOfInput(Mode::Encoding, Expected::TwoByte, Position::Two) => f.write_str("Unexpected end of input. [Unable to encode a two byte encoding. (Second byte)]"),
				Self::EndOfInput(Mode::Encoding, Expected::ThreeByte, Position::Two) => f.write_str("Unexpected end of input. [Unable to encode a three byte encoding. (Second byte)]"),
				Self::EndOfInput(Mode::Encoding, Expected::ThreeByte, Position::Three) => f.write_str("Unexpected end of input. [Unable to encode a three byte encoding. (Third byte)]"),
				Self::EndOfInput(Mode::Encoding, Expected::FourByte, Position::Two) => f.write_str("Unexpected end of input. [Unable to encode a four byte encoding. (Second byte)]"),
				Self::EndOfInput(Mode::Encoding, Expected::FourByte, Position::Three) => f.write_str("Unexpected end of input. [Unable to encode a four byte encoding. (Third byte)]"),
				Self::EndOfInput(Mode::Encoding, Expected::FourByte, Position::Four) => f.write_str("Unexpected end of input. [Unable to encode a four byte encoding. (Fourth byte)]"),

				Self::EndOfInput(Mode::Decoding, Expected::TwoByte, Position::Two) => f.write_str("Unexpected end of input. [Unable to decode a two byte encoding. (Second byte)]"),
				Self::EndOfInput(Mode::Decoding, Expected::ThreeByte, Position::Two) => f.write_str("Unexpected end of input. [Unable to decode a three byte encoding. (Second byte)]"),
				Self::EndOfInput(Mode::Decoding, Expected::ThreeByte, Position::Three) => f.write_str("Unexpected end of input. [Unable to decode a three byte encoding. (Third byte)]"),
				Self::EndOfInput(Mode::Decoding, Expected::SixByte, Position::Four) => f.write_str("Unexpected end of input. [Unable to decode a six byte encoding. (Fourth byte)]"),
				Self::EndOfInput(Mode::Decoding, Expected::SixByte, Position::Five) => f.write_str("Unexpected end of input. [Unable to decode a six byte encoding. (Fifth byte)]"),
				Self::EndOfInput(Mode::Decoding, Expected::SixByte, Position::Six) => f.write_str("Unexpected end of input. [Unable to decode a six byte encoding. (Sixth byte)]"),

				Self::InvalidUtf8 {
					bytes: _,
					error
				} => {
					f.write_str("Invalid UTF-8 input. [Failed to decode string into UTF-8 (")?;
					Display::fmt(error, f)?;
					f.write_str(")]")
				},

				_ => unreachable!(),
			}
		}
	}

	impl std::error::Error for Error {
	}

	impl From<Utf8Error> for Error {
		fn from(err: Utf8Error) -> Self {
			Error::InvalidUtf8 {
				bytes: None,
				error: err
			}
		}
	}

	impl From<FromUtf8Error> for Error {
		fn from(err: FromUtf8Error) -> Self {
			let error = err.utf8_error();
			let bytes = err.into_bytes();
			Error::InvalidUtf8 {
				bytes: Some(bytes),
				error,
			}
		}
	}

	/// Used to describe the transcoding state.
	/// We define encoding as going to MUTF-8.
	#[derive(Debug)]
	pub enum Mode {
		/// UTF-8 being encoded, and converted into MUTF-8.
		Encoding,
		/// MUTF-8 being decoded, and converted into UTF-8.
		Decoding,
	}

	/// What specifically the conversion functions were trying to encode/decode before they ran into an issue.
	#[derive(Debug)]
	pub enum Expected {
		/// The UTF-8 and MUTF-8 specification both define two byte encodings.
		/// To determine which it refers to, examine the main error enum, that should contain the `Mode`. (This describes if the error occurred during encoding or decoding)
		TwoByte,
		/// The UTF-8 and MUTF-8 specification both define three byte encodings.
		/// To determine which it refers to, examine the main error enum, that should contain the `Mode`. (This describes if the error occurred during encoding or decoding)
		///
		/// Due to how the six byte is encoded, it can be incorrectly reported as a three byte error.
		ThreeByte,
		/// Only the UTF-8 specification defines a four byte encoding.
		/// The four byte equivalent in MUTF-8 is defined as a six byte encoding.
		FourByte,
		/// Only the MUTF-8 specification defines a six-byte encoding.
		/// The four byte equivalent in UTF-8 is defined as a four byte encoding.
		/// In other words, this is MUTF-8's representation of UTF-8's four byte encoding.
		///
		/// Due to how the six byte is encoded, it can be incorrectly reported as a three byte error.
		/// The information on whether a three byte encoding is a six byte or not is encoded in the second and third byte, so if they somehow get cut, we lose that information.
		SixByte,
	}

	/// The position of the current byte during the encoding/decoding phase.
	/// Only some of the positions are valid for some combinations of the bytes.
	///
	/// For example, it doesn't make any sense for there to be an error like: (Expected::TwoByte, Position::Five).
	/// There is no fifth byte to be read as it's only trying to read two bytes, so this should be treated as an internal error.
	#[derive(Debug)]
	pub enum Position {
		Two,
		Three,
		Four,
		Five,
		Six,
	}
}
