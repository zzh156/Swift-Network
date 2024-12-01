//! Utility functions and helpers.

mod crypto;

pub use crypto::{hash_message, verify_signature};

use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp in milliseconds
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Get current timestamp in seconds
pub fn current_timestamp() -> u64 {
    current_timestamp_ms() / 1000
}

/// Convert bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Convert hex string to bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(hex)
}

/// Pad bytes to length
pub fn pad_bytes(bytes: &[u8], length: usize) -> Vec<u8> {
    let mut result = vec![0u8; length];
    let copy_len = std::cmp::min(bytes.len(), length);
    result[..copy_len].copy_from_slice(&bytes[..copy_len]);
    result
}

/// Truncate bytes to length
pub fn truncate_bytes(bytes: &[u8], length: usize) -> Vec<u8> {
    bytes.iter().take(length).cloned().collect()
}