use std::collections::HashSet;
use crate::file_walker::FileWalker;
use crate::text_processor::{TextProcessor};
use crate::progress_reporter::ProgressReporter;
use serde::{Deserialize, Serialize};

pub struct CssParser {
    directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssClass {
    pub name: String,
    pub file: String,
    pub line: usize,
}

impl CssParser {
    pub fn new(directory: String) -> Self {
        Self { directory }
    }

    /* ========================================================================================== */
    pub fn extract_classes(&self) -> Result<Vec<CssClass>, Box<dyn std::error::Error>> {
        let walker = FileWalker::new(self.directory.clone())
            .with_extensions(vec!["css", "scss"]);
        
        let files_with_content = walker.walk_with_content()?;
        
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