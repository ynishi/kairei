use kairei_http::models::ListSystemsResponse;
use reqwest::{Client, StatusCode};
use serde_json::json;
use std::error::Error;

pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn list_systems(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let url = format!("{}/api/v1/systems", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let data = response.json::<ListSystemsResponse>().await?;
                Ok(data
                    .system_statuses
                    .into_iter()
                    .map(|(id, status)| {
                        let v = json!(status);
                        json!({
                            "id": id,
                            "status": v.to_string(),
                        })
                        .to_string()
                    })
                    .collect())
            }
            status => Err(format!("API error: {}", status).into()),
        }
    }

    // Additional methods for each API endpoint...
}
