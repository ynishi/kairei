use std::time::Duration;

use async_trait::async_trait;
use futures::future::join_all;
use html2text::from_read;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;

use crate::{
    config::{PluginConfig, SearchConfig},
    provider::{
        capabilities::common::CapabilityType,
        llm::LLMResponse,
        plugin::{PluginContext, ProviderPlugin},
        provider::{ProviderSecret, Section},
        types::{ProviderError, ProviderResult},
    },
};

#[derive(Debug, Deserialize, Serialize)]
struct SerperResponse {
    #[serde(default)]
    organic: Vec<OrganicResult>,
    #[serde(default, rename = "searchParameters")]
    search_parameters: SearchParameters,
}

#[derive(Debug, Deserialize, Serialize)]
struct OrganicResult {
    title: String,
    link: String,
    snippet: String,
    position: Option<i32>,
    // その他のフィールドは必要に応じて追加
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct SearchParameters {
    #[serde(default)]
    q: String,
    #[serde(default)]
    gl: String,
    #[serde(default)]
    hl: String,
}

#[derive(Debug, Clone)]
pub struct WebSearchPlugin {
    config: SearchConfig,
    api_key: String,
    client: Client,
}

impl WebSearchPlugin {
    pub fn new(config: &SearchConfig, secret: &ProviderSecret) -> Self {
        let api_key = secret
            .additional_auth
            .get("web_search_serper_api_key")
            .map(|v| v.expose_secret().to_string())
            .unwrap_or_default();
        Self {
            config: config.clone(),
            api_key,
            client: Client::new(),
        }
    }

    pub fn try_new(config: &SearchConfig, secret: &ProviderSecret) -> ProviderResult<Self> {
        let api_key = secret
            .additional_auth
            .get("web_search_serper_api_key")
            .map(|v| v.expose_secret().to_string())
            .ok_or(ProviderError::Authentication(
                "api key not found in additional_auth.web_search_serper_api_key".to_string(),
            ))?;
        Ok(Self {
            config: config.clone(),
            api_key,
            client: Client::new(),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn search(
        &self,
        query: &str,
        config: &SearchConfig,
    ) -> ProviderResult<Vec<SearchResult>> {
        debug!("Searching for {}", query);
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-API-KEY",
            HeaderValue::from_str(&self.api_key)
                .map_err(|e| ProviderError::InternalError(e.to_string()))?,
        );

        let response: SerperResponse = self
            .client
            .post("https://google.serper.dev/search")
            .headers(headers)
            .json(&json!({
                "q": query,
                "num": config.max_results,
            }))
            .send()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;
        let timeout = self.config.fetch_timeout;

        let futures = response
            .organic
            .iter()
            .take(config.max_fetch_per_result)
            .map(|result| async move {
                debug!("Fetching content from {}", result.link);
                let content_result = Self::fetch_content(&result.link, &timeout).await;
                let content = match content_result {
                    Ok(content) => content,
                    Err(ProviderError::FetchFailed(_)) => "Failed to fetch content".to_string(),
                    Err(e) => {
                        tracing::warn!("Failed to fetch content: {}", e);
                        format!("Failed to fetch content: {}", e)
                    }
                };
                SearchResult {
                    title: result.title.clone(),
                    link: result.link.clone(),
                    snippet: result.snippet.clone(),
                    content,
                }
            });

        let results = join_all(futures).await;

        Ok(results)
    }

    #[tracing::instrument]
    async fn fetch_content(url: &str, timeout: &Duration) -> ProviderResult<String> {
        let client = Client::new();
        let response = client
            .get(url)
            .timeout(timeout.to_owned())
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
            )
            .send()
            .await
            .map_err(|e| ProviderError::FetchFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ProviderError::FetchFailed(format!(
                "Failed to fetch content from {}: {}",
                url,
                response.status()
            )));
        }

        let html_text = response
            .text()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        if html_text.is_empty() {
            return Err(ProviderError::FetchFailed(format!(
                "Failed to fetch content from {}: empty response",
                url
            )));
        }

        let text = from_read(html_text.as_bytes(), 80)
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        let text = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        Ok(text)
    }

    fn format_results(&self, results: &[SearchResult]) -> String {
        results
            .iter()
            .enumerate()
            .map(|(index, result)| {
                format!(
                    "=============START===========\n\
                    NO. {}\n\
                    TITLE: {}\n\
                    URL: {}\n\
                    Summary: {}\n\
                    ================================\n\
                    {}\n\
                    ==============END=============\n",
                    index + 1,
                    result.title,
                    result.link,
                    result.snippet,
                    result.content,
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[derive(Debug)]
struct SearchResult {
    title: String,
    link: String,
    snippet: String,
    content: String,
}

#[async_trait]
impl ProviderPlugin for WebSearchPlugin {
    fn priority(&self) -> i32 {
        100 // WebSearch処理は優先度高め
    }

    fn capability(&self) -> CapabilityType {
        CapabilityType::Search
    }

    async fn generate_section<'a>(&self, context: &PluginContext<'a>) -> ProviderResult<Section> {
        // リクエストから検索クエリを取得
        let query = context.request.input.query.to_string();

        // WebConfigを取得
        let config =
            if let Some(PluginConfig::Search(search_config)) = context.configs.get("search") {
                search_config
            } else {
                &self.config
            };

        // 検索を実行
        let results = self.search(&query, config).await?;

        // 結果をフォーマット
        let content = self.format_results(&results);

        Ok(Section {
            content,
            priority: self.priority(),
            metadata: Default::default(),
        })
    }

    async fn process_response<'a>(
        &self,
        _context: &PluginContext<'a>,
        _response: &LLMResponse,
    ) -> ProviderResult<()> {
        // Web検索の場合、レスポンス後の処理は特に必要ない
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::provider::ProviderSecret;

    #[tokio::test]
    async fn test_web_search() {
        let mut additional_auth = std::collections::HashMap::new();
        additional_auth.insert(
            "web_search_serper_api_key".to_string(),
            secrecy::SecretString::from("test_web_search_serper_api_key"),
        );
        let secret_config = ProviderSecret {
            api_key: secrecy::SecretString::from("api_key"),
            additional_auth,
        };
        let plugin = WebSearchPlugin::new(&SearchConfig::default(), &secret_config);

        assert!(plugin.api_key == "test_web_search_serper_api_key");
        assert_eq!(plugin.capability(), CapabilityType::Search);
    }
}
