use crate::file_walker::FileWalker;
use crate::text_processor::TextProcessor;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;

pub struct FileScanner {
    target_word: String,
    directory: String,
    thread_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub css_files: Vec<String>,
    pub other_files: Vec<String>,
    pub is_css_only: bool,
}

impl FileScanner {
    pub fn new(target_word: String, directory: String) -> Self {
        Self {
            target_word,
            directory,
            thread_count: None,
        }
    }

    /* ========================================================================================== */
    pub fn with_thread_count(mut self, count: usize) -> Self {
        self.thread_count = Some(count);
        self
    }

    /* ========================================================================================== */
    pub fn scan(&self) -> Result<ScanResult, Box<dyn std::error::Error>> {
        let walker = FileWalker::new(self.directory.clone());
        let files_with_content = walker.walk_with_content()?;
        let processor = TextProcessor::new();
        
        let mut css_files = Vec::new();
        let mut other_files = Vec::new();

        for (file_path, content) in files_with_content {
            if processor.find_exact_words(&content, &self.target_word) {
                let file_path_str = file_path.to_string_lossy().to_string();
                let extension = file_path.extension().and_then(|ext| ext.to_str());
                
                match extension {
                    Some("css") | Some("scss") => css_files.push(file_path_str),
                    _ => other_files.push(file_path_str),
                }
            }
        }

        let is_css_only = !css_files.is_empty() && other_files.is_empty();

        Ok(ScanResult {
            css_files,
            other_files,
            is_css_only,
        })
    }

    /* ========================================================================================== */
    pub fn scan_parallel(&self) -> Result<ScanResult, Box<dyn std::error::Error>> {
        let walker = FileWalker::new(self.directory.clone())
            .with_thread_count(self.thread_count.unwrap_or(num_cpus::get()));
        
        let files_with_content = walker.walk_with_content_parallel()?;
        let processor = TextProcessor::new();
        
        // Configure thread pool
        let pool = match self.thread_count {
            Some(count) => rayon::ThreadPoolBuilder::new().num_threads(count).build()?,
            None => rayon::ThreadPoolBuilder::new().build()?,
        };

        let results: Vec<(String, bool)> = pool.install(|| {
            files_with_content
                .par_iter()
                .filter_map(|(file_path, content)| {
                    if processor.find_exact_words(content, &self.target_word) {
                        let file_path_str = file_path.to_string_lossy().to_string();
                        let extension = file_path.extension().and_then(|ext| ext.to_str());
                        let is_css = matches!(extension, Some("css") | Some("scss"));
                        Some((file_path_str, is_css))
                    } else {
                        None
                    }
                })
                .collect()
        });

        let mut css_files = Vec::new();
        let mut other_files = Vec::new();
        
        for (file_path, is_css) in results {
            if is_css {
                css_files.push(file_path);
            } else {
                other_files.push(file_path);
            }
        }

        let is_css_only = !css_files.is_empty() && other_files.is_empty();

        Ok(ScanResult {
            css_files,
            other_files,
            is_css_only,
        })
    }

}