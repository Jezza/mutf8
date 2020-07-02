# MUTF-8

## Usage

```rust
fn main() {
    let output: Cow<u8> = mutf8::utf8_to_mutf8("Hello, \0World");

    // `output` contains no NUL bytes.
}
```

There's also a `MString` and `mstr` struct.  
These are the counterparts to `String` and `str` within the standard library.  

```rust
fn main() {
    let data = mstr::from_utf8(b"\0");
	assert_eq!(data.len(), 2);
}
```

## About
This crate allows converting UTF-8 to and from MUTF-8.

Some data formats, such as the JVM classfile, make use of an altered UTF-8 encoding.  
This one in particular is the MUTF-8 variant.

It allows a NUL byte to be encoded without using the NUL byte itself.  


## WIP

The algorithm itself is done, and useable.  
It works as well as any other.  

The reason I still call this crate WIP is because of the two String structs.  
I'm not happy with them.

I do use this crate for a couple of projects, but _none_ of them make use of the structs themselves.  

I typically use this crate as just a jump from a `[u8]` to a `Cow<str>`.  
So, until I work out where I want to go with this crate, it's probably going to stay like this.
