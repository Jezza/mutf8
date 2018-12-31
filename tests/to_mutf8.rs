extern crate mutf8;

use mutf8::{MString, mstr};

#[test]
fn test() {
	let t = MString::new();
	method(&t, &t);
//	let value = "test".into();
//	method(&value, &value);
//	let data = "test".into_boxed_bytes();
//	let t = MStr::from_utf8(data.as_ref());
//	println("{:?}", L);
//	method(t);
}

use std::convert::AsRef;
fn method(value0: impl AsRef<mstr>, value1: impl AsRef<mstr>) {
	println!("{:?}", value0.as_ref());
	println!("{:?}", value1.as_ref());
}
