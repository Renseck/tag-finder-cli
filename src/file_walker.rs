use walkdir::WalkDir;
use std::path::{Path, PathBuf};
use rayon::prelude::*;

pub struct FileWalker {
    directory: String,
    file_filter: Box<dyn Fn(&Path) -> bool + Send + Sync>,
    thread_count: Option<usize>,
}

impl FileWalker {
    pub fn new(directory: String) -> Self {
        Self {
            directory,
            file_filter: Box::new(|_| true),
            thread_count: None,
        }
    }

    /* ========================================================================================== */
    pub fn with_thread_count(mut self, count: usize) -> Self {
        self.thread_count = Some(count);
        self
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
    pub fn walk_with_content_parallel(&self) -> Result<Vec<(PathBuf, String)>, Box<dyn std::error::Error>> {
        // Configure thread pool
        let pool = match self.thread_count {
            Some(count) => rayon::ThreadPoolBuilder::new().num_threads(count).build()?,
            None => rayon::ThreadPoolBuilder::new().build()?,
        };

        let files = self.walk()?;
        println!("📁 Reading {} files using {} threads...", files.len(), pool.current_num_threads());

        let results: Result<Vec<_>, Box<dyn std::error::Error + Send + Sync>> = pool.install(|| {
            files
                .par_iter()
                .map(|file| -> Result<Option<(PathBuf, String)>, Box<dyn std::error::Error + Send + Sync>> {
                    match std::fs::read_to_string(file) {
                        Ok(content) => Ok(Some((file.clone(), content))),
                        Err(_) => Ok(None), // Skip files we can't read
                    }
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|vec| vec.into_iter().flatten().collect())
        });

        results.map_err(|e| -> Box<dyn std::error::Error> { 
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
         })
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