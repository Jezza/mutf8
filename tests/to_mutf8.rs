extern crate mutf8;

use mutf8::{MString, mstr};

macro_rules! assert_owned {
    ($var:ident, $msg:expr) => {
		{
			use std::borrow::Cow;
			if let Cow::Borrowed(_) = $var {
				panic!($msg)
			}
		}    
    };
}

macro_rules! assert_borrowed {
    ($var:ident, $msg:expr) => {
		{
			use std::borrow::Cow;
			if let Cow::Owned(_) = $var {
				panic!($msg)
			}
		}    
    };
}

#[test]
fn ascii_test() {
	let data = mstr::from_utf8(b"value");
	assert_eq!(data.len(), 5);

	assert_borrowed!(data, "Data not borrowed. [It's just pure ascii which uses the same encoding as utf8]");
}

#[test]
fn nul_test() {
	let data = mstr::from_utf8(b"\0");
	assert_eq!(data.len(), 2);

	assert_owned!(data, "Data not owned. [A nul byte needs two bytes in mutf8]");
}
