#![cfg(all(feature = "use-structs", feature = "serde"))]

use mutf8::{utf8_to_mutf8, MString};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Blah(u64, MString, String);

#[test]
fn test_serialise() {
    let value = MString::from_utf8("Hello, World!");

    let value = Blah(64, value, String::new());

    let output = serde_json::to_string(&value).unwrap();
    assert_eq!(
        output,
        r#"[64,[72,101,108,108,111,44,32,87,111,114,108,100,33],""]"#
    );
}

#[test]
fn test_deserialise() {
    let input = r#"[64,[72,101,108,108,111,44,32,87,111,114,108,100,33],""]"#;

    let output: Blah = serde_json::from_str(&input).unwrap();

    assert_eq!(output.1, MString::from_utf8("Hello, World!"));
}

#[test]
fn test_serialise_nul() {
    let value = MString::from_utf8("Hello, \0World!");

    let value = Blah(64, value, String::new());

    let output = serde_json::to_string(&value).unwrap();
    assert_eq!(
        output,
        r#"[64,[72,101,108,108,111,44,32,192,128,87,111,114,108,100,33],""]"#
    );
}

#[test]
fn test_deserialise_nul() {
    let input = r#"[64,[72,101,108,108,111,44,32,192,128,87,111,114,108,100,33],""]"#;

    let output: Blah = serde_json::from_str(&input).unwrap();

    assert_eq!(output.1, MString::from_utf8("Hello, \0World!"));
}
