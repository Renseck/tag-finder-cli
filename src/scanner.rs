use crate::file_walker::FileWalker;
use crate::text_processor::TextProcessor;
use serde::{Deserialize, Serialize};

pub struct FileScanner {
    target_word: String,
    directory: String,
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
            directory
        }
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

}