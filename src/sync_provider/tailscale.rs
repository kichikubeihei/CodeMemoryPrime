use crate::mesh_sync::MemoryDeltaPackage;
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct TailscaleBackend {
    pub endpoint: String,
    pub client: Client,
}

impl TailscaleBackend {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn push_delta_package(&self, package: &MemoryDeltaPackage) -> Result<String, String> {
        let url = format!("{}/api/memory/delta", self.endpoint);
        let res = self
            .client
            .post(&url)
            .json(package)
            .send()
            .await
            .map_err(|e| format!("Network error pushing delta to Tailscale node ({}): {}", url, e))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Tailscale node returned error (HTTP {}): {}", status, text));
        }

        Ok(format!("Pushed to {}", url))
    }

    pub async fn pull_all_deltas(&self) -> Result<Vec<MemoryDeltaPackage>, String> {
        let url = format!("{}/api/memory/deltas", self.endpoint);
        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error pulling deltas from Tailscale node ({}): {}", url, e))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Tailscale node GET deltas failed (HTTP {}): {}", status, text));
        }

        let deltas = res
            .json::<Vec<MemoryDeltaPackage>>()
            .await
            .map_err(|e| format!("Failed to parse deltas from Tailscale node: {}", e))?;

        Ok(deltas)
    }

    pub async fn test_connection(&self) -> Result<bool, String> {
        let url = format!("{}/api/memory/health", self.endpoint);
        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Tailscale node unreachable at {}: {}", url, e))?;

        Ok(res.status().is_success())
    }
}
