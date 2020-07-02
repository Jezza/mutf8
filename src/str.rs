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

use crate::utf8_to_mutf8;
use crate::mutf8_to_utf8;

#[cfg(feature = "serde")]
use serde::{
	Deserialize,
	Serialize,
	Serializer,
	Deserializer,
	de::SeqAccess
};

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct MString {
	inner: Box<[u8]>
}

impl MString {
	pub fn from_utf8(input: &[u8]) -> MString {
		let boxed_data = match utf8_to_mutf8(input) {
			Cow::Borrowed(_data) => input.into(),
			Cow::Owned(data) => data.into_boxed_slice(),
		};
		MString {
			inner: boxed_data
		}
	}

	pub fn from_mutf8(input: impl Into<Box<[u8]>>) -> MString {
		MString {
			inner: input.into(),
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
			Cow::Borrowed(_data) => bytes.into_vec(),
			Cow::Owned(data) => data,
		}
	}

	pub fn into_boxed_utf8_bytes(self) -> Box<[u8]> {
		let bytes = self.into_inner();
		match mutf8_to_utf8(&bytes) {
			Cow::Borrowed(_data) => bytes,
			Cow::Owned(data) => data.into_boxed_slice(),
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


#[cfg(feature = "serde")]
impl Serialize for MString {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_bytes(&self.inner)
	}
}

#[cfg(feature = "serde")]
struct MStringVisitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for MStringVisitor {
	type Value = MString;

	fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
		formatter.write_str("mutf8 bytes")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de>, {
		let mut data = vec![];
		while let Some(val) = seq.next_element::<u8>()? {
			data.push(val);
		}
		Ok(MString::from_mutf8(data))
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: serde::de::Error {
		Ok(MString::from_mutf8(v))
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for MString {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		deserializer.deserialize_bytes(MStringVisitor)
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
		mstr::from_mutf8(&self.inner)
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
				let data = mstr::from_mutf8(data);
				Cow::Borrowed(data)
			}
			Cow::Owned(data) => {
				let data = MString::from_mutf8(data);
				Cow::Owned(data)
			}
		}
	}

	pub fn from_mutf8(bytes: &[u8]) -> &mstr {
		unsafe {
			&*(bytes as *const [u8] as *const mstr)
		}
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
		MString::from_mutf8(&self.bytes)
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
