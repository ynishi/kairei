use kairei_core::{
    config::{SecretConfig, SystemConfig},
    event_bus::{EventError, RequestBuilder, Value},
    system::{System, SystemError},
};
use std::sync::Arc;
use uuid::Uuid;

use super::DslLoader;

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

    pub async fn initialize(&mut self, is_load: bool) -> Result<(), SystemError> {
        let mut system = System::new(&self.config, &self.secret_config).await;
        if is_load {
            let dsl = self
                .dsl_loader
                .load_all()
                .map_err(|err| SystemError::Initialization(err.to_string()))?
                .merge_dsl_files();
            let root = system.parse_dsl(&dsl).await?;
            system.initialize(root).await?;
        }
        self.system = Some(Arc::new(system));
        Ok(())
    }

    pub async fn validate_dsl(&self, code: &str) -> Result<String, SystemError> {
        if let Some(system) = self.system.clone() {
            let parsed = system.parse_dsl(code).await;
            println!("parsed: {:?}", parsed);
            match parsed {
                Ok(_) => Ok("Successfully parsed DSL".to_string()),
                Err(err) => {
                    let agents = system
                        .get_system_status()
                        .await
                        .map(|status| status.agent_count)
                        .map_err(|err| SystemError::Initialization(err.to_string()))?;
                    if agents == 0 {
                        return Err(SystemError::Initialization(
                            "Failed to Parse, No agents found".to_string(),
                        ));
                    }
                    let request = RequestBuilder::new()
                        .request_id(&Uuid::new_v4().to_string())
                        .request_type("ValidateDsl")
                        .requester("manager")
                        .responder("ValidatorAgent")
                        .parameter("code", &Value::String(code.to_string()))
                        .parameter("error", &Value::String(err.to_string()))
                        .build()
                        .map_err(SystemError::from)?;

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
                        kairei_core::event_bus::Value::String(s) => Ok(s),
                        _ => Err(SystemError::Event(EventError::ReceiveFailed {
                            message: "unsupported response type".to_string(),
                        })),
                    }
                }
            }
        } else {
            Err(SystemError::Initialization(
                "system not initialized".to_string(),
            ))
        }
    }
}
