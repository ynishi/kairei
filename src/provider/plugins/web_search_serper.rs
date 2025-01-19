use async_trait::async_trait;
use html2text::from_read;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::provider::{
    capability::CapabilityType,
    llm::LLMResponse,
    plugin::{PluginContext, ProviderPlugin},
    provider::{ProviderSecret, Section},
    types::{ProviderError, ProviderResult},
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
    api_key: String,
    client: Client,
}

impl WebSearchPlugin {
    pub fn new(secret: &ProviderSecret) -> Self {
        let api_key = secret
            .additional_auth
            .get("web_search_serper_api_key")
            .map(|v| v.expose_secret().to_string())
            .unwrap_or_default();
        Self {
            api_key,
            client: Client::new(),
        }
    }

    pub fn try_new(secret: &ProviderSecret) -> ProviderResult<Self> {
        let api_key = secret
            .additional_auth
            .get("web_search_serper_api_key")
            .map(|v| v.expose_secret().to_string())
            .ok_or(ProviderError::Authentication(
                "api key not found in additional_auth.web_search_serper_api_key".to_string(),
            ))?;
        Ok(Self {
            api_key,
            client: Client::new(),
        })
    }

    async fn search(&self, query: &str) -> ProviderResult<Vec<SearchResult>> {
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
                "num": 3
            }))
            .send()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?
            .json()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        let mut results = Vec::new();
        for result in &response.organic {
            let content = Self::fetch_content(&result.link).await?;
            results.push(SearchResult {
                title: result.title.clone(),
                link: result.link.clone(),
                snippet: result.snippet.clone(),
                content,
            });
        }

        Ok(results)
    }

    async fn fetch_content(url: &str) -> ProviderResult<String> {
        let client = Client::new();
        let response = client
            .get(url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
            )
            .send()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

        let html_text = response
            .text()
            .await
            .map_err(|e| ProviderError::ApiError(e.to_string()))?;

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

        // 検索を実行
        let results = self.search(&query).await?;

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
        let plugin = WebSearchPlugin::new(&secret_config);

        assert!(plugin.api_key == "test_web_search_serper_api_key");
        assert_eq!(plugin.capability(), CapabilityType::Search);
    }
}
