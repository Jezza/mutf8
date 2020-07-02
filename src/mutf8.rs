use std::borrow::Cow;

pub fn utf8_to_mutf8(input: &[u8]) -> Cow<[u8]> {
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
			bits += (byte4 as u32) & 0x3F;

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

pub fn mutf8_to_utf8(input: &[u8]) -> Cow<[u8]> {
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