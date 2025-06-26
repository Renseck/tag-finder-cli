use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub scan: ScanConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScanConfig {
    #[serde(default = "default_exclude_dirs")]
    pub exclude_dirs: Vec<String>,
    #[serde(default = "default_include_extensions")]
    pub include_extensions: Vec<String>,
    #[serde(default = "default_css_extensions")]
    pub css_extensions: Vec<String>,
}

/* =================================== Default value functions ================================== */

fn default_exclude_dirs() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".git".to_string(),
        ".vscode".to_string(),
        ".idea".to_string(),
        "target".to_string(),
    ]
}

fn default_include_extensions() -> Vec<String> {
    vec![
        "html".to_string(),
        "js".to_string(),
        "jsx".to_string(),
        "ts".to_string(),
        "tsx".to_string(),
        "php".to_string(),
    ]
}

fn default_css_extensions() -> Vec<String> {
    vec![
        "css".to_string(),
        "scss".to_string(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan: ScanConfig {
                exclude_dirs: default_exclude_dirs(),
                include_extensions: default_include_extensions(),
                css_extensions: default_css_extensions(),
            },
        }
    }
}

/* ==================================== Config implementation =================================== */

impl Config {
    /* =================================== Load from file path ================================== */
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /* ========================================================================================== */
    pub fn from_file_or_default(path: &str) -> Self {
        match Self::from_file(path) {
            Ok(config) => {
                println!("Loaded configuration from {}", path);
                config
            },
            Err(_) => {
                println!("Using default configurating (no config file found)");
                Self::default()
            }
        }
    }

    /* =========================== Automatically find configs and load ========================== */
    pub fn find_config_file() -> Option<String> {
        let possible_paths = [
            "tag-finder.toml",
            ".tag-finder.toml",
            "config/tag-finder.toml",
        ];

        for path in &possible_paths {
            if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }

        None
    }

    /* ========================================================================================== */
    pub fn load_or_default() -> Self {
        if let Some(config_path) = Self::find_config_file() {
            Self::from_file_or_default(&config_path)
        } else {
            println!("No config file found, using defaults");
            Self::default()
        }
    }

    /* =================================== Scanning functions =================================== */
    // ?Should these be in the FileWalker?
    pub fn should_exclude_dir(&self, dir_name: &str) -> bool {
        self.scan.exclude_dirs.iter().any(|excluded| {
            dir_name == excluded || dir_name.starts_with(&format!("{}/", excluded))
        })
    }

    /* ========================================================================================== */
    pub fn should_include_file(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            self.scan.include_extensions.iter().any(|allowed| allowed == ext)
        } else {
            false
        }
    }

    /* ========================================================================================== */
    pub fn is_css_file(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            self.scan.css_extensions.iter().any(|css_ext| css_ext == ext)
        } else {
            false
        }
    }
}