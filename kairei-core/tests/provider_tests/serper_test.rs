use kairei_core::{
    config::SearchConfig,
    provider::{
        plugin::ProviderPlugin, plugins::web_search_serper::WebSearchPlugin,
        provider::ProviderSecret,
    },
};

use crate::{TestContextHolder, provider_tests::setup_config, should_run_external_api_tests};

const WEB_SEARCH_TEST_PROVIDER: &str = "web_search_test";

#[tokio::test]
async fn test_web_search() {
    if !should_run_external_api_tests() {
        return;
    }

    let (_system_config, secret_config) = setup_config();

    let plugin = WebSearchPlugin::new(
        &SearchConfig::default(),
        &ProviderSecret::from(
            secret_config
                .providers
                .get(WEB_SEARCH_TEST_PROVIDER)
                .unwrap()
                .to_owned(),
        ),
    );
    let context_holder = TestContextHolder::new("What is Rust programming language?");
    let context = context_holder.get_plugin_context();

    let section = plugin.generate_section(&context).await.unwrap();

    assert!(!section.content.is_empty());
    assert!(section.content.contains("Rust"));
    // 結果の構造を確認
    assert!(section.content.contains("=============START==========="));
    assert!(section.content.contains("TITLE:"));
    assert!(section.content.contains("URL:"));
}
