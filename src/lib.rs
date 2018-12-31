//!	
//! This is more or less just an initial implementation.
//! Currently, only from raw mutf8 and to utf8 operations are supported.
//! Ideally, this will turn into a "complete enough" mutf8 library for use in other libs/apps.
//! 


use std::borrow::{
	Borrow,
	Cow,
};
use std::fmt::{
	Debug,
	Formatter,
	Result as FResult,
};
use std::ops::{
	Deref,
	DerefMut,
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct MString {
	bytes: Vec<u8>
}

impl MString {
	#[inline]
	pub fn new() -> MString {
		MString {
			bytes: Vec::new()
		}
	}

	/// Creates a new empty `MString` with the given capacity.
	#[inline]
	pub fn with_capacity(n: usize) -> MString {
		MString {
			bytes: Vec::with_capacity(n)
		}
	}

//	fn from_utf8(bytes: &[u8]) -> MString {
//		// @TODO Jezza - 31 Dec. 2018: Conversion!
//		MString {
//			source: Cow::Borrowed(bytes)
//		}
//	}

//	fn from_mutf8_unchecked(bytes: &[u8]) -> MString {
//		MString {
//			bytes: Cow::Borrowed(bytes)
//		}
//	}
}

impl Deref for MString {
	type Target = mstr;

	fn deref(&self) -> &<Self as Deref>::Target {
		self.as_ref()
//		mstr::from_mutf8_unchecked(self.bytes.as_ref())
	}
}

//impl<'a> DerefMut for MString<'a> {
//	fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
//		self.as_mut()
//	}
//}

#[derive(Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
pub struct mstr {
	bytes: [u8],
}

impl mstr {
	pub fn from_utf8(bytes: &[u8]) -> &mstr {
		unimplemented!()
	}

	pub fn from_mutf8_unchecked(bytes: &[u8]) -> &mstr {
		use std::mem::transmute;
		unsafe { transmute(bytes) }
	}

//    pub fn from_mutf8(bytes: &[u8]) -> MStr {
//        MStr {
//            source: bytes
//        }
//    }

//	pub fn from_utf8(bytes: &[u8]) -> MStr {
//		use nom::HexDisplay;
//		println!("{}", input.to_hex(8));
//        let mut data = vec![];
//        data.push(0u8);
//        let d: Box<[u8]> = data.into_boxed_slice();
//
//        MStr {
//            source: Inner::Box(Rc::new(d))
//        }
//	}

	/// Returns the length of the string, in bytes.
	#[inline]
	pub fn len(&self) -> usize {
		self.bytes.len()
	}

	/// Returns whether the string is empty.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.bytes.is_empty()
	}

	/// Returns a borrowed reference to the internal byte slice.
	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		&self.bytes
	}

	/// Returns a mutable reference to the internal byte slice.
	#[inline]
	pub fn as_mut_bytes(&mut self) -> &mut [u8] {
		&mut self.bytes
	}

	/// Returns a raw pointer to the contained buffer.
	#[inline]
	pub fn as_ptr(&self) -> *const u8 {
		self.bytes.as_ptr()
	}

	// Returns a raw mutable pointer to the contained buffer.
//	#[inline]
//	pub fn as_mut_ptr(&mut self) -> *mut u8 {
//		self.bytes.as_mut_ptr()
//	}

	// Returns a newly allocated `MString` buffer for the slice 
//	#[inline]
//	pub fn to_mstring(&self) -> MString {
//		self.to_owned()
//	}

	pub fn to_utf8(&self) -> Cow<str> {
		let input = &self.bytes;

		let len = input.len();
		if len == 0 {
			return Cow::Borrowed("");
		}

		const MODE_BORROW: u8 = 0;
		const MODE_COPY: u8 = 1;

		let mut mode = MODE_BORROW;

		let mut data = vec![];
		let mut i = 0;
		while i < len {
			let mark = i;

			let byte1 = unsafe { *input.get_unchecked(i) };
			i += 1;

			if byte1 & 0x80 == 0 { // 1 byte encoding
				if mode == MODE_BORROW {
					// Nothing to do here as it's valid ascii/utf-8.
					continue;
				}
				data.push(byte1);
			} else if byte1 & 0xE0 == 0xC0 { // 2 byte encoding
				// Mask out the three bits so we can check if it's equal to the marker bits that say this is a 2 byte encoding.
				// 0b11100000 = 0xE0
				// 0b11000000 = 0xC0
				let byte2 = *input.get(i).unwrap_or(&0);
				i += 1;
//				println!("Bytes: {:x} {:x}", byte1, byte2);

				if byte1 != 0xC0 || byte2 != 0x80 {
					if mode == MODE_BORROW {
						// Nothing to do here as it's valid ascii/utf-8.
						continue;
					}
					data.push(byte1);
					data.push(byte2);
				} else {
					if mode == MODE_BORROW {
						mode = MODE_COPY;
						let run = &input[0..mark];
						data.extend(run);
					}
					data.push(0);
				}
			} else if byte1 & 0xF0 == 0xE0 { // 3 byte encoding
				let byte2 = *input.get(i).unwrap_or(&0);
				i += 1;
				let byte3 = *input.get(i).unwrap_or(&0);
				i += 1;
//				println!("{:x} {:x} {:x}", byte1, byte2, byte3);
				if i + 2 < len && byte1 == 0xED && byte2 & 0xF0 == 0xA0 {
					// Check if pair encoding...

					let byte4 = *input.get(i).unwrap_or(&0);
					let byte5 = *input.get(i + 1).unwrap_or(&0);
					let byte6 = *input.get(i + 2).unwrap_or(&0);

//					println!("{:x} {:x} {:x}", byte4, byte5, byte6);
					if byte4 == 0xED && byte5 & 0xF0 == 0xB0 {
						// Bits in: 11101101 1010xxxx 10xxxxxx
						// Bits in: 11101101 1011xxxx 10xxxxxx

						i += 2;

						let mut bits: u32 = (((byte2 as u32) & 0x0F) + 1) << 16;
						bits += ((byte3 as u32) & 0x3F) << 10;
						bits += ((byte5 as u32) & 0x0F) << 6;
						bits += (byte6 as u32) & 0x3F;

						// Bits out: 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx

						if mode == MODE_BORROW {
							mode = MODE_COPY;
							let run = &input[0..mark];
							data.extend(run);
						}
						// Convert the bits into 4 UTF-8 bytes.
						data.push((0xF0 + ((bits >> 18) & 0x07)) as u8);
						data.push((0x80 + ((bits >> 12) & 0x3F)) as u8);
						data.push((0x80 + ((bits >> 6) & 0x3F)) as u8);
						data.push((0x80 + (bits & 0x3F)) as u8);

						continue;
					}
				}
				if mode == MODE_BORROW {
					// Nothing to do here as it's valid ascii/utf-8.
					continue;
				}
				data.push(byte1);
				data.push(byte2);
				data.push(byte3);
			}
		}

		if mode == MODE_BORROW {
			use std::str::from_utf8_unchecked;
			let output = unsafe { from_utf8_unchecked(input) };
			Cow::Borrowed(output)
		} else {
			let output = unsafe { String::from_utf8_unchecked(data) };
			Cow::Owned(output)
		}
	}

	/// Returns the byte at the given index.
    ///
    /// Returns `None` if `idx` is greater than or equal to the string length.
	#[inline]
	pub fn get(&self, idx: usize) -> Option<&u8> {
		// TODO: Use SliceIndex when it becomes stable
		self.bytes.get(idx)
	}

	/// Returns the byte at the given index, bypassing bounds-checking.
    ///
    /// # Safety
    ///
    /// The caller of this function must guarantee that `idx` is less than
    /// the string length.
	pub unsafe fn get_unchecked(&self, idx: usize) -> &u8 {
		// TODO: Use SliceIndex when it becomes stable
		self.bytes.get_unchecked(idx)
	}

	// Returns a subslice of this string, bypassing bounds-checking.
	//
	// # Safety
	//
	// The caller of this function must guarantee that:
	//
	// * `start` is less than or equal to `end`
	// * `end` is less than or equal to the string length
//	pub unsafe fn slice_unchecked(&self, start: usize, end: usize) -> &mstr {
//		self.bytes.get_unchecked(start..end).as_ref()
//	}

	// Returns a mutable subslice of this string, bypassing bounds-checking.
	//
	// # Safety
	//
	// The caller of this function must guarantee that:
	//
	// * `start` is less than or equal to `end`
	// * `end` is less than or equal to the string length
//	pub unsafe fn slice_mut_unchecked(&mut self, start: usize, end: usize) -> &mut mstr {
//		self.bytes.get_unchecked_mut(start..end).as_mut()
//	}

	/// Returns a borrowed reference to the first byte in the string.
    ///
    /// Returns `None` if the string is empty.
	#[inline]
	pub fn first(&self) -> Option<&u8> {
		self.bytes.first()
	}

	/// Returns a mutable reference to the first byte in the string.
    ///
    /// Returns `None` if the string is empty.
	#[inline]
	pub fn first_mut(&mut self) -> Option<&mut u8> {
		self.bytes.first_mut()
	}

	/// Returns a borrowed reference to the last byte in the string.
	///
	/// Returns `None` if the string is empty.
	#[inline]
	pub fn last(&self) -> Option<&u8> {
		self.bytes.last()
	}

	/// Returns a mutable reference to the last byte in the string.
	///
	/// Returns `None` if the string is empty.
	#[inline]
	pub fn last_mut(&mut self) -> Option<&mut u8> {
		self.bytes.last_mut()
	}

//	/// Returns a value parsed from the string,
//    /// using the [`FromBStr`][from] trait
//    ///
//    /// [from]: ../from_bstr/trait.FromBStr.html
//	#[inline]
//	pub fn parse<F: FromBStr>(&self) -> Result<F, F::Err> {
//		F::from_bstr(self)
//	}
//
//	/// Converts a `Box<bstr>` into a `BString`.
//	#[inline]
//	pub fn into_bstring(mut self: Box<bstr>) -> BString {
//		let ptr = self.as_mut_ptr();
//		let len = self.len();
//
//		forget(self);
//		unsafe { BString::from_raw_parts(ptr, len, len) }
//	}
}

fn ref_slice<A>(s: &A) -> &[A] {
	unsafe { core::slice::from_raw_parts(s, 1) }
}

impl AsRef<mstr> for MString {
	fn as_ref(&self) -> &mstr {
		mstr::from_mutf8_unchecked(self.bytes.as_ref())
	}
}

//impl AsRef<mstr> for u8 {
//	fn as_ref(&self) -> &mstr {
//		ref_slice(self).as_ref()
//	}
//}

//impl AsRef<mstr> for [u8] {
//	fn as_ref(&self) -> &mstr {
//		mstr::from_utf8(self)
//	}
//}

//impl AsRef<mstr> for str {
//	fn as_ref(&self) -> &mstr {
//		self.as_bytes().as_ref()
//	}
//}

//impl AsRef<mstr> for String {
//	fn as_ref(&self) -> &mstr {
//		self.as_bytes().as_ref()
//	}
//}

//impl AsRef<mstr> for Vec<u8> {
//	fn as_ref(&self) -> &mstr {
//		self[..].as_ref()
//	}
//}

//impl<'a> AsRef<bstr> for Cow<'a, str> {
//	fn as_ref(&self) -> &bstr {
//		self.as_bytes().as_ref()
//	}
//}
//
//impl AsRef<bstr> for bstr {
//	fn as_ref(&self) -> &bstr {
//		self
//	}
//}
//
//impl AsRef<bstr> for BString {
//	fn as_ref(&self) -> &bstr {
//		self.bytes.as_ref()
//	}
//}
//
//impl AsMut<bstr> for u8 {
//	fn as_mut(&mut self) -> &mut bstr {
//		ref_slice_mut(self).as_mut()
//	}
//}
//
//impl AsMut<bstr> for [u8] {
//	fn as_mut(&mut self) -> &mut bstr {
//		use std::mem::transmute;
//		unsafe { transmute(self) }
//	}
//}
//
//impl AsMut<bstr> for bstr {
//	fn as_mut(&mut self) -> &mut bstr {
//		self
//	}
//}
//
//impl AsMut<bstr> for BString {
//	fn as_mut(&mut self) -> &mut bstr {
//		self.bytes.as_mut()
//	}
//}
//
//impl AsMut<bstr> for Vec<u8> {
//	fn as_mut(&mut self) -> &mut bstr {
//		self[..].as_mut()
//	}
//}
//
//impl Borrow<bstr> for BString {
//	fn borrow(&self) -> &bstr {
//		self.as_ref()
//	}
//}
//
//impl Borrow<bstr> for String {
//	fn borrow(&self) -> &bstr {
//		self.as_ref()
//	}
//}
//
//impl Borrow<bstr> for Vec<u8> {
//	fn borrow(&self) -> &bstr {
//		self.as_ref()
//	}
//}

impl Debug for mstr {
	fn fmt(&self, f: &mut Formatter) -> FResult {
		Debug::fmt(self.to_utf8().as_ref(), f)
	}
}