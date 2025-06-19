use walkdir::WalkDir;
use std::path::{Path, PathBuf};

pub struct FileWalker {
    directory: String,
    file_filter: Box<dyn Fn(&Path) -> bool>,
}

impl FileWalker {
    pub fn new(directory: String) -> Self {
        Self {
            directory,
            file_filter: Box::new(|_| true),
        }
    }

    /* ========================================================================================== */
    pub fn walk(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(&self.directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if (self.file_filter)(path) {
                files.push(path.to_path_buf());
            }
        }

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
        F: Fn(&Path) -> bool + 'static,
    {
        self.file_filter = Box::new(filter);
        self
    }
}