use async_trait::async_trait;

use crate::provider::{provider::Section, types::ProviderResult};

use super::optimizer::Optimizer;

#[async_trait]
pub trait Generator: Send + Sync {
    async fn generate(&self, sections: Vec<Section>) -> ProviderResult<String>;
}

pub struct PromptGenerator {
    optimizer: Option<Box<dyn Optimizer>>,
}

#[async_trait]
impl Generator for PromptGenerator {
    async fn generate(&self, sections: Vec<Section>) -> ProviderResult<String> {
        let mut prompt = String::new();
        for section in sections {
            prompt.push_str(&format!("{}\n\n", section.to_string()));
        }

        if let Some(optimizer) = &self.optimizer {
            optimizer.optimize(prompt).await
        } else {
            Ok(prompt)
        }
    }
}

impl PromptGenerator {
    pub fn new(optimizer: Option<Box<dyn Optimizer>>) -> Self {
        Self { optimizer }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        provider::{generator::optimizer::OrderedListSections, provider::SectionMetadata},
        timestamp::Timestamp,
    };

    struct MockOptimizer;

    #[async_trait]
    impl Optimizer for MockOptimizer {
        async fn optimize(&self, prompt: String) -> ProviderResult<String> {
            Ok(prompt)
        }

        // optimize sections
        async fn optimize_sections(
            &self,
            sections: OrderedListSections,
        ) -> ProviderResult<OrderedListSections> {
            Ok(sections)
        }
    }

    #[tokio::test]
    async fn test_generate() {
        let generator = PromptGenerator::new(Some(Box::new(MockOptimizer {})));

        let sections = vec![
            Section {
                content: "section1".to_string(),
                priority: 1,
                metadata: SectionMetadata {
                    source: "source1".to_string(),
                    timestamp: Timestamp::now(),
                },
            },
            Section {
                content: "section2".to_string(),
                priority: 2,
                metadata: SectionMetadata {
                    source: "source2".to_string(),
                    timestamp: Timestamp::now(),
                },
            },
        ];

        let prompt = generator.generate(sections).await.unwrap();
        assert_eq!(prompt, "section1\n\nsection2\n\n")
    }
}
