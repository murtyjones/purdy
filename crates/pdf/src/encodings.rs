mod glyphnames;
mod mappings;

pub use self::mappings::*;

use encoding::all::UTF_16BE;
use encoding::types::{DecoderTrap, EncoderTrap, Encoding};

pub fn bytes_to_string(encoding: [Option<u16>; 256], bytes: &[u8]) -> String {
    let code_points = bytes
        .iter()
        .filter_map(|&byte| encoding[byte as usize])
        .collect::<Vec<u16>>();
    String::from_utf16_lossy(&code_points)
}

pub fn string_to_bytes(encoding: [Option<u16>; 256], text: &str) -> Vec<u8> {
    text.encode_utf16()
        .filter_map(|ch| encoding.iter().position(|&code| code == Some(ch)))
        .map(|byte| byte as u8)
        .collect()
}

pub fn decode_text(encoding: Option<&str>, bytes: &[u8]) -> String {
    if let Some(encoding) = encoding {
        match encoding {
            "StandardEncoding" => bytes_to_string(STANDARD_ENCODING, bytes),
            "MacRomanEncoding" => bytes_to_string(MAC_ROMAN_ENCODING, bytes),
            "MacExpertEncoding" => bytes_to_string(MAC_EXPERT_ENCODING, bytes),
            "WinAnsiEncoding" => bytes_to_string(WIN_ANSI_ENCODING, bytes),
            "UniGB-UCS2-H" | "UniGB−UTF16−H" => {
                UTF_16BE.decode(bytes, DecoderTrap::Ignore).unwrap()
            }
            "Identity-H" => unimplemented!(),
            _ => String::from_utf8_lossy(bytes).to_string(),
        }
    } else {
        bytes_to_string(STANDARD_ENCODING, bytes)
    }
}

pub fn encode_text(encoding: Option<&str>, text: &str) -> Vec<u8> {
    if let Some(encoding) = encoding {
        match encoding {
            "StandardEncoding" => string_to_bytes(STANDARD_ENCODING, text),
            "MacRomanEncoding" => string_to_bytes(MAC_ROMAN_ENCODING, text),
            "MacExpertEncoding" => string_to_bytes(MAC_EXPERT_ENCODING, text),
            "WinAnsiEncoding" => string_to_bytes(WIN_ANSI_ENCODING, text),
            "UniGB-UCS2-H" | "UniGB−UTF16−H" => {
                UTF_16BE.encode(text, EncoderTrap::Ignore).unwrap()
            }
            "Identity-H" => unimplemented!(),
            _ => text.as_bytes().to_vec(),
        }
    } else {
        string_to_bytes(STANDARD_ENCODING, text)
    }
}
