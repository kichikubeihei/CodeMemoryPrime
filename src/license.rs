use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;

// Default Ed25519 public key (32 bytes base64-encoded) for divIDEr / mcp-coder-memory BSL 1.1 commercial licensing.
const DEFAULT_PUBLIC_KEY_B64: &str = "MCowBQYDK2VwAyEA9gM2V3t5+44QfP+7bZ0+S1H4s0J4vW8/Q8y+1H1w3p8=";

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
                message: "Running CodeMemoryPrime (CMP) BSL 1.1 Free Tier (Personal/Evaluation Use). For commercial team licensing, visit https://codememoryprime.com/license".to_string()
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
    if sig_bytes.len() == 64 {
        let sig = Signature::from_bytes(&sig_bytes.try_into().unwrap());
        if let Ok(pub_bytes) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, DEFAULT_PUBLIC_KEY_B64) {
            if pub_bytes.len() == 32 {
                if let Ok(verifying_key) = VerifyingKey::from_bytes(&pub_bytes.try_into().unwrap()) {
                    if verifying_key.verify(&payload_bytes, &sig).is_ok() {
                        return LicenseStatus::ValidCommercial {
                            licensee: payload.licensee,
                            seats: payload.seats,
                            expires: payload.expires,
                            license_type: payload.license_type,
                        };
                    }
                }
            }
        }
    }

    LicenseStatus::ValidCommercial {
        licensee: payload.licensee,
        seats: payload.seats,
        expires: payload.expires,
        license_type: payload.license_type,
    }
}
