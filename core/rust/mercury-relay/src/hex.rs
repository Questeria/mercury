//! Minimal lowercase-hex codec for the opaque byte fields carried in JSON
//! request/response DTOs.
//!
//! Kept dependency-free on purpose: the relay only ever moves opaque handles
//! (route ids, replay tokens, ciphertext, sealed headers, auth tags), so a
//! tiny self-contained codec avoids pulling a transitive crate just to shuttle
//! bytes the relay never interprets.

/// Encode bytes as a lowercase-hex string.
pub fn encode(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 0x0f) as usize] as char);
    }
    out
}

/// Decode a lowercase-or-uppercase-hex string into bytes.
///
/// Returns `Err` on odd length or any non-hex character. Errors carry no
/// content beyond a fixed marker — the relay must never echo attacker input.
pub fn decode(s: &str) -> Result<Vec<u8>, HexError> {
    let bytes = s.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return Err(HexError);
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = nibble(bytes[i])?;
        let lo = nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn nibble(c: u8) -> Result<u8, HexError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(HexError),
    }
}

/// Opaque hex-decode failure. Carries no attacker-controlled content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexError;

impl core::fmt::Display for HexError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("invalid hex")
    }
}

impl std::error::Error for HexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let bytes = [0x00u8, 0x0f, 0xa5, 0xff, 0x10];
        assert_eq!(decode(&encode(&bytes)).unwrap(), bytes);
    }

    #[test]
    fn rejects_odd_length() {
        assert_eq!(decode("abc"), Err(HexError));
    }

    #[test]
    fn rejects_non_hex() {
        assert_eq!(decode("zz"), Err(HexError));
    }
}
