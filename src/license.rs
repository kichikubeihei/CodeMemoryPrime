use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;

// Default Ed25519 public key (32 bytes base64-encoded) for divider / mcp-coder-memory BSL 1.1 commercial licensing.
const DEFAULT_PUBLIC_KEY_B64: &str = "453G9lTA37XHXhDe+sw/yoEIjybAtP/cNWhlJpnvvx8=";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
    pub licensee: String,
    pub seats: u32,
    pub expires: String, // YYYY-MM-DD
    pub license_type: String, // "commercial" | "enterprise"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LicenseStatus {
    FreeTier { message: String },
    ValidCommercial { licensee: String, seats: u32, expires: String, license_type: String },
    Expired { licensee: String, expires: String },
    Invalid { reason: String },
}

pub fn calculate_chunk_hash(code_content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code_content.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn check_license_key(key_str: Option<&str>) -> LicenseStatus {
    let key = match key_str {
        Some(k) if !k.trim().is_empty() => k.trim().to_string(),
        _ => match std::env::var("CMP_LICENSE_KEY").or_else(|_| std::env::var("DIVIDER_LICENSE_KEY")) {
            Ok(k) if !k.trim().is_empty() => k,
            _ => return LicenseStatus::FreeTier {
                message: "Running CodeMemoryPrime (CMP) BSL 1.1 Free Tier (Personal/Evaluation Use). For commercial team licensing, visit https://www.codememoryprime.com".to_string()
            },
        },
    };

    // Format: CMP-LICENSE-<payload_b64>.<sig_b64> or DIVIDER-LICENSE-<payload_b64>.<sig_b64>
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 2 {
        return LicenseStatus::Invalid { reason: "License key format invalid. Expected payload.signature format.".to_string() };
    }

    let payload_part = parts[0].trim_start_matches("CMP-LICENSE-").trim_start_matches("DIVIDER-LICENSE-");
    let sig_part = parts[1];

    let payload_bytes = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, payload_part) {
        Ok(b) => b,
        Err(_) => return LicenseStatus::Invalid { reason: "Failed to decode license payload base64.".to_string() },
    };

    let sig_bytes = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, sig_part) {
        Ok(b) => b,
        Err(_) => return LicenseStatus::Invalid { reason: "Failed to decode license signature base64.".to_string() },
    };

    let payload: LicensePayload = match serde_json::from_slice(&payload_bytes) {
        Ok(p) => p,
        Err(_) => return LicenseStatus::Invalid { reason: "Invalid JSON payload structure inside license key.".to_string() },
    };

    // Check expiration date
    if let Ok(exp_date) = chrono::NaiveDate::parse_from_str(&payload.expires, "%Y-%m-%d") {
        let now = chrono::Local::now().naive_local().date();
        if now > exp_date {
            return LicenseStatus::Expired { licensee: payload.licensee, expires: payload.expires };
        }
    }

    // Verify Ed25519 signature
    if sig_bytes.len() != 64 {
        return LicenseStatus::Invalid { reason: "Signature length must be 64 bytes.".to_string() };
    }

    let sig = Signature::from_bytes(&sig_bytes.try_into().unwrap());
    
    let pub_bytes = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, DEFAULT_PUBLIC_KEY_B64) {
        Ok(b) => b,
        Err(_) => return LicenseStatus::Invalid { reason: "Failed to decode compiled public key.".to_string() },
    };

    if pub_bytes.len() != 32 {
        return LicenseStatus::Invalid { reason: "Compiled public key is not 32 bytes.".to_string() };
    }

    let verifying_key = match VerifyingKey::from_bytes(&pub_bytes.try_into().unwrap()) {
        Ok(vk) => vk,
        Err(_) => return LicenseStatus::Invalid { reason: "Failed to parse compiled public key.".to_string() },
    };

    if verifying_key.verify(&payload_bytes, &sig).is_err() {
        return LicenseStatus::Invalid { reason: "Signature verification failed.".to_string() };
    }

    LicenseStatus::ValidCommercial {
        licensee: payload.licensee,
        seats: payload.seats,
        expires: payload.expires,
        license_type: payload.license_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{SigningKey, Signer};
    use base64::Engine;

    #[test]
    fn test_invalid_license_format() {
        let res = check_license_key(Some("CMP-LICENSE-badpayload.badsig"));
        assert!(matches!(res, LicenseStatus::Invalid { .. }));
    }

    #[test]
    fn test_signature_verification_failure() {
        let payload = LicensePayload {
            licensee: "Tester".to_string(),
            seats: 2,
            expires: "2099-01-01".to_string(),
            license_type: "commercial".to_string(),
        };
        let payload_json = serde_json::to_vec(&payload).unwrap();
        let payload_b64 = base64::engine::general_purpose::STANDARD.encode(&payload_json);
        
        let dummy_sig = [0u8; 64];
        let sig_b64 = base64::engine::general_purpose::STANDARD.encode(&dummy_sig);
        
        let key = format!("CMP-LICENSE-{}.{}", payload_b64, sig_b64);
        let res = check_license_key(Some(&key));
        
        match res {
            LicenseStatus::Invalid { reason } => {
                println!("DEBUG REASON: {}", reason);
                assert!(reason.contains("Signature verification failed"));
            }
            other => panic!("Expected Invalid status, got {:?}", other),
        }
    }

    #[test]
    fn test_expired_license() {
        let payload = LicensePayload {
            licensee: "Expired Tester".to_string(),
            seats: 2,
            expires: "2020-01-01".to_string(),
            license_type: "commercial".to_string(),
        };
        let payload_json = serde_json::to_vec(&payload).unwrap();
        let payload_b64 = base64::engine::general_purpose::STANDARD.encode(&payload_json);
        
        let dummy_sig = [0u8; 64];
        let sig_b64 = base64::engine::general_purpose::STANDARD.encode(&dummy_sig);
        
        let key = format!("CMP-LICENSE-{}.{}", payload_b64, sig_b64);
        let res = check_license_key(Some(&key));
        
        match res {
            LicenseStatus::Expired { licensee, expires } => {
                assert_eq!(licensee, "Expired Tester");
                assert_eq!(expires, "2020-01-01");
            }
            other => panic!("Expected Expired status, got {:?}", other),
        }
    }

    #[test]
    fn test_valid_license_flow() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let verifying_key = signing_key.verifying_key();
        
        let payload = LicensePayload {
            licensee: "Valid User".to_string(),
            seats: 10,
            expires: "2099-12-31".to_string(),
            license_type: "commercial".to_string(),
        };
        let payload_json = serde_json::to_vec(&payload).unwrap();
        let sig = signing_key.sign(&payload_json);
        
        assert!(verifying_key.verify(&payload_json, &sig).is_ok());
    }
}

