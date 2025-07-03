use crate::text_processor::TextProcessor;
use crate::config::Config;
use crate::utils::{create_thread_pool, separate_items_by_condition};
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use std::path::PathBuf;

pub struct FileScanner {
    thread_count: Option<usize>,
    config: Option<Config>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub css_files: Vec<String>,
    pub other_files: Vec<String>,
    pub is_css_only: bool,
}

impl FileScanner {
    pub fn new() -> Self {
        Self {
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
        self.config = Some(config);
        self
    }

    /* ========================================================================================== */
    pub fn scan(&self, target_word: String, files_with_content: Vec<(PathBuf, String)>) -> Result<ScanResult, Box<dyn std::error::Error>> {
        let processor = TextProcessor::new();
        
        // Configure thread pool
        let pool = create_thread_pool(self.thread_count)?;

        let results: Vec<ScanFileResult> = pool.install(|| {
            files_with_content
                .par_iter()
                .filter_map(|(file_path, content)| {
                    let has_match = if self.contains_special_chars(&target_word) {
                        // For words with special characters, use simple string search
                        content.contains(&target_word)
                    } else {
                        // For regular words, use exact word matching
                        processor.find_exact_words(content, &target_word)
                    };
                    
                    if has_match {
                        let file_path_str = file_path.to_string_lossy().to_string();
                        let extension = file_path.extension().and_then(|ext| ext.to_str());
                        let is_css = self.is_css_file(extension);
                        
                        Some(ScanFileResult {
                            file_path: file_path_str,
                            is_css,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        });

        self.process_scan_results(results)
    }

    /* ========================================================================================== */
    fn is_css_file(&self, extension: Option<&str>) -> bool {
        if let Some(config) = &self.config {
            extension.map_or(false, |ext| {
                config.scan.css_extensions.iter().any(|css_ext| css_ext == ext)
            })
        } else {
            matches!(extension, Some("css") | Some("scss"))
        }
    }

    /* ========================================================================================== */
    fn process_scan_results(&self, results: Vec<ScanFileResult>) -> Result<ScanResult, Box<dyn std::error::Error>> {
        let (css_results, other_results) = separate_items_by_condition(
            results,
            |result| result.is_css
        );

        let css_files: Vec<String> = css_results.into_iter().map(|r| r.file_path).collect();
        let other_files: Vec<String> = other_results.into_iter().map(|r| r.file_path).collect();

        let is_css_only = !css_files.is_empty() && other_files.is_empty();

        Ok(ScanResult {
            css_files,
            other_files,
            is_css_only,
        })
    }

    /* ========================================================================================== */
    fn contains_special_chars(&self, word: &str) -> bool {
        word.chars().any(|c| !c.is_alphanumeric() && c != '_' && c != '-')
    }

}

// Helper struct for internal processing
#[derive(Debug)]
struct ScanFileResult {
    file_path: String,
    is_css: bool,
}