use async_trait::async_trait;

use crate::provider::types::{ProviderResult, Section};

pub type OrderedListSections = Vec<(usize, Section)>;

#[async_trait]
#[mockall::automock]
pub trait Optimizer: Send + Sync {
    async fn optimize(&self, prompt: String) -> ProviderResult<String>;
    async fn optimize_sections(
        &self,
        sections: OrderedListSections,
    ) -> ProviderResult<OrderedListSections>;
}

// PromptOptimizer impl Optimizer
pub struct PromptOptimizer;

// basic impl Optimizer for PromptOptimizer
#[async_trait]
impl Optimizer for PromptOptimizer {
    async fn optimize(&self, sections: String) -> ProviderResult<String> {
        Ok(sections)
    }

    async fn optimize_sections(
        &self,
        sections: OrderedListSections,
    ) -> ProviderResult<OrderedListSections> {
        Ok(sections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_optimize() {
        let optimizer = PromptOptimizer;
        let prompt = "This is a test prompt".to_string();
        let optimized = optimizer.optimize(prompt).await.unwrap();
        assert_eq!(optimized, "This is a test prompt");
    }

    #[tokio::test]
    async fn test_optimize_sections() {
        let optimizer = PromptOptimizer;
        let sections = vec![(1, Section::new("This is a test section"))];
        let optimized = optimizer.optimize_sections(sections).await.unwrap();
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].1.content, "This is a test section");
    }
}
