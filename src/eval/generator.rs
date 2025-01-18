use async_trait::async_trait;

use crate::Policy;

use super::evaluator::EvalResult;
#[async_trait]
pub trait PromptGenerator: Send + Sync {
    async fn generate_prompt(
        &self,
        user_content: String,
        policies: &[Policy],
        meta: PromptMeta,
    ) -> EvalResult<Prompt>;
}

#[derive(Debug)]
pub struct Prompt {
    pub system: String,
    pub user: String,
}

pub struct PromptMeta {
    pub agent_name: String,
}

pub struct StandardPromptGenerator;

impl Default for StandardPromptGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl StandardPromptGenerator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PromptGenerator for StandardPromptGenerator {
    async fn generate_prompt(
        &self,
        user_content: String,
        policies: &[Policy],
        meta: PromptMeta,
    ) -> EvalResult<Prompt> {
        let mut system = format!("You are acting as {}.\n", meta.agent_name);

        for policy in policies {
            system.push_str(&policy.text);
            system.push('\n');
        }

        // TODO: when metadata added, include prompt.
        Ok(Prompt {
            system,
            user: user_content,
        })
    }
}
