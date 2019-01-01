//!	
//! This is more or less just an initial implementation.
//! Currently, only from raw mutf8 and to utf8 operations are supported.
//! Ideally, this will turn into a "complete enough" mutf8 library for use in other libs/apps.
//! 


use std::borrow::{
	Borrow,
	Cow,
	ToOwned,
};
use std::fmt::{
	Debug,
	Formatter,
	Result as FResult,
};
use std::ops::Deref;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct MString {
	inner: Box<[u8]>
}

impl MString {
	pub fn from_utf8<T: Into<Vec<u8>>>(t: T) -> MString {
		let bytes = t.into();
		let boxed_data = match utf8_to_mutf8(&bytes) {
			Cow::Borrowed(data) => {
				// Then we can just consume the input vector.
				bytes.into_boxed_slice()
			}
			Cow::Owned(data) => {
				// We need convert the returned vector into a boxed slice
				data.into_boxed_slice()
			}
		};
		MString {
			inner: boxed_data
		}
	}

	pub fn from_mutf8<T: Into<Vec<u8>>>(t: T) -> MString {
		// @FIXME Jezza - 01 Jan. 2019: I guess the only way to verify it is check if there's a nul byte?
		// I'll just let this sit here, as I have no idea what would be a good idea...
		// Actually, now that I've thought about it, there is something to check...
		let data = t.into();
		MString {
			inner: data.into_boxed_slice()
		}
	}

	pub unsafe fn from_mutf8_unchecked<T: Into<Vec<u8>>>(t: T) -> MString {
		MString {
			inner: t.into().into_boxed_slice()
		}
	}

	pub fn into_string(self) -> String {
		unsafe { String::from_utf8_unchecked(self.into_utf8_bytes()) }
	}

	pub fn into_mutf8_bytes(self) -> Vec<u8> {
		self.into_boxed_mutf8_bytes().into_vec()
	}

	pub fn into_boxed_mutf8_bytes(self) -> Box<[u8]> {
		self.into_inner()
	}

	pub fn into_utf8_bytes(self) -> Vec<u8> {
		let bytes = self.into_inner();
		match mutf8_to_utf8(&bytes) {
			Cow::Borrowed(data) => {
				bytes.into_vec()
			}
			Cow::Owned(data) => {
				data
			}
		}
	}

	pub fn into_boxed_utf8_bytes(self) -> Box<[u8]> {
		let bytes = self.into_inner();
		match mutf8_to_utf8(&bytes) {
			Cow::Borrowed(data) => {
				bytes
			}
			Cow::Owned(data) => {
				data.into_boxed_slice()
			}
		}
	}

	pub fn as_mstr(&self) -> &mstr {
		self
	}

	pub fn into_boxed_str(self) -> Box<str> {
		unsafe { Box::from_raw(Box::into_raw(self.into_boxed_utf8_bytes()) as *mut str) }
	}

	pub fn into_boxed_mstr(self) -> Box<mstr> {
		unsafe { Box::from_raw(Box::into_raw(self.into_boxed_mutf8_bytes()) as *mut mstr) }
	}

	fn into_inner(self) -> Box<[u8]> {
		let result = unsafe {
			use std::ptr::read;
			read(&self.inner)
		};

		use std::mem::forget;
		forget(self);

		result
	}

	pub fn as_mutf8_bytes(&self) -> &[u8] {
		&self.inner
	}

	pub fn as_utf8_bytes(&self) -> Cow<[u8]> {
		mutf8_to_utf8(&self.inner)
	}
}

impl Borrow<mstr> for MString {
	fn borrow(&self) -> &mstr {
		self
	}
}

impl Deref for MString {
	type Target = mstr;

	fn deref(&self) -> &<Self as Deref>::Target {
		let data = self.inner.as_ref();
		mstr::from_mutf8_unchecked(data)
	}
}

impl Debug for MString {
	fn fmt(&self, f: &mut Formatter) -> FResult {
		Debug::fmt(&**self, f)
	}
}

impl From<MString> for Vec<u8> {
	#[inline]
	fn from(s: MString) -> Vec<u8> {
		s.into_utf8_bytes()
	}
}

#[derive(Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
pub struct mstr {
	bytes: [u8],
}

impl mstr {
	pub fn from_utf8(bytes: &[u8]) -> Cow<mstr> {
		match utf8_to_mutf8(bytes) {
			Cow::Borrowed(data) => {
				let data = mstr::from_mutf8_unchecked(data);
				Cow::Borrowed(data)
			}
			Cow::Owned(data) => {
				let data = unsafe { MString::from_mutf8_unchecked(data) };
				Cow::Owned(data)
			}
		}
	}

	pub fn from_mutf8_unchecked(bytes: &[u8]) -> &mstr {
		use std::mem::transmute;
		unsafe { transmute(bytes) }
	}

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

	pub fn to_str(&self) -> Cow<str> {
		self.to_utf8()
	}

	pub fn to_utf8(&self) -> Cow<str> {
		let input = &self.bytes;

		// @FIXME Jezza - 01 Jan. 2019: Eh, I don't know if I like this solution...
		// I like separating all of the mutf8 -> utf8 code, but destructuring the Cow like this...
		match mutf8_to_utf8(input) {
			Cow::Borrowed(data) => {
				use std::str::from_utf8_unchecked;
				let output = unsafe { from_utf8_unchecked(data) };
				Cow::Borrowed(output)
			}
			Cow::Owned(data) => {
				let output = unsafe { String::from_utf8_unchecked(data) };
				Cow::Owned(output)
			}
		}
	}

	pub fn into_m_string(self: Box<mstr>) -> MString {
		let raw = Box::into_raw(self) as *mut [u8];
		MString { inner: unsafe { Box::from_raw(raw) } }
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
}

impl From<MString> for Box<mstr> {
	#[inline]
	fn from(s: MString) -> Box<mstr> {
		s.into_boxed_mstr()
	}
}

impl<'a> From<MString> for Cow<'a, mstr> {
	fn from(s: MString) -> Cow<'a, mstr> {
		Cow::Owned(s)
	}
}

impl<'a> From<Cow<'a, mstr>> for MString {
	#[inline]
	fn from(s: Cow<'a, mstr>) -> Self {
		s.into_owned()
	}
}

impl From<Box<mstr>> for MString {
	fn from(value: Box<mstr>) -> Self {
		value.into_m_string()
	}
}

impl ToOwned for mstr {
	type Owned = MString;

	fn to_owned(&self) -> MString {
		unsafe { MString::from_mutf8_unchecked(&self.bytes) }
	}
}

impl AsRef<mstr> for mstr {
	#[inline]
	fn as_ref(&self) -> &mstr {
		self
	}
}

impl AsRef<mstr> for MString {
	#[inline]
	fn as_ref(&self) -> &mstr {
		self
	}
}

impl Debug for mstr {
	fn fmt(&self, f: &mut Formatter) -> FResult {
		Debug::fmt(self.to_utf8().as_ref(), f)
	}
}

fn utf8_to_mutf8(input: &[u8]) -> Cow<[u8]> {
	let len = input.len();
	if len == 0 {
		return Cow::Borrowed(input);
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

		// nul bytes and bytes starting with 11110xxx are somewhat special
		if byte1 & 0x80 == 0 { // 1-byte encoding
			if byte1 == 0 {
				if mode == MODE_BORROW {
					mode = MODE_COPY;
					let run = &input[0..mark];
					data.extend(run);
				}
				data.push(0xC0);
				data.push(0x80);
			} else if mode == MODE_COPY {
				data.push(byte1);
			}
		} else if byte1 & 0xE0 == 0xC0 { // 2-byte encoding
			if mode == MODE_COPY {
				data.push(byte1);
				let byte2 = *input.get(i).unwrap_or(&0);
				i += 1;
				data.push(byte2);
			}
		} else if byte1 & 0xF0 == 0xE0 { // 3-byte encoding
			if mode == MODE_COPY {
				data.push(byte1);
				let byte2 = *input.get(i).unwrap_or(&0);
				i += 1;
				data.push(byte2);
				let byte3 = *input.get(i).unwrap_or(&0);
				i += 1;
				data.push(byte3);
			}
		} else if byte1 & 0xF8 == 0xF0 {
			if mode == MODE_BORROW {
				mode = MODE_COPY;
				let run = &input[0..mark];
				data.extend(run);
			}

			// Beginning of 4-byte encoding, turn into 2 3-byte encodings
			// Bits in: 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
			let byte2 = *input.get(i).unwrap_or(&0);
			i += 1;
			let byte3 = *input.get(i).unwrap_or(&0);
			i += 1;
			let byte4 = *input.get(i).unwrap_or(&0);
			i += 1;

			// Reconstruct full 21-bit value
			let mut bits: u32 = ((byte1 as u32) & 0x07) << 18;
			bits += ((byte2 as u32) & 0x3F) << 12;
			bits += ((byte3 as u32) & 0x3F) << 6;
			bits += ((byte4 as u32) & 0x3F);

			// Bits out: 11101101 1010xxxx 10xxxxxx
			data.push(0xED);
			data.push((0xA0 + (((bits >> 16) - 1) & 0x0F)) as u8);
			data.push((0x80 + ((bits >> 10) & 0x3F)) as u8);

			// Bits out: 11101101 1011xxxx 10xxxxxx
			data.push(0xED);
			data.push((0xB0 + ((bits >> 6) & 0x0F)) as u8);
			data.push(byte4);
		}
	}

	if mode == MODE_BORROW {
		Cow::Borrowed(input)
	} else {
		Cow::Owned(data)
	}
}

fn mutf8_to_utf8(input: &[u8]) -> Cow<[u8]> {
	let len = input.len();
	if len == 0 {
		return Cow::Borrowed(input);
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
		Cow::Borrowed(input)
	} else {
		Cow::Owned(data)
	}
}