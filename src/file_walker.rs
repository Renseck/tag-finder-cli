use walkdir::WalkDir;
use std::path::{Path, PathBuf};
use crate::parallel_processor::ParallelProcessor;
use crate::utils::{get_thread_count_or_default};
use crate::config::Config;
use crate::traits::{ThreadCountConfigurable, ConfigConfigurable};
use crate::ProcessorBuilder;

pub struct FileWalker {
    directory: String,
    file_filter: Box<dyn Fn(&Path) -> bool + Send + Sync>,
    thread_count: Option<usize>,
    config: Option<Config>,
}

impl FileWalker {
    pub fn new(directory: String) -> Self {
        Self {
            directory,
            file_filter: Box::new(|_| true),
            thread_count: None,
            config: None,
        }
    }

    /* ========================================================================================== */
    pub fn with_thread_count(mut self, count: usize) -> Self {
        self.thread_count = Some(count);
        self
    }

    /* ========================================================================================== */
    pub fn with_config(mut self, config: Config) -> Self {
        let exclude_dirs = config.scan.exclude_dirs.clone();
        let include_extensions = config.scan.include_extensions.clone();
        let css_extensions = config.scan.css_extensions.clone();
        
        // Combine include and CSS extensions for file filtering
        let all_allowed_extensions = {
            let mut combined = include_extensions.clone();
            combined.extend(css_extensions);
            combined
        };
        
        self.file_filter = Box::new(move |path: &Path| {
            // Check directory exclusions
            for component in path.components() {
                if let Some(dir_name) = component.as_os_str().to_str() {
                    if exclude_dirs.iter().any(|excluded| excluded == dir_name) {
                        return false;
                    }
                }
            }
            
            // Check file extension inclusions
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                all_allowed_extensions.iter().any(|allowed| allowed == ext)
            } else {
                false // Exclude files without extensions
            }
        });

        self.config = Some(config);
        self
    }

    /* ========================================================================================== */
    pub fn walk(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let files: Vec<PathBuf> = WalkDir::new(&self.directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|entry| entry.path().to_path_buf())
            .filter(|path| (self.file_filter)(path))
            .collect();

        Ok(files)
    }

    /* ========================================================================================== */
    pub fn walk_with_content(&self) -> Result<Vec<(PathBuf, String)>, Box<dyn std::error::Error>> {
        let files = self.walk()?;
        let mut results = Vec::new();

        for file in files {
            if let Ok(content) = std::fs::read_to_string(&file) {
                results.push((file, content));
            }
        }

        Ok(results)
    }

    /* ========================================================================================== */
    pub fn walk_with_content_parallel(&self) -> Result<Vec<(PathBuf, String)>, Box<dyn std::error::Error>> {
        let files = self.walk()?;
        println!("📁 Reading {} files using {} threads...", files.len(), get_thread_count_or_default(self.thread_count));

        let processor = ParallelProcessor::new().configure_threads(self.thread_count);
        
        let results = processor.process(
            files,
            |file| -> Result<Option<(PathBuf, String)>, Box<dyn std::error::Error + Send + Sync>> {
                match std::fs::read_to_string(file) {
                    Ok(content) => Ok(Some((file.clone(), content))),
                    Err(_) => Ok(None), // Skip files we can't read
                }
            },
            "Reading files"
        )?;

        Ok(results.into_iter().flatten().collect())
    }
    
    /* ========================================================================================== */
    pub fn with_extensions(mut self, extensions: Vec<&str>) -> Self {
        // Lifetime shittery so do it this way
        let extensions: Vec<String> = extensions.iter().map(|s| s.to_string()).collect();
        self.file_filter = Box::new(move |path: &Path| {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                extensions.iter().any(|e| e == ext)
            } else {
                false
            }
        });
        self
    }

    /* ========================================================================================== */
    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&Path) -> bool + Send + Sync + 'static,
    {
        self.file_filter = Box::new(filter);
        self
    }
}

impl ThreadCountConfigurable for FileWalker {
    fn with_thread_count(mut self, count: usize) -> Self {
        self.thread_count = Some(count);
        self
    }
}

impl ConfigConfigurable for FileWalker {
    fn with_config(mut self, config: Config) -> Self {
        let exclude_dirs = config.scan.exclude_dirs.clone();
        self.file_filter = Box::new(move |path: &Path| {
            for component in path.components() {
                if let Some(dir_name) = component.as_os_str().to_str() {
                    if exclude_dirs.iter().any(|excluded| excluded == dir_name) {
                        return false
                    }
                }
            }
            true
        });

        self.config = Some(config);
        self
    }
}