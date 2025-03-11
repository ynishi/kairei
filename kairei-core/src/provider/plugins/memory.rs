use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::MemoryConfig;
use crate::provider::capability::CapabilityType;
use crate::provider::llm::LLMResponse;
use crate::provider::plugin::{PluginContext, ProviderPlugin};
use crate::provider::provider::Section;
use crate::provider::types::ProviderResult;

/// メモリエントリ
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Memory {
    content: String,
    timestamp: DateTime<Utc>,
    importance: f64,
    metadata: HashMap<String, serde_json::Value>, // contextをそのまま保存
}

/// メモリプラグインの実装
pub struct MemoryPlugin {
    short_term: Arc<RwLock<VecDeque<Memory>>>,
    long_term: Arc<RwLock<Vec<Memory>>>,
    config: MemoryConfig,
}

impl MemoryPlugin {
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            short_term: Arc::new(RwLock::new(VecDeque::with_capacity(config.max_short_term))),
            long_term: Arc::new(RwLock::new(Vec::with_capacity(config.max_long_term))),
            config,
        }
    }

    /// 関連する記憶の検索
    #[tracing::instrument(skip(self, context), level = "debug")]
    async fn retrieve_relevant_memories(
        &self,
        context: &PluginContext<'_>,
    ) -> ProviderResult<Vec<Memory>> {
        let mut memories = Vec::new();

        // 短期記憶からの取得
        {
            let short_term = self.short_term.read().await;
            memories.extend(short_term.iter().cloned());
        }

        // 長期記憶から関連する記憶を検索
        {
            let long_term = self.long_term.read().await;
            let relevant = self.search_relevant_memories(&long_term, context).await?;
            memories.extend(relevant);
        }

        Ok(memories)
    }

    /// 記憶の保存
    #[tracing::instrument(skip(self, memory), level = "debug")]
    async fn store_memory(&self, memory: Memory) -> ProviderResult<()> {
        // 重要度に応じて保存先を決定
        if memory.importance >= self.config.importance_threshold {
            // 長期記憶への保存
            let mut long_term = self.long_term.write().await;
            if long_term.len() >= self.config.max_long_term {
                // 最も重要度の低い記憶を削除
                if let Some(min_idx) = long_term
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| a.importance.partial_cmp(&b.importance).unwrap())
                    .map(|(i, _)| i)
                {
                    long_term.remove(min_idx);
                }
            }
            long_term.push(memory);
        } else {
            // 短期記憶への保存
            let mut short_term = self.short_term.write().await;
            if short_term.len() >= self.config.max_short_term {
                short_term.pop_front(); // 最も古い記憶を削除
            }
            short_term.push_back(memory);
        }
        Ok(())
    }

    /// 関連する記憶の検索（シンプルな類似度計算）
    #[tracing::instrument(skip(self, memories, _context), level = "debug")]
    async fn search_relevant_memories(
        &self,
        memories: &[Memory],
        _context: &PluginContext<'_>,
    ) -> ProviderResult<Vec<Memory>> {
        // ここでは簡単な実装として、最新のN件を返す
        // 実際の実装では、エンベッディングを使用した類似度計算などを行う
        let recent_memories: Vec<Memory> = memories
            .iter()
            .rev()
            .take(5) // 最新の5件
            .cloned()
            .collect();

        Ok(recent_memories)
    }

    /// 記憶のフォーマット
    fn format_memories(&self, memories: Vec<Memory>) -> String {
        let mut formatted = String::from("Previous Context:\n");
        for memory in memories {
            formatted.push_str(&format!(
                "- {} (Importance: {:.2})\n",
                memory.content, memory.importance
            ));
        }
        formatted
    }

    /// レスポンスの重要度を計算
    fn calculate_importance(&self, response: &LLMResponse) -> f64 {
        // 基本スコア (0.0-1.0)
        let mut importance = 0.5;

        // 1. レスポンスの長さによる調整
        // - 長いレスポンスは重要である可能性が高い
        let length_factor = {
            let len = response.content.len();
            if len > 1000 {
                0.2
            } else if len > 500 {
                0.1
            } else if len < 50 {
                -0.1
            } else {
                0.0
            }
        };

        // 2. メタデータからの調整
        let metadata_factor = response
            .metadata
            .finish_reason
            .as_ref()
            .map_or(0.0, |reason| {
                // error should be invalid
                if reason.contains("timeout") {
                    -0.2
                } else {
                    0.0
                }
            });

        // 3. キーワードベースの調整
        let keyword_factor = {
            let content = response.content.to_lowercase();
            let important_keywords = ["critical", "important", "urgent", "key", "essential"];
            let keyword_count = important_keywords
                .iter()
                .filter(|&k| content.contains(k))
                .count();
            (keyword_count as f64) * 0.1
        };

        // 重要度の計算
        importance += length_factor + metadata_factor + keyword_factor;

        // 範囲を0.0-1.0に制限
        importance.clamp(0.0, 1.0)
    }
}

#[async_trait]
impl ProviderPlugin for MemoryPlugin {
    fn priority(&self) -> i32 {
        100 // メモリは高優先度で実行
    }

    fn capability(&self) -> CapabilityType {
        CapabilityType::Memory
    }

    #[tracing::instrument(skip(self, context), level = "debug")]
    async fn generate_section<'a>(&self, context: &PluginContext<'a>) -> ProviderResult<Section> {
        // 関連する記憶の取得
        let memories = self.retrieve_relevant_memories(context).await?;

        // セクションの生成
        Ok(Section {
            content: self.format_memories(memories),
            priority: self.priority(),
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context, response), level = "debug")]
    async fn process_response<'a>(
        &self,
        context: &PluginContext<'a>,
        response: &LLMResponse,
    ) -> ProviderResult<()> {
        // レスポンスを記憶として保存
        let importance = self.calculate_importance(response);

        let mut metadata = HashMap::new();
        metadata.insert("request".to_string(), json!(context.request));
        metadata.insert("context".to_string(), json!(context.context));
        let memory = Memory {
            content: response.content.clone(),
            timestamp: Utc::now(),
            importance,
            metadata,
        };

        self.store_memory(memory).await
    }
}

/// エラー型の定義
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Failed to store memory: {0}")]
    StorageError(String),
    #[error("Failed to retrieve memory: {0}")]
    RetrievalError(String),
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        provider::{llm::ResponseMetadata, plugins::provider_tests::TestContextHolder},
        timestamp::Timestamp,
    };

    use super::*;

    // テスト用のヘルパー関数
    fn create_test_plugin() -> MemoryPlugin {
        MemoryPlugin::new(MemoryConfig {
            max_short_term: 5,
            max_long_term: 10,
            importance_threshold: 0.7,
            max_items: 100,
        })
    }

    fn create_test_response(content: &str) -> LLMResponse {
        LLMResponse {
            content: content.to_string(),
            metadata: ResponseMetadata {
                model: "test".to_string(),
                created_at: Timestamp::now(),
                token_usage: None,
                finish_reason: None,
            },
        }
    }

    // テストでの使用例
    #[tokio::test]
    async fn test_basic_memory_storage() -> ProviderResult<()> {
        let plugin = create_test_plugin();
        let context_holder = TestContextHolder::new("test request");
        let context = context_holder.get_plugin_context();

        let response = create_test_response("test response");
        plugin.process_response(&context, &response).await?;

        let section = plugin.generate_section(&context).await?;
        assert!(section.content.contains("test response"));
        Ok(())
    }

    #[tokio::test]
    async fn test_short_term_memory_limit() -> ProviderResult<()> {
        let plugin = create_test_plugin();
        let context_holder = TestContextHolder::new("test request");
        let context = context_holder.get_plugin_context();

        // max_short_term + 1 個のメモリを保存
        for i in 0..6 {
            let response = create_test_response(&format!("response {}", i));
            plugin.process_response(&context, &response).await?;
        }

        // セクションを生成して内容を確認
        let section = plugin.generate_section(&context).await?;

        // 最古のメモリ（"response 0"）が削除されているはず
        assert!(!section.content.contains("response 0"));
        assert!(section.content.contains("response 5"));
        Ok(())
    }

    #[tokio::test]
    async fn test_importance_calculation() -> ProviderResult<()> {
        let plugin = create_test_plugin();

        // 短いレスポース（低重要度）
        let short_response = create_test_response("ok");
        let short_importance = plugin.calculate_importance(&short_response);

        // 長いレスポンス（高重要度）
        let long_response = create_test_response(&"a".repeat(1000));
        let long_importance = plugin.calculate_importance(&long_response);

        // 重要キーワードを含むレスポンス
        let important_response = create_test_response("This is a critical and important message");
        let important_importance = plugin.calculate_importance(&important_response);

        assert!(short_importance < long_importance);
        assert!(important_importance > short_importance);
        Ok(())
    }

    #[tokio::test]
    async fn test_long_term_memory_storage() -> ProviderResult<()> {
        let plugin = create_test_plugin();
        let context_holder = TestContextHolder::new("test request");
        let context = context_holder.get_plugin_context();

        // 高重要度のレスポンス
        let important_response = create_test_response(
            "This is a critical and important message that needs long-term storage",
        );
        plugin
            .process_response(&context, &important_response)
            .await?;

        // しばらく待機
        tokio::time::sleep(Duration::from_millis(100)).await;

        // セクションを生成して確認
        let section = plugin.generate_section(&context).await?;
        assert!(section.content.contains("critical and important message"));
        Ok(())
    }

    #[tokio::test]
    async fn test_metadata_storage() -> ProviderResult<()> {
        let plugin = create_test_plugin();
        let request_content = "specific test request";
        let context_holder = TestContextHolder::new(request_content);
        let context = context_holder.get_plugin_context();
        let response = create_test_response("test response");

        // メモリの保存
        plugin.process_response(&context, &response).await?;

        // 短期メモリから直接確認
        let short_term = plugin.short_term.read().await;
        let memory = short_term.back().unwrap();

        // メタデータの確認
        assert!(memory.metadata.contains_key("request"));
        assert!(memory.metadata.contains_key("context"));

        // リクエストの内容確認
        let request_json = memory.metadata.get("request").unwrap();
        assert!(request_json.to_string().contains(request_content));

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let plugin = Arc::new(create_test_plugin());
        let context_holder = TestContextHolder::new("test request");
        let mut handles = vec![];

        // 複数の同時アクセス
        for i in 0..10 {
            let plugin = plugin.clone();
            let context_holder = context_holder.clone(); // context_holderをクローン
            let response = create_test_response(&format!("concurrent response {}", i));

            let handle = tokio::spawn(async move {
                let context = context_holder.get_plugin_context();
                plugin.process_response(&context, &response).await
            });
            handles.push(handle);
        }

        // 全てのタスクの完了を待つ
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // 結果の確認（元のcontext_holderを使用）
        let context = context_holder.get_plugin_context();
        let section = plugin.generate_section(&context).await.unwrap();
        assert!(section.content.contains("concurrent response"));
    }
}
