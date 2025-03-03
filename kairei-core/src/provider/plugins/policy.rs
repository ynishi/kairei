use async_trait::async_trait;

use crate::{
    PolicyScope,
    provider::{
        capability::CapabilityType,
        llm::LLMResponse,
        plugin::{PluginContext, ProviderPlugin},
        provider::Section,
        types::ProviderResult,
    },
};

/// ポリシープラグインの定義
pub struct PolicyPlugin;

#[async_trait]
impl ProviderPlugin for PolicyPlugin {
    fn priority(&self) -> i32 {
        10 // ポリシーは早めに適用
    }

    #[tracing::instrument(skip(self, context))]
    async fn generate_section<'a>(&self, context: &PluginContext<'a>) -> ProviderResult<Section> {
        let mut policy_section = String::new();
        let policies = context.request.state.policies.clone();

        let world_policies = policies
            .iter()
            .filter(|p| matches!(p.scope, PolicyScope::World(_)))
            .collect::<Vec<_>>();
        if !world_policies.is_empty() {
            policy_section.push_str("Global Policies:\n");
            for policy in world_policies {
                policy_section.push_str(&format!("- {}\n", policy.text));
            }
        }

        let agent_policies = policies
            .iter()
            .filter(|p| matches!(p.scope, PolicyScope::Agent(_)))
            .collect::<Vec<_>>();

        if !agent_policies.is_empty() {
            policy_section.push_str("\nAgent-Specific Policies:\n");
            for policy in agent_policies {
                policy_section.push_str(&format!("- {}\n", policy.text));
            }
        }

        let think_policies = policies
            .iter()
            .filter(|p| matches!(p.scope, PolicyScope::Think))
            .collect::<Vec<_>>();

        if !think_policies.is_empty() {
            policy_section.push_str("\nThink-Specific Policies:\n");
            for policy in think_policies {
                policy_section.push_str(&format!("- {}\n", policy.text));
            }
        }

        Ok(Section {
            content: policy_section,
            priority: 10,
            metadata: Default::default(),
        })
    }

    fn capability(&self) -> CapabilityType {
        CapabilityType::PolicyPrompt
    }

    async fn process_response<'a>(
        &self,
        _context: &PluginContext<'a>,
        _response: &LLMResponse,
    ) -> ProviderResult<()> {
        // ポリシーの後処理は不要
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::{ast::*, provider::plugins::provider_tests::TestContextHolder};

    fn create_test_plugin() -> PolicyPlugin {
        PolicyPlugin
    }

    #[tokio::test]
    async fn test_policy_section_generation() -> ProviderResult<()> {
        let plugin = create_test_plugin();
        let mut context_holder = TestContextHolder::new("test request");

        // テスト用ポリシーの作成
        let policies = vec![
            Policy {
                text: "Global Policy 1".to_string(),
                scope: PolicyScope::World("test".to_string()),
                internal_id: PolicyId(Uuid::new_v4().to_string()),
            },
            Policy {
                text: "Agent Policy 1".to_string(),
                scope: PolicyScope::Agent("agent1".to_string()),
                internal_id: PolicyId(Uuid::new_v4().to_string()),
            },
            Policy {
                text: "Think Policy 1".to_string(),
                scope: PolicyScope::Think,
                internal_id: PolicyId(Uuid::new_v4().to_string()),
            },
        ];

        // コンテキストにポリシーを設定
        context_holder.request.state.policies = policies;
        let context = context_holder.get_plugin_context();

        // セクションの生成
        let section = plugin.generate_section(&context).await?;

        // 各種ポリシーが含まれていることを確認
        assert!(section.content.contains("Global Policies:"));
        assert!(section.content.contains("Global Policy 1"));
        assert!(section.content.contains("Agent-Specific Policies:"));
        assert!(section.content.contains("Agent Policy 1"));
        assert!(section.content.contains("Think-Specific Policies:"));
        assert!(section.content.contains("Think Policy 1"));

        // プラグインの優先度を確認
        assert_eq!(section.priority, 10);

        Ok(())
    }

    #[tokio::test]
    async fn test_empty_policies() -> ProviderResult<()> {
        let plugin = create_test_plugin();
        let context_holder = TestContextHolder::new("test request");
        let context = context_holder.get_plugin_context();

        let section = plugin.generate_section(&context).await?;

        // ポリシーが空の場合、セクションも空になることを確認
        assert!(section.content.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_plugin_capabilities() {
        let plugin = create_test_plugin();
        assert_eq!(plugin.capability(), CapabilityType::PolicyPrompt);
        assert_eq!(plugin.priority(), 10);
    }

    #[tokio::test]
    async fn test_process_response() -> ProviderResult<()> {
        let plugin = create_test_plugin();
        let context_holder = TestContextHolder::new("test request");
        let context = context_holder.get_plugin_context();
        let response = LLMResponse::default();

        // process_responseは何もしないことを確認
        let result = plugin.process_response(&context, &response).await;
        assert!(result.is_ok());
        Ok(())
    }
}
