use crate::mesh_sync::MemoryDeltaPackage;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Manifest {
    pub updated_at: String,
    pub delta_keys: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CloudflareR2Backend {
    pub account_id: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket: String,
    pub custom_endpoint: Option<String>,
    pub client: Client,
}

impl CloudflareR2Backend {
    pub fn new(
        account_id: String,
        access_key_id: String,
        secret_access_key: String,
        bucket: String,
        custom_endpoint: Option<String>,
    ) -> Self {
        Self {
            account_id,
            access_key_id,
            secret_access_key,
            bucket,
            custom_endpoint,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }

    fn get_host(&self) -> String {
        if let Some(ref ep) = self.custom_endpoint {
            ep.trim_start_matches("https://")
                .trim_start_matches("http://")
                .trim_end_matches('/')
                .to_string()
        } else {
            format!("{}.r2.cloudflarestorage.com", self.account_id)
        }
    }

    fn get_url(&self, key: &str) -> String {
        let clean_key = key.trim_start_matches('/');
        format!("https://{}/{}/{}", self.get_host(), self.bucket, clean_key)
    }

    fn sign_request(
        &self,
        method: &str,
        key: &str,
        query: &str,
        payload: &[u8],
        content_type: Option<&str>,
    ) -> (String, String, String) {
        let now = Utc::now();
        let amz_date = now.format("%Y%m%d%T%H%M%SZ").to_string().replace(':', "");
        let date_stamp = now.format("%Y%m%d").to_string();
        let host = self.get_host();

        let mut hasher = Sha256::new();
        hasher.update(payload);
        let payload_hash = hex::encode(hasher.finalize());

        let clean_key = key.trim_start_matches('/');
        let canonical_uri = format!("/{}/{}", self.bucket, clean_key);

        let (canonical_headers, signed_headers) = if let Some(ct) = content_type {
            (
                format!(
                    "content-type:{}\nhost:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
                    ct, host, payload_hash, amz_date
                ),
                "content-type;host;x-amz-content-sha256;x-amz-date",
            )
        } else {
            (
                format!(
                    "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
                    host, payload_hash, amz_date
                ),
                "host;x-amz-content-sha256;x-amz-date",
            )
        };

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method, canonical_uri, query, canonical_headers, signed_headers, payload_hash
        );

        let mut req_hasher = Sha256::new();
        req_hasher.update(canonical_request.as_bytes());
        let hashed_canonical_request = hex::encode(req_hasher.finalize());

        let credential_scope = format!("{}/auto/s3/aws4_request", date_stamp);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            amz_date, credential_scope, hashed_canonical_request
        );

        let k_date = hmac_sha256(format!("AWS4{}", self.secret_access_key).as_bytes(), date_stamp.as_bytes());
        let k_region = hmac_sha256(&k_date, b"auto");
        let k_service = hmac_sha256(&k_region, b"s3");
        let k_signing = hmac_sha256(&k_service, b"aws4_request");
        let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()));

        let auth_header = format!(
            "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.access_key_id, credential_scope, signed_headers, signature
        );

        (auth_header, amz_date, payload_hash)
    }

    pub async fn put_object(&self, key: &str, data: &[u8], content_type: &str) -> Result<(), String> {
        let (auth, date, payload_hash) = self.sign_request("PUT", key, "", data, Some(content_type));
        let url = self.get_url(key);

        let res = self
            .client
            .put(&url)
            .header("Authorization", auth)
            .header("x-amz-date", date)
            .header("x-amz-content-sha256", payload_hash)
            .header("content-type", content_type)
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| format!("Network error connecting to Cloudflare R2: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(format!("Cloudflare R2 PUT failed (HTTP {}): {}", status, body));
        }

        Ok(())
    }

    pub async fn get_object(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        let (auth, date, payload_hash) = self.sign_request("GET", key, "", &[], None);
        let url = self.get_url(key);

        let res = self
            .client
            .get(&url)
            .header("Authorization", auth)
            .header("x-amz-date", date)
            .header("x-amz-content-sha256", payload_hash)
            .send()
            .await
            .map_err(|e| format!("Network error reading from Cloudflare R2: {}", e))?;

        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(format!("Cloudflare R2 GET failed (HTTP {}): {}", status, body));
        }

        let bytes = res
            .bytes()
            .await
            .map_err(|e| format!("Error reading R2 body bytes: {}", e))?
            .to_vec();

        Ok(Some(bytes))
    }

    pub async fn push_delta_package(&self, package: &MemoryDeltaPackage) -> Result<String, String> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let short_sig = if package.hmac_signature.len() >= 12 {
            &package.hmac_signature[..12]
        } else {
            &package.hmac_signature
        };
        let filename = format!("deltas/{}_{}_{}.json", package.device_source, timestamp, short_sig);

        let json_bytes = serde_json::to_vec_pretty(package)
            .map_err(|e| format!("Failed to serialize MemoryDeltaPackage: {}", e))?;

        self.put_object(&filename, &json_bytes, "application/json").await?;

        // Update Manifest
        let manifest_key = "deltas/manifest.json";
        let mut manifest: R2Manifest = match self.get_object(manifest_key).await? {
            Some(bytes) => serde_json::from_slice(&bytes).unwrap_or(R2Manifest {
                updated_at: Utc::now().to_rfc3339(),
                delta_keys: Vec::new(),
            }),
            None => R2Manifest {
                updated_at: Utc::now().to_rfc3339(),
                delta_keys: Vec::new(),
            },
        };

        if !manifest.delta_keys.contains(&filename) {
            manifest.delta_keys.push(filename.clone());
            // Keep recent 100 deltas in manifest
            if manifest.delta_keys.len() > 100 {
                manifest.delta_keys.drain(0..manifest.delta_keys.len() - 100);
            }
        }
        manifest.updated_at = Utc::now().to_rfc3339();

        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize R2Manifest: {}", e))?;
        self.put_object(manifest_key, &manifest_bytes, "application/json").await?;

        Ok(filename)
    }

    pub async fn pull_all_deltas(&self) -> Result<Vec<MemoryDeltaPackage>, String> {
        let manifest_key = "deltas/manifest.json";
        let manifest: Option<R2Manifest> = match self.get_object(manifest_key).await? {
            Some(bytes) => serde_json::from_slice(&bytes).ok(),
            None => None,
        };

        let Some(manifest) = manifest else {
            return Ok(Vec::new());
        };

        let mut packages = Vec::new();
        for key in manifest.delta_keys {
            if let Some(bytes) = self.get_object(&key).await? {
                if let Ok(pkg) = serde_json::from_slice::<MemoryDeltaPackage>(&bytes) {
                    packages.push(pkg);
                }
            }
        }

        Ok(packages)
    }

    pub async fn test_connection(&self) -> Result<bool, String> {
        let test_key = "health_check.json";
        let payload = serde_json::json!({
            "status": "ok",
            "tested_at": Utc::now().to_rfc3339(),
            "origin": "CodeMemoryPrime"
        });
        let bytes = serde_json::to_vec(&payload).unwrap_or_default();
        self.put_object(test_key, &bytes, "application/json").await?;
        let retrieved = self.get_object(test_key).await?;
        Ok(retrieved.is_some())
    }
}

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut key_block = vec![0u8; 64];
    if key.len() > 64 {
        let hashed_key = Sha256::digest(key);
        key_block[..32].copy_from_slice(&hashed_key);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut i_key_pad = vec![0u8; 64];
    let mut o_key_pad = vec![0u8; 64];
    for i in 0..64 {
        i_key_pad[i] = key_block[i] ^ 0x36;
        o_key_pad[i] = key_block[i] ^ 0x5c;
    }

    let mut inner = Sha256::new();
    inner.update(&i_key_pad);
    inner.update(data);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&o_key_pad);
    outer.update(&inner_hash);
    outer.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r2_sigv4_signing() {
        let backend = CloudflareR2Backend::new(
            "test_account".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
            "test_bucket".to_string(),
            None,
        );

        let (auth, date, hash) = backend.sign_request("PUT", "deltas/test.json", "", b"{\"hello\":\"world\"}", Some("application/json"));

        assert!(auth.starts_with("AWS4-HMAC-SHA256 Credential=test_key/"));
        assert!(auth.contains("SignedHeaders=content-type;host;x-amz-content-sha256;x-amz-date"));
        assert!(auth.contains("Signature="));
        assert!(date.ends_with("Z"));
        assert_eq!(hash.len(), 64);
    }
}
