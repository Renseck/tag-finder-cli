use std::collections::HashSet;
use crate::text_processor::{TextProcessor};
use crate::progress_reporter::ProgressReporter;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

pub struct CssParser {
    thread_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssClass {
    pub name: String,
    pub file: String,
    pub line: usize,
}

impl CssParser {
    pub fn new() -> Self {
        Self { 
            thread_count: None,
        }
    }

    /* ========================================================================================== */
    pub fn with_thread_count(mut self, count: usize) -> Self {
        self.thread_count = Some(count);
        self
    }

    /* ========================================================================================== */
    // Currently unused
    pub fn extract_classes(&self, files_with_content: Vec<(PathBuf, String)>) -> Result<Vec<CssClass>, Box<dyn std::error::Error>> {
        let processor = TextProcessor::new()
            .add_pattern("css_class", r"\.([a-zA-Z][a-zA-Z0-9_-]*)")?;
        
        let mut progress = ProgressReporter::new(files_with_content.len(), "Processing".to_string());
        let mut classes = Vec::new();
        
        for (file_path, content) in files_with_content {
            progress.tick();
            
            let matches = processor.process_content(&content);
            let file_path_str = file_path.to_string_lossy().to_string();
            
            for text_match in matches {
                if text_match.pattern_name == "css_class" && self.is_valid_class_name(&text_match.matched_text) {
                    classes.push(CssClass {
                        name: text_match.matched_text,
                        file: file_path_str.clone(),
                        line: text_match.line,
                    });
                }
            }
        }
        
        progress.finish("CSS extraction complete!");
        self.deduplicate_classes(&mut classes);
        Ok(classes)
    }

    /* ========================================================================================== */
    pub fn extract_classes_parallel(&self, files_with_content: Vec<(PathBuf, String)>) -> Result<Vec<CssClass>, Box<dyn std::error::Error>> {
        let processor = Arc::new(
            TextProcessor::new()
                .add_pattern("css_class", r"\.([a-zA-Z][a-zA-Z0-9_-]*)")?
        );
        
        let progress = Arc::new(Mutex::new(
            ProgressReporter::new(files_with_content.len(), "Processing file".to_string())
        ));
        
        // Configure thread pool
        let pool = match self.thread_count {
            Some(count) => rayon::ThreadPoolBuilder::new().num_threads(count).build()?,
            None => rayon::ThreadPoolBuilder::new().build()?,
        };

        let all_classes: Vec<CssClass> = pool.install(|| {
            files_with_content
                .par_iter()
                .flat_map(|(file_path, content)| {
                    // Update progress (thread-safe)
                    if let Ok(mut prog) = progress.lock() {
                        prog.tick();
                    }
                    
                    let matches = processor.process_content(content);
                    let file_path_str = file_path.to_string_lossy().to_string();
                    
                    matches
                        .into_iter()
                        .filter(|text_match| {
                            text_match.pattern_name == "css_class" 
                                && self.is_valid_class_name(&text_match.matched_text)
                        })
                        .map(|text_match| CssClass {
                            name: text_match.matched_text,
                            file: file_path_str.clone(),
                            line: text_match.line,
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        });
        
        if let Ok(prog) = progress.lock() {
            prog.finish("CSS extraction complete!");
        }
        
        let mut classes = all_classes;
        self.deduplicate_classes(&mut classes);
        Ok(classes)
    }

    /* ========================================================================================== */
    fn is_valid_class_name(&self, name: &str) -> bool {
        name.len() >= 2 && !name.chars().all(|c| c.is_ascii_digit())
    }

    /* ========================================================================================== */
    fn deduplicate_classes(&self, classes: &mut Vec<CssClass>) {
        let mut seen = HashSet::new();
        classes.retain(|class| {
            let key = (class.name.clone(), class.file.clone());
            seen.insert(key)
        });
    }
}