use std::collections::HashSet;
use crate::text_processor::{TextProcessor};
use crate::parallel_processor::ParallelProcessor;
use crate::ProcessorBuilder;
use crate::traits::ThreadCountConfigurable;
use serde::{Deserialize, Serialize};
use std::sync::{Arc};
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
    pub fn extract_classes_parallel(&self, files_with_content: Vec<(PathBuf, String)>) -> Result<Vec<CssClass>, Box<dyn std::error::Error>> {
        let processor_arc = Arc::new(
            TextProcessor::new()
                .add_pattern("css_class", r"\.([a-zA-Z][a-zA-Z0-9_-]*)")?
        );

        let parallel_processor = ParallelProcessor::new().configure_threads(self.thread_count);
        
        let all_classes = parallel_processor.process_flat_map(
            files_with_content,
            |(file_path, content)| {
                let matches = processor_arc.process_content(content);
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
            },
            "Processing files for CSS classes"
        )?;
        
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

impl ThreadCountConfigurable for CssParser {
    fn with_thread_count(mut self, count: usize) -> Self {
        self.thread_count = Some(count);
        self
    }
}