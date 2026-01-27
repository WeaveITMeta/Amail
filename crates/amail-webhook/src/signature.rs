// =============================================================================
// Webhook Signature Verification
// =============================================================================
// Table of Contents:
// 1. HMAC-SHA256 signature verification
// 2. Timestamp validation
// =============================================================================

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Verify webhook signature using HMAC-SHA256
pub fn verify_webhook_signature(payload: &[u8], signature: &str, secret: &str) -> bool {
    // Empty secret means signature verification is disabled
    if secret.is_empty() {
        return true;
    }

    // Parse signature (format: "sha256=<hex>")
    let expected_hex = if let Some(hex) = signature.strip_prefix("sha256=") {
        hex
    } else {
        signature
    };

    // Decode expected signature
    let expected_bytes = match hex::decode(expected_hex) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    // Compute HMAC
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(mac) => mac,
        Err(_) => return false,
    };

    mac.update(payload);

    // Verify
    mac.verify_slice(&expected_bytes).is_ok()
}

/// Generate webhook signature for testing
pub fn generate_signature(payload: &[u8], secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    
    mac.update(payload);
    let result = mac.finalize();
    
    format!("sha256={}", hex::encode(result.into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_verification() {
        let payload = b"test payload";
        let secret = "test_secret";
        
        let signature = generate_signature(payload, secret);
        assert!(verify_webhook_signature(payload, &signature, secret));
    }

    #[test]
    fn test_invalid_signature() {
        let payload = b"test payload";
        let secret = "test_secret";
        
        assert!(!verify_webhook_signature(payload, "sha256=invalid", secret));
    }

    #[test]
    fn test_empty_secret_allows_all() {
        let payload = b"test payload";
        assert!(verify_webhook_signature(payload, "anything", ""));
    }
}
