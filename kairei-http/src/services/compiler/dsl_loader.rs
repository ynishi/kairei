use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tracing::{debug, error, info, warn};

/// Represents a loaded DSL file with metadata
#[derive(Debug, Clone)]
pub struct DslFile {
    /// Name of the agent (derived from filename)
    pub name: String,
    /// Type of the agent (derived from subdirectory)
    pub agent_type: String,
    /// Full path to the DSL file
    pub path: PathBuf,
    /// Content of the DSL file
    pub content: String,
}

/// Configuration for the DSL loader
#[derive(Debug, Clone)]
pub struct DslLoaderConfig {
    /// Base directory for DSL files
    pub base_dir: PathBuf,
    /// File extension to look for
    pub file_extension: String,
}

impl Default for DslLoaderConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("dsl"),
            file_extension: "kairei".to_string(),
        }
    }
}

/// Loader for Kairei DSL files that support the compiler service
#[derive(Debug, Clone, Default)]
pub struct DslLoader {
    /// Configuration for the loader
    config: DslLoaderConfig,
}

impl DslLoader {
    /// Create a new DslLoader with a specified configuration
    pub fn new(config: DslLoaderConfig) -> Self {
        Self { config }
    }

    /// Create a new DslLoader with a specified base directory
    pub fn with_base_dir(base_dir: impl AsRef<Path>) -> Self {
        Self::new(DslLoaderConfig {
            base_dir: base_dir.as_ref().to_path_buf(),
            ..Default::default()
        })
    }

    /// Auto-detect subdirectories in the base directory, ignoring those starting with a dot
    pub fn discover_subdirectories(&self) -> Result<Vec<String>> {
        let base_dir = &self.config.base_dir;
        debug!("Discovering subdirectories in: {:?}", base_dir);

        if !base_dir.exists() {
            warn!("Base directory does not exist: {:?}", base_dir);
            return Ok(Vec::new());
        }

        let mut subdirs = Vec::new();
        let entries = fs::read_dir(base_dir)
            .with_context(|| format!("Failed to read base directory: {:?}", base_dir))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip directories that start with a dot
                    if !dir_name.starts_with('.') {
                        subdirs.push(dir_name.to_string());
                        debug!("Discovered subdirectory: {}", dir_name);
                    } else {
                        debug!("Skipping hidden directory: {}", dir_name);
                    }
                }
            }
        }

        info!("Discovered {} subdirectories", subdirs.len());
        Ok(subdirs)
    }

    /// Load all DSL files from all auto-detected subdirectories
    pub fn load_all(&self) -> Result<Vec<DslFile>> {
        let subdirs = self.discover_subdirectories()?;
        let mut all_files = Vec::new();

        for subdir in &subdirs {
            match self.load_directory(subdir) {
                Ok(mut files) => {
                    info!("Loaded {} DSL files from {}", files.len(), subdir);
                    all_files.append(&mut files);
                }
                Err(e) => {
                    error!("Failed to load directory {}: {}", subdir, e);
                }
            }
        }

        info!("Total DSL files loaded: {}", all_files.len());
        Ok(all_files)
    }

    /// Load all DSL files from a specific subdirectory
    pub fn load_directory(&self, subdir: &str) -> Result<Vec<DslFile>> {
        let dir_path = self.config.base_dir.join(subdir);
        debug!("Loading DSL files from directory: {:?}", dir_path);

        if !dir_path.exists() {
            warn!("Directory does not exist: {:?}", dir_path);
            return Ok(Vec::new());
        }

        let mut dsl_files = Vec::new();
        let entries = fs::read_dir(&dir_path)
            .with_context(|| format!("Failed to read directory: {:?}", dir_path))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            debug!("Examining path: {:?}", path);

            if path.is_file() && self.is_dsl_file(&path) {
                let agent_name = self.extract_file_stem(&path);

                match self.load_dsl_file(&path) {
                    Ok(content) => {
                        info!("Successfully loaded agent: {}", agent_name);
                        dsl_files.push(DslFile {
                            name: agent_name,
                            agent_type: subdir.to_string(),
                            path: path.clone(),
                            content,
                        });
                    }
                    Err(e) => {
                        error!("Failed to load agent {} from {:?}: {}", agent_name, path, e);
                    }
                }
            }
        }

        debug!("Loaded {} DSL files from {}", dsl_files.len(), subdir);
        Ok(dsl_files)
    }

    /// Check if a file is a DSL file based on its extension
    fn is_dsl_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == self.config.file_extension)
            .unwrap_or(false)
    }

    /// Extract the file stem (filename without extension) from a path
    fn extract_file_stem(&self, path: &Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Load a single DSL file
    pub fn load_dsl_file(&self, path: impl AsRef<Path>) -> Result<String> {
        let path = path.as_ref();
        let file_name = self.extract_file_stem(path);

        // Read DSL file content
        let dsl_content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read DSL file: {:?}", path))?;

        debug!("Loaded {} bytes from {}", dsl_content.len(), file_name);
        Ok(dsl_content)
    }

    /// Group DSL files by their type (subdirectory)
    pub fn group_by_type(dsl_files: &[DslFile]) -> HashMap<String, Vec<&DslFile>> {
        let mut grouped = HashMap::new();

        for file in dsl_files {
            grouped
                .entry(file.agent_type.clone())
                .or_insert_with(Vec::new)
                .push(file);
        }

        grouped
    }

    /// Merge multiple DSL files into a single string
    pub fn merge_dsl_files(dsl_files: &[DslFile]) -> String {
        let mut merged = String::new();

        for file in dsl_files {
            // Add a comment with the file name for better traceability
            merged.push_str(&format!("// From: {}\n", file.path.display()));
            merged.push_str(&file.content);
            merged.push_str("\n\n");
        }

        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_dsl_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.kairei");
        let content = "micro Agent1 { policy \"Test\" }";

        fs::write(&file_path, content).unwrap();

        let loader = DslLoader::with_base_dir(temp_dir.path());
        let loaded = loader.load_dsl_file(&file_path).unwrap();

        assert_eq!(loaded, content);
    }

    #[test]
    fn test_load_directory() {
        let temp_dir = tempdir().unwrap();
        let subdir = "validators";
        let subdir_path = temp_dir.path().join(subdir);

        fs::create_dir(&subdir_path).unwrap();

        let file1_path = subdir_path.join("test1.kairei");
        let file2_path = subdir_path.join("test2.kairei");
        let non_dsl_path = subdir_path.join("not_dsl.txt");

        fs::write(&file1_path, "micro Agent1 { policy \"Test\" }").unwrap();
        fs::write(&file2_path, "micro Agent2 { policy \"Test2\" }").unwrap();
        fs::write(&non_dsl_path, "This is not a DSL file").unwrap();

        let loader = DslLoader::with_base_dir(temp_dir.path());
        let files = loader.load_directory(subdir).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.name == "test1"));
        assert!(files.iter().any(|f| f.name == "test2"));
    }

    #[test]
    fn test_discover_subdirectories() {
        let temp_dir = tempdir().unwrap();

        // Create regular directories
        fs::create_dir(temp_dir.path().join("validators")).unwrap();
        fs::create_dir(temp_dir.path().join("fixers")).unwrap();

        // Create a hidden directory
        fs::create_dir(temp_dir.path().join(".hidden")).unwrap();

        let loader = DslLoader::with_base_dir(temp_dir.path());
        let subdirs = loader.discover_subdirectories().unwrap();

        assert_eq!(subdirs.len(), 2);
        assert!(subdirs.contains(&"validators".to_string()));
        assert!(subdirs.contains(&"fixers".to_string()));
        assert!(!subdirs.contains(&".hidden".to_string()));
    }

    #[test]
    fn test_load_all() {
        let temp_dir = tempdir().unwrap();

        // Create subdirectories
        let validators_dir = temp_dir.path().join("validators");
        let fixers_dir = temp_dir.path().join("fixers");
        let hidden_dir = temp_dir.path().join(".hidden");

        fs::create_dir(&validators_dir).unwrap();
        fs::create_dir(&fixers_dir).unwrap();
        fs::create_dir(&hidden_dir).unwrap();

        // Create DSL files
        fs::write(
            validators_dir.join("validator1.kairei"),
            "micro Validator1 { policy \"Test\" }",
        )
        .unwrap();

        fs::write(
            fixers_dir.join("fixer1.kairei"),
            "micro Fixer1 { policy \"Test\" }",
        )
        .unwrap();

        fs::write(
            hidden_dir.join("hidden.kairei"),
            "micro Hidden { policy \"Test\" }",
        )
        .unwrap();

        let loader = DslLoader::with_base_dir(temp_dir.path());
        let all_files = loader.load_all().unwrap();

        assert_eq!(all_files.len(), 2);
        assert!(
            all_files
                .iter()
                .any(|f| f.name == "validator1" && f.agent_type == "validators")
        );
        assert!(
            all_files
                .iter()
                .any(|f| f.name == "fixer1" && f.agent_type == "fixers")
        );
        assert!(!all_files.iter().any(|f| f.name == "hidden"));
    }

    #[test]
    fn test_merge_dsl_files() {
        let dsl_files = vec![
            DslFile {
                name: "test1".to_string(),
                agent_type: "validators".to_string(),
                path: PathBuf::from("/path/to/test1.kairei"),
                content: "micro Agent1 { policy \"Test\" }".to_string(),
            },
            DslFile {
                name: "test2".to_string(),
                agent_type: "validators".to_string(),
                path: PathBuf::from("/path/to/test2.kairei"),
                content: "micro Agent2 { policy \"Test2\" }".to_string(),
            },
        ];

        let merged = DslLoader::merge_dsl_files(&dsl_files);

        assert!(merged.contains("Agent1"));
        assert!(merged.contains("Agent2"));
        assert!(merged.contains("// From: /path/to/test1.kairei"));
        assert!(merged.contains("// From: /path/to/test2.kairei"));
    }

    #[test]
    fn test_group_by_type() {
        let dsl_files = vec![
            DslFile {
                name: "test1".to_string(),
                agent_type: "validators".to_string(),
                path: PathBuf::from("/path/to/test1.kairei"),
                content: "micro Agent1 { policy \"Test\" }".to_string(),
            },
            DslFile {
                name: "test2".to_string(),
                agent_type: "fixers".to_string(),
                path: PathBuf::from("/path/to/test2.kairei"),
                content: "micro Agent2 { policy \"Test2\" }".to_string(),
            },
            DslFile {
                name: "test3".to_string(),
                agent_type: "validators".to_string(),
                path: PathBuf::from("/path/to/test3.kairei"),
                content: "micro Agent3 { policy \"Test3\" }".to_string(),
            },
        ];

        let grouped = DslLoader::group_by_type(&dsl_files);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get("validators").unwrap().len(), 2);
        assert_eq!(grouped.get("fixers").unwrap().len(), 1);
    }
}
