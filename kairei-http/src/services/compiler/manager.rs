use kairei_core::{
    config::{SecretConfig, SystemConfig},
    event_bus::{EventError, RequestBuilder, Value},
    system::{System, SystemError},
};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use super::{DslLoader, DslSplitter};

/// Manager for System instances used by the compiler service
#[derive(Clone, Default)]
pub struct CompilerSystemManager {
    config: SystemConfig,
    secret_config: SecretConfig,
    system: Option<Arc<System>>,
    dsl_loader: DslLoader,
}

impl CompilerSystemManager {
    /// Create a new CompilerSystemManager with custom configurations
    pub fn new(
        config: SystemConfig,
        secret_config: SecretConfig,
        dsl_loader: Option<DslLoader>,
    ) -> Self {
        Self {
            config,
            secret_config,
            system: None,
            dsl_loader: dsl_loader.unwrap_or_default(),
        }
    }

    pub async fn initialize(&mut self, is_load: bool) -> Result<(), CompilerError> {
        let mut system = System::new(&self.config, &self.secret_config).await;
        if is_load {
            let dsl_files = self
                .dsl_loader
                .load_all()
                .map_err(|e| CompilerError::InitializationError(e.to_string()))?;
            let dsl = DslLoader::merge_dsl_files(&dsl_files);
            let root = system.parse_dsl(&dsl).await?;
            system.initialize(root).await?;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            system.start().await?;
        }
        self.system = Some(Arc::new(system));
        Ok(())
    }

    pub async fn validate_dsl(&self, code: &str) -> Result<String, CompilerError> {
        if let Some(system) = self.system.clone() {
            let parsed = system.parse_dsl(code).await;
            println!("parsed: {:?}", parsed);
            match parsed {
                Ok(_) => Ok("Successfully parsed DSL".to_string()),
                Err(err) => {
                    let splitted = self.split_dsl_blocks(code);
                    let splitted_one_tier = self.split_dsl_blocks_one_tier(code);
                    let agents = system
                        .get_system_status()
                        .await
                        .map(|status| status.agent_count)
                        .map_err(|err| CompilerError::InitializationError(err.to_string()))?;
                    if agents == 0 {
                        return Err(CompilerError::InitializationError(
                            "Failed to Parse, No agents found".to_string(),
                        ));
                    }
                    let request = RequestBuilder::new()
                        .request_id(&Uuid::new_v4().to_string())
                        .request_type("Validate")
                        .requester("manager")
                        .responder("Validator")
                        .parameter("code", &Value::String(code.to_string()))
                        .parameter(
                            "stateInput",
                            &Value::String(
                                splitted
                                    .get("state")
                                    .unwrap_or(&vec!["Not Found".to_string()])
                                    .join("\n"),
                            ),
                        )
                        .parameter(
                            "answerInput",
                            &Value::String(
                                splitted
                                    .get("answer")
                                    .unwrap_or(&vec!["Not Found".to_string()])
                                    .join("\n"),
                            ),
                        )
                        .parameter(
                            "reactInput",
                            &Value::String(
                                splitted
                                    .get("react")
                                    .unwrap_or(&vec!["Not Found".to_string()])
                                    .join("\n"),
                            ),
                        )
                        .parameter(
                            "microInput",
                            &Value::String(
                                splitted
                                    .get("micro")
                                    .unwrap_or(&vec!["Not Found".to_string()])
                                    .join("\n"),
                            ),
                        )
                        .parameter(
                            "policiesInput",
                            &Value::String(
                                splitted_one_tier
                                    .get("micro")
                                    .unwrap_or(&vec!["Not Found".to_string()])
                                    .join("\n"),
                            ),
                        )
                        .parameter(
                            "observeInput",
                            &Value::String(
                                splitted
                                    .get("observe")
                                    .unwrap_or(&vec!["Not Found".to_string()])
                                    .join("\n"),
                            ),
                        )
                        .parameter(
                            "lifecycleInput",
                            &Value::String(format!(
                                "{}\n{}",
                                splitted
                                    .get("onInit")
                                    .unwrap_or(&vec!["Not Found".to_string()])
                                    .join("\n"),
                                splitted
                                    .get("onEnd")
                                    .unwrap_or(&vec!["Not Found".to_string()])
                                    .join("\n")
                            )),
                        )
                        .parameter("parseError", &Value::String(err.to_string()))
                        .build()?;

                    let (tx, rx) = tokio::sync::oneshot::channel();
                    let request_clone = request.clone();

                    tokio::spawn(async move {
                        let result = match system.send_request(request_clone).await {
                            Ok(result) => result,
                            Err(e) => {
                                tracing::error!("Failed to request agent: {}", e);
                                // Default value or appropriate error representation
                                Value::Null
                            }
                        };

                        // Send the result back (ignore errors if receiver dropped)
                        let _ = tx.send(result);
                    });

                    // Later, receive the result
                    let response = match rx.await {
                        Ok(result) => result,
                        Err(_) => {
                            tracing::error!("Channel closed before receiving result");
                            Value::Null
                        }
                    };

                    match response {
                        kairei_core::event_bus::Value::Null => {
                            Err(CompilerError::RequestError("Request failed".to_string()))
                        }
                        _ => Err(CompilerError::CompilationError {
                            message: "Check Completed".to_string(),
                            errors: vec![err.to_string()],
                            suggestions: vec![Self::extract_output(serde_json::Value::from(
                                &response,
                            ))],
                        }),
                    }
                }
            }
        } else {
            Err(CompilerError::InitializationError(
                "system not initialized".to_string(),
            ))
        }
    }

    fn extract_output(value: serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s,
            serde_json::Value::Object(obj) => {
                if let Some(output) = obj.get("output") {
                    Self::extract_output(output.clone())
                } else {
                    obj.iter()
                        .map(|(k, v)| format!("{}: {}", k, Self::extract_output(v.clone())))
                        .collect::<Vec<String>>()
                        .join("\n")
                }
            }
            _ => value.to_string(),
        }
    }

    /// Split DSL code into blocks by keyword
    ///
    /// # Arguments
    /// * `code` - The DSL code to split
    ///
    /// # Returns
    /// A HashMap with keywords as keys and lists of related blocks as values
    pub fn split_dsl_blocks(&self, code: &str) -> HashMap<String, Vec<String>> {
        let splitter = DslSplitter::new();
        splitter.split_dsl_blocks(code)
    }

    /// Split DSL code into one-tier blocks (only top-level elements within a specified block)
    ///
    /// # Arguments
    /// * `code` - The DSL code to split
    /// * `parent_block` - The parent block to extract from (e.g., "micro")
    ///
    /// # Returns
    /// A HashMap with keywords as keys and lists of related blocks as values
    pub fn split_dsl_blocks_one_tier(&self, code: &str) -> HashMap<String, Vec<String>> {
        let splitter = DslSplitter::new();
        let mut acc_map = HashMap::new();
        for keywords in [
            "policy", "state", "answer", "observe", "onInit", "onEnd", "on", "think", "await",
            "world",
        ] {
            let parent_block = keywords;
            let map = splitter.split_dsl_blocks_one_tier(code, parent_block);
            for (key, value) in map {
                acc_map.insert(key, value);
            }
        }
        acc_map
    }
}

/// custom error of CompilerSystemManager using thiserror
#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    #[error("Failed to parse DSL: {0}")]
    ParseError(#[from] SystemError),
    #[error(
        "Compilation error with suggest: {message}, errors: {errors:?}, suggestions: {suggestions:?}"
    )]
    CompilationError {
        message: String,
        errors: Vec<String>,
        suggestions: Vec<String>,
    },
    #[error("Failed to initialize system: {0}")]
    InitializationError(String),
    #[error("Event error: {0}")]
    EventError(#[from] EventError),
    #[error("Request error: {0}")]
    RequestError(String),
}
