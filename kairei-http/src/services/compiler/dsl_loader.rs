use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::{debug, error, info, warn};

/// Loader for Kairei DSL files that support the compiler service
#[derive(Debug, Clone)]
pub struct DslLoader {
    /// Base directory for DSL files
    base_dir: PathBuf,
    /// Dirs to load DSL files from
    sub_dirs: Vec<&'static str>,
    /// Loaded agents
    pub agents: Vec<String>,
}

impl Default for DslLoader {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("dsl"),
            sub_dirs: vec!["validators", "fixers", "assistants"],
            agents: Vec::new(),
        }
    }
}

impl DslLoader {
    /// Create a new DslLoader with a specified base directory
    pub fn new(base_dir: impl AsRef<Path>, sub_dirs: Option<Vec<&'static str>>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            sub_dirs: sub_dirs.unwrap_or_default(),
            ..Default::default()
        }
    }

    /// Load all DSL files from a subdirectory
    pub fn load_directory(&mut self, subdir: &str) -> Result<Vec<String>> {
        let dir_path = self.base_dir.join(subdir);

        if !dir_path.exists() {
            warn!("Directory does not exist: {:?}", dir_path);
            return Ok(self.agents.clone());
        }

        // Find all .kairei files in the directory
        let entries = fs::read_dir(dir_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "kairei") {
                let agent_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                match self.load_dsl_file(&path) {
                    Ok(dsl) => {
                        info!("Successfully loaded agent: {}", agent_name);
                        self.agents.push(dsl);
                    }
                    Err(e) => {
                        error!("Failed to load agent {}: {}", agent_name, e);
                    }
                }
            }
        }

        Ok(self.agents.clone())
    }

    /// Load a single DSL file
    pub fn load_dsl_file(&self, path: impl AsRef<Path>) -> Result<String> {
        let path = path.as_ref();

        // Read DSL file content
        let dsl_content = fs::read_to_string(path)?;
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        debug!("Loaded {} bytes from {}", dsl_content.len(), file_name);

        Ok(dsl_content)
    }

    /// Load all DSL files from all subdirectories
    pub fn load_all(&mut self) -> Result<Self> {
        for subdir in &self.sub_dirs.clone() {
            match self.load_directory(subdir) {
                Ok(agents) => {
                    info!("Loaded {} agents from {}", agents.len(), subdir);
                    self.agents.extend(agents);
                }
                Err(e) => {
                    error!("Failed to load directory {}: {}", subdir, e);
                }
            }
        }

        info!("Total agents loaded: {}", self.agents.len());
        Ok(self.clone())
    }

    /// Merge multiple DSL files into a single string
    pub fn merge_dsl_files(&self) -> String {
        let mut merged = String::new();

        for agent in self.agents.clone() {
            merged.push_str(&agent);
            merged.push_str("\n\n");
        }

        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let mut loader = DslLoader::new(temp_dir.path(), Some(vec!["./"]));

        // Test merging
        let merged = loader.load_all().unwrap().merge_dsl_files();

        assert!(merged.contains("Agent1"));
        assert!(merged.contains("Agent2"));
    }
}
