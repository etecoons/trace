//! Small helpers that don't fit elsewhere.
//!
//! Most of what used to live here has been replaced by [`shared-backend`]:
//!
//! - Constant-time comparison → [`constant_time_eq`]
//! - Client-IP extraction   → [`shared_backend::server::get_client_ip`]
//! - PIN-hash helper        → [`hash_pin`] (kept for the local session
//!   ID generator in `auth.rs`)

#[allow(dead_code)]
pub fn hash_pin(pin: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    let result = hasher.finalize();
    result
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_pin_is_deterministic() {
        assert_eq!(hash_pin("test"), hash_pin("test"));
    }

    #[test]
    fn hash_pin_differs_per_input() {
        assert_ne!(hash_pin("a"), hash_pin("b"));
    }

    #[test]
    fn hash_pin_is_64_hex_chars() {
        // SHA-256 = 32 bytes = 64 hex chars.
        assert_eq!(hash_pin("x").len(), 64);
    }
}
