//! Base64 encoding and decoding utilities
//!
//! This module provides functions for encoding and decoding base64 data.

use crate::error::Error;

/// Encode data as base64
pub fn encode_base64(data: &[u8]) -> String {
    base64::encode(data)
}

/// Decode base64 data
pub fn decode_base64(data: &str) -> Result<Vec<u8>, Error> {
    base64::decode(data)
        .map_err(|e| Error::ParseError(format!("Failed to decode base64: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let original = b"Hello, World!";
        let encoded = encode_base64(original);
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(original, decoded.as_slice());
    }
}
