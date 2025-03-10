use anyhow::Result;
use kairei_core::system::System;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};
use tracing::{debug, error, info, warn};

/// Loader for Kairei DSL files that support the compiler service
#[derive(Clone)]
pub struct DslLoader {
    /// Base directory for DSL files
    base_dir: PathBuf,
    /// System instance to load agents into
    system: Arc<System>,
    /// Dirs to load DSL files from
    sub_dirs: Vec<&'static str>,
}

impl DslLoader {
    /// Create a new DslLoader with a specified base directory
    pub fn new(base_dir: impl AsRef<Path>, system: Arc<System>, sub_dirs: Option<Vec<&'static str>>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            system,
            sub_dirs: sub_dirs.unwrap_or(vec!["validators", "fixers", "assistants"]),
        }
    }

    /// Load all DSL files from a subdirectory
    pub async fn load_directory(&self, subdir: &str) -> Result<Vec<String>> {
        let dir_path = self.base_dir.join(subdir);
        let mut loaded_agents = Vec::new();

        if !dir_path.exists() {
            warn!("Directory does not exist: {:?}", dir_path);
            return Ok(loaded_agents);
        }

        // Find all .kairei files in the directory
        let entries = fs::read_dir(dir_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "kairei") {
                let agent_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                match self.load_dsl_file(&path).await {
                    Ok(_) => {
                        info!("Successfully loaded agent: {}", agent_name);
                        loaded_agents.push(agent_name);
                    }
                    Err(e) => {
                        error!("Failed to load agent {}: {}", agent_name, e);
                    }
                }
            }
        }

        Ok(loaded_agents)
    }

    /// Load a single DSL file
    pub async fn load_dsl_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let start = Instant::now();
        
        // Read DSL file content
        let dsl_content = fs::read_to_string(path)?;
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
        
        debug!("Loading DSL file: {} ({} bytes)", file_name, dsl_content.len());
        
        // Parse the DSL
        self.system.parse_dsl(&dsl_content).await?;
        
        let duration = start.elapsed();
        info!(
            "Successfully loaded DSL file {} in {:?}",
            file_name, duration
        );
        
        Ok(())
    }

    /// Load all DSL files from all subdirectories
    pub async fn load_all(&self) -> Result<Vec<String>> {
        let mut all_agents = Vec::new();
         
        for subdir in &self.sub_dirs {
            match self.load_directory(subdir).await {
                Ok(agents) => {
                    info!("Loaded {} agents from {}", agents.len(), subdir);
                    all_agents.extend(agents);
                }
                Err(e) => {
                    error!("Failed to load directory {}: {}", subdir, e);
                }
            }
        }
        
        info!("Total agents loaded: {}", all_agents.len());
        Ok(all_agents)
    }

    /// Merge multiple DSL files into a single string
    pub fn merge_dsl_files(&self, paths: &[PathBuf]) -> Result<String> {
        let mut merged = String::new();
        
        for path in paths {
            if path.is_file() && path.extension().map_or(false, |ext| ext == "kairei") {
                let content = fs::read_to_string(path)?;
                merged.push_str(&content);
                merged.push_str("\n\n");
            }
        }
        
        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kairei_core::{config::{SecretConfig, SystemConfig}, system::System};
    use std::sync::Arc;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_merge_dsl_files() {
        let temp_dir = tempdir().unwrap();
        
        // Create test DSL files
        let file1_path = temp_dir.path().join("test1.kairei");
        let file2_path = temp_dir.path().join("test2.kairei");
        
        fs::write(&file1_path, "micro Agent1 { policy \"Test\" }").unwrap();
        fs::write(&file2_path, "micro Agent2 { policy \"Test2\" }").unwrap();
        
        // Create loader
        let system = Arc::new(System::new(&SystemConfig::default(), &SecretConfig::default()).await);
        let loader = DslLoader::new(temp_dir.path(), system, None);
        
        // Test merging
        let merged = loader.merge_dsl_files(&[file1_path, file2_path]).unwrap();
        
        assert!(merged.contains("Agent1"));
        assert!(merged.contains("Agent2"));
    }
}