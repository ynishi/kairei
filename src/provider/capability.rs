use mockall::automock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::types::{ProviderError, ProviderResult};

/// Provider Capabilityの種類を定義
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum CapabilityType {
    // Core Capabilities
    /// 基本的な文章生成機能
    Generate,
    /// ポリシーベースの制御機能
    Policy,

    // Interaction Capabilities
    /// スレッド/会話の維持機能
    Thread,
    /// メモリ/状態保持機能
    Memory,
    /// ストリーミング処理機能
    // Streaming,

    // Knowledge Capabilities
    /// RAG (Retrieval Augmented Generation)
    Rag,
    /// Web検索機能
    Search,
    /// 外部データソースとの連携
    // ExternalData,

    // Function Capabilities
    /// 関数呼び出し機能
    // FunctionCall,

    // Model Capabilities
    /// モデルの最大トークン数
    // MaxTokens(usize),
    /// コンテキストウィンドウサイズ
    // WindowSize(usize),
    /// システムプロンプトのサポート
    SystemPrompt,
    /// 独自のトークン化方式
    // TokenEncoding,

    // Custom Capabilities
    /// カスタム機能
    Custom(String),
}

/// Capability管理構造体
#[derive(Debug, Clone, Default)]
pub struct Capabilities {
    capabilities: HashSet<CapabilityType>,
}

impl From<CapabilityType> for Capabilities {
    fn from(capability: CapabilityType) -> Self {
        let mut capabilities = HashSet::new();
        capabilities.insert(capability);
        Self { capabilities }
    }
}

impl From<Vec<CapabilityType>> for Capabilities {
    fn from(capabilities: Vec<CapabilityType>) -> Self {
        Self {
            capabilities: HashSet::from_iter(capabilities),
        }
    }
}

/// Capabilities の操作を行うメソッドを提供
/// 外部向けにはVecを返すが、内部的にはHashSetを使用。そのため重複はないが、Orderは保持されないので注意。
impl Capabilities {
    /// 新しいCapabilities インスタンスを作成
    pub fn new(capabilities: HashSet<CapabilityType>) -> Self {
        Self { capabilities }
    }

    /// Capabilityを追加
    pub fn push(&mut self, capability: CapabilityType) {
        self.capabilities.insert(capability);
    }

    /// 特定のCapabilityをサポートしているか確認
    pub fn supports(&self, capability: &CapabilityType) -> bool {
        self.capabilities.contains(capability)
    }

    /// 複数のCapabilityを全てサポートしているか確認
    pub fn supports_all(&self, capabilities: &[CapabilityType]) -> bool {
        capabilities.iter().all(|c| self.capabilities.contains(c))
    }

    /// 複数のCapabilityのいずれかをサポートしているか確認
    pub fn supports_any(&self, capabilities: &[CapabilityType]) -> bool {
        capabilities.iter().any(|c| self.capabilities.contains(c))
    }

    /// Capabilityの一覧を取得
    pub fn list(&self) -> Vec<&CapabilityType> {
        self.capabilities.iter().collect()
    }

    pub fn or(&self, s: Capabilities) -> Capabilities {
        let mut new_capabilities = self.capabilities.clone();
        new_capabilities.extend(s.capabilities.iter().cloned());
        Capabilities::new(new_capabilities)
    }

    pub(crate) fn default() -> Self {
        Self {
            capabilities: HashSet::new(),
        }
    }
}

/// Capability要件を定義する構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredCapabilities {
    capabilities: HashSet<CapabilityType>,
}

impl RequiredCapabilities {
    pub fn new(capabilities: Vec<CapabilityType>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
        }
    }

    pub fn supported(&self, capabilities: &Capabilities) -> bool {
        capabilities.supports_all(&self.capabilities.iter().cloned().collect::<Vec<_>>())
    }

    pub fn unsupported(&self, capabilities: &Capabilities) -> ProviderResult<()> {
        let unspported: Vec<CapabilityType> = self
            .capabilities
            .iter()
            .filter(|capability| !capabilities.supports(capability))
            .cloned()
            .collect();
        if !unspported.is_empty() {
            return Err(ProviderError::MissingCapabilities(unspported));
        }
        Ok(())
    }

    pub fn capabilities(&self) -> &HashSet<CapabilityType> {
        &self.capabilities
    }
}

#[automock]
pub trait RequiresCapabilities {
    // Providerが必要とする必須Capabilityを返す
    fn required_capabilities(&self) -> RequiredCapabilities;

    /// Providerの要件を検証
    fn validate<P: HasCapabilities + 'static>(&self, provider: &P) -> ProviderResult<()> {
        let missing_capabilities: Vec<CapabilityType> = self
            .required_capabilities()
            .capabilities
            .iter()
            .filter(|capability| !provider.supports(capability))
            .cloned()
            .collect();

        println!("missing_capabilities: {:?}", missing_capabilities);
        if !missing_capabilities.is_empty() {
            return Err(ProviderError::MissingCapabilities(
                missing_capabilities.clone(),
            ));
        }
        Ok(())
    }
}

/// Provider実装用のCapability管理trait
#[automock]
pub trait HasCapabilities {
    /// Capabilitiesを取得
    fn capabilities(&self) -> &Capabilities;

    /// Capabilityをサポートしているか確認
    fn supports(&self, capability: &CapabilityType) -> bool {
        self.capabilities().supports(capability)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // write all test for capalibity and relatied models
    #[test]
    fn test_capabilities() {
        let mut capabilities = Capabilities::default();
        capabilities.push(CapabilityType::Generate);
        capabilities.push(CapabilityType::Policy);

        assert!(capabilities.supports(&CapabilityType::Generate));
        assert!(capabilities.supports(&CapabilityType::Policy));
        assert!(!capabilities.supports(&CapabilityType::Rag));

        let required_capabilities = RequiredCapabilities::new(vec![CapabilityType::Generate]);
        assert!(required_capabilities.supported(&capabilities));

        let required_capabilities =
            RequiredCapabilities::new(vec![CapabilityType::Generate, CapabilityType::Policy]);
        assert!(required_capabilities.supported(&capabilities));

        let required_capabilities =
            RequiredCapabilities::new(vec![CapabilityType::Generate, CapabilityType::Rag]);
        assert!(!required_capabilities.supported(&capabilities));
    }

    #[test]
    fn test_capabilities_or() {
        let mut capabilities = Capabilities::default();
        capabilities.push(CapabilityType::Generate);
        capabilities.push(CapabilityType::Policy);

        let mut capabilities2 = Capabilities::default();
        capabilities2.push(CapabilityType::Rag);
        capabilities2.push(CapabilityType::Search);

        let new_capabilities = capabilities.or(capabilities2);
        assert!(new_capabilities.supports(&CapabilityType::Generate));
        assert!(new_capabilities.supports(&CapabilityType::Policy));
        assert!(new_capabilities.supports(&CapabilityType::Rag));
        assert!(new_capabilities.supports(&CapabilityType::Search));
    }

    #[test]
    fn test_required_capabilities() {
        let required_capabilities =
            RequiredCapabilities::new(vec![CapabilityType::Generate, CapabilityType::Policy]);
        let capabilities =
            Capabilities::from(vec![CapabilityType::Generate, CapabilityType::Policy]);
        assert!(required_capabilities.supported(&capabilities));

        let capabilities = Capabilities::from(vec![CapabilityType::Generate]);
        assert!(!required_capabilities.supported(&capabilities));
    }

    #[test]
    fn test_has_capabilities() {
        struct TestProvider {
            capabilities: Capabilities,
        }

        impl HasCapabilities for TestProvider {
            fn capabilities(&self) -> &Capabilities {
                &self.capabilities
            }
        }

        let provider = TestProvider {
            capabilities: Capabilities::from(vec![
                CapabilityType::Generate,
                CapabilityType::Policy,
            ]),
        };

        assert!(provider.supports(&CapabilityType::Generate));
        assert!(provider.supports(&CapabilityType::Policy));
        assert!(!provider.supports(&CapabilityType::Rag));
    }

    #[test]
    fn test_capability_type() {
        let capability = CapabilityType::Generate;
        let capabilities = Capabilities::from(vec![capability.clone()]);
        assert!(capabilities.supports(&capability));
    }

    #[test]
    fn test_capabilities_from() {
        let capabilities =
            Capabilities::from(vec![CapabilityType::Generate, CapabilityType::Policy]);
        assert!(capabilities.supports(&CapabilityType::Generate));
        assert!(capabilities.supports(&CapabilityType::Policy));
    }

    #[test]
    fn test_capabilities_list() {
        let capabilities =
            Capabilities::from(vec![CapabilityType::Generate, CapabilityType::Policy]);
        let list = capabilities.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_capabilities_push() {
        let mut capabilities = Capabilities::default();
        capabilities.push(CapabilityType::Generate);
        capabilities.push(CapabilityType::Policy);
        assert!(capabilities.supports(&CapabilityType::Generate));
        assert!(capabilities.supports(&CapabilityType::Policy));
    }

    #[test]
    fn test_capabilities_supports_all() {
        let capabilities =
            Capabilities::from(vec![CapabilityType::Generate, CapabilityType::Policy]);
        assert!(capabilities.supports_all(&[CapabilityType::Generate, CapabilityType::Policy]));
        assert!(!capabilities.supports_all(&[CapabilityType::Generate, CapabilityType::Rag]));
    }

    #[test]
    fn test_capabilities_supports_any() {
        let capabilities =
            Capabilities::from(vec![CapabilityType::Generate, CapabilityType::Policy]);
        assert!(capabilities.supports_any(&[CapabilityType::Generate, CapabilityType::Rag]));
        assert!(!capabilities.supports_any(&[CapabilityType::Rag, CapabilityType::Search]));
    }

    #[test]
    fn test_capabilities_default() {
        let capabilities = Capabilities::default();
        assert!(capabilities.list().is_empty());
    }

    #[test]
    fn test_capabilities_new() {
        let capabilities = Capabilities::new(HashSet::from_iter(vec![
            CapabilityType::Generate,
            CapabilityType::Policy,
        ]));
        assert!(capabilities.supports(&CapabilityType::Generate));
        assert!(capabilities.supports(&CapabilityType::Policy));
    }

    #[test]
    fn test_capability_type_custom() {
        let capability = CapabilityType::Custom("custom".to_string());
        let capabilities = Capabilities::from(vec![capability.clone()]);
        assert!(capabilities.supports(&capability));
    }

    #[test]
    fn test_requires_capabilities() {
        struct TestProvider {
            capabilities: Capabilities,
        }

        impl HasCapabilities for TestProvider {
            fn capabilities(&self) -> &Capabilities {
                &self.capabilities
            }
        }

        impl RequiresCapabilities for TestProvider {
            fn required_capabilities(&self) -> RequiredCapabilities {
                RequiredCapabilities::new(vec![CapabilityType::Generate, CapabilityType::Policy])
            }
        }

        let provider = TestProvider {
            capabilities: Capabilities::from(vec![
                CapabilityType::Generate,
                CapabilityType::Policy,
            ]),
        };

        assert!(provider.validate(&provider).is_ok());

        let provider = TestProvider {
            capabilities: Capabilities::from(vec![CapabilityType::Generate]),
        };

        assert!(provider.validate(&provider).is_err());
    }
}
