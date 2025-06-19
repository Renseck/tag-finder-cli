use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssClass {
    pub name: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedClass {
    pub class: CssClass,
    pub is_unused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedReport {
    pub total_classes: usize,
    pub unused_classes: Vec<CssClass>,
    pub used_classes: Vec<CssClass>,
    pub by_file: std::collections::HashMap<String, Vec<UnusedClass>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub css_files: Vec<String>,
    pub other_files: Vec<String>,
    pub is_css_only: bool,
}

impl UnusedReport {
    pub fn unused_percentage(&self) -> f64 {
        if self.total_classes == 0 {
            0.0
        } else {
            (self.unused_classes.len() as f64 / self.total_classes as f64) * 100.0
        }
    }

    pub fn unused_by_file(&self) -> std::collections::HashMap<String, Vec<UnusedClass>> {
        let mut result = std::collections::HashMap::new();
        
        for (file, classes) in &self.by_file {
            let unused: Vec<UnusedClass> = classes.iter()
                .filter(|c| c.is_unused)
                .cloned()
                .collect();
            
            if !unused.is_empty() {
                result.insert(file.clone(), unused);
            }
        }
        
        result
    }
}