use crate::css_parser::{CssClass, CssParser};
use crate::utils::{print_header_line, print_section_line};
use crate::scanner::FileScanner;
use crate::file_walker::FileWalker;
use crate::config::Config;
use crate::text_processor::{TextProcessor, DynamicPattern};
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub struct UnusedDetector {
    directory: String,
    thread_count: Option<usize>,
    config: Option<Config>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnusedClass {
    pub class: CssClass,
    pub is_unused: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnusedReport {
    pub total_classes: usize,
    pub unused_classes: Vec<CssClass>,
    pub used_classes: Vec<CssClass>,
    pub by_file: HashMap<String, Vec<UnusedClass>>,
}

impl UnusedDetector {
    pub fn new(directory: String) -> Self {
        Self { 
            directory,
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
    pub fn generate_report(&self) -> Result<UnusedReport, Box<dyn std::error::Error>> {
        // Single walker for all operations
        let mut walker = FileWalker::new(self.directory.clone())
            .with_thread_count(self.thread_count.unwrap_or(num_cpus::get()));

        if let Some(config) = &self.config {
            walker = walker.with_config(config.clone());
        }

        // Get files and split
        let all_files_with_content = walker.walk_with_content_parallel()?;
        let css_files_with_content = self.filter_css_files(all_files_with_content.clone());

        // Extract classes
        let classes = self.extract_classes(css_files_with_content)?;

        // Detect dynamic patterns
        let dynamic_patterns = self.detect_patterns(&classes);

        // Check usage status
        let (unused_classes, used_classes, by_file) = self.analyze_class_usage(&classes, all_files_with_content, &dynamic_patterns)?;

        Ok(UnusedReport {
            total_classes: classes.len(),
            unused_classes,
            used_classes,
            by_file,
        })
    }

    /* ========================================================================================== */
    fn filter_css_files(&self, files_with_content: Vec<(PathBuf, String)>) -> Vec<(PathBuf, String)> {
        if let Some(config) = &self.config {
            files_with_content
                .into_iter()
                .filter(|(path, _)| config.is_css_file(path))
                .collect()
                
        } else {
            // Fallback to default CSS extensions if no config
            files_with_content
                .into_iter()
                .filter(|(path, _)| {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        matches!(ext, "css" | "scss")
                    } else {
                        false
                    }
                })
                .collect()
        }
    }

    /* ========================================================================================== */
    fn extract_classes(&self, files_with_content: Vec<(PathBuf, String)>) -> Result<Vec<CssClass>, Box<dyn std::error::Error>> {
        println!("🔍 Extracting CSS classes...");
        let css_parser = CssParser::new()
            .with_thread_count(self.thread_count.unwrap_or(num_cpus::get()));
        let classes = css_parser.extract_classes_parallel(files_with_content)?;
        println!("📊 Found {} CSS classes. Checking usage...", classes.len());
        Ok(classes)
    }

    /* ========================================================================================== */
    fn detect_patterns(&self, classes: &[CssClass]) -> Vec<DynamicPattern> {
        println!("🔍 Detecting dynamic patterns...");
        let processor = TextProcessor::new();
        let class_names: Vec<String> = classes.iter().map(|c| c.name.clone()).collect();
        let patterns = processor.detect_dynamic_patterns(&class_names);
        
        if !patterns.is_empty() {
            println!("📊 Found {} dynamic patterns:", patterns.len());
            for pattern in &patterns {
                println!("   {} (covers {} classes)", pattern.pattern, pattern.matching_classes.len());
            }
        }
        
        patterns
    }

    /* ========================================================================================== */
    fn analyze_class_usage(
        &self,
        classes: &[CssClass],
        all_files_with_content: Vec<(PathBuf, String)>,
        dynamic_patterns: &[DynamicPattern],
    ) -> Result<(Vec<CssClass>, Vec<CssClass>, HashMap<String, Vec<UnusedClass>>), Box<dyn std::error::Error>> {

        let progress_counter = Arc::new(Mutex::new(0usize));
        let total = classes.len();
        let files_arc = Arc::new(all_files_with_content);
        let patterns_arc = Arc::new(dynamic_patterns.to_vec());
        
        // Configure thread pool
        let pool = match self.thread_count {
            Some(count) => rayon::ThreadPoolBuilder::new().num_threads(count).build()?,
            None => rayon::ThreadPoolBuilder::new().build()?,
        };

        println!("🔍 Analyzing {} classes using {} threads...", total, pool.current_num_threads());
        println!("   Step 1: Checking exact matches...");
        let exact_match_results: Result<Vec<_>, Box<dyn std::error::Error + Send + Sync>> = pool.install(|| {
            classes
                .par_iter()
                .map(|class| -> Result<UnusedClass, Box<dyn std::error::Error + Send + Sync>> {
                    // Update progress
                    {
                        let mut counter = progress_counter.lock().unwrap();
                        *counter += 1;
                        if *counter % 25 == 0 {
                            println!("   Processed {}/{} classes...", *counter, total);
                        }
                    }

                    let is_unused = self.is_class_unused_exact(class, &files_arc)?;
                    Ok(UnusedClass {
                        class: class.clone(),
                        is_unused,
                    })
                })
                .collect()
        });

        let exact_results = exact_match_results.map_err(|e| -> Box<dyn std::error::Error> { 
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        // Separate classes into used and potentially unused
        let mut used_classes = Vec::new();
        let mut potentially_unused_classes = Vec::new();

        for unused_class in exact_results {
            if !unused_class.is_unused {
                used_classes.push(unused_class.class);
            } else {
                potentially_unused_classes.push(unused_class.class);
            }
        }

        println!("   Step 1 complete: {} used via exact match, {} need pattern check", 
            used_classes.len(), potentially_unused_classes.len());

        // Step 2: Only check dynamic patterns for classes that weren't found via exact match
        let mut unused_classes = Vec::new();
        
        if !potentially_unused_classes.is_empty() && !dynamic_patterns.is_empty() {
            println!("   Step 2: Checking dynamic patterns for remaining {} classes...", potentially_unused_classes.len());
            
            let pattern_results: Result<Vec<_>, Box<dyn std::error::Error + Send + Sync>> = pool.install(|| {
                potentially_unused_classes
                    .par_iter()
                    .map(|class| -> Result<(CssClass, bool), Box<dyn std::error::Error + Send + Sync>> {
                        let is_used_via_pattern = self.is_class_unused_dynamic(class, &files_arc, &patterns_arc)?;
                        Ok((class.clone(), is_used_via_pattern))
                    })
                    .collect()
            });

            let pattern_results = pattern_results.map_err(|e| -> Box<dyn std::error::Error> { 
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
            })?;

            for (class, is_used_via_pattern) in pattern_results {
                if is_used_via_pattern {
                    used_classes.push(class);
                } else {
                    unused_classes.push(class);
                }
            }
        } else {
            // No patterns to check, all potentially unused classes are actually unused
            unused_classes = potentially_unused_classes;
        }

        // Build the by_file structure
        let mut by_file: HashMap<String, Vec<UnusedClass>> = HashMap::new();
        
        for class in &used_classes {
            by_file
                .entry(class.file.clone())
                .or_default()
                .push(UnusedClass {
                    class: class.clone(),
                    is_unused: false,
                });
        }
        
        for class in &unused_classes {
            by_file
                .entry(class.file.clone())
                .or_default()
                .push(UnusedClass {
                    class: class.clone(),
                    is_unused: true,
                });
        }

        println!("✅ Analysis complete!");
        Ok((unused_classes, used_classes, by_file))
    }

    /* ========================================================================================== */
    fn is_class_unused_exact(&self, class: &CssClass, files_with_content: &Arc<Vec<(PathBuf, String)>>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // First try regular scanning for exact matches (fastest)
        let scanner = FileScanner::new();
        let result = scanner.scan(class.name.clone(), files_with_content.to_vec())
            .map_err(|e| format!("Scanner error: {}", e))?;
        Ok(result.is_css_only)
    }

    /* ========================================================================================== */
    fn is_class_unused_dynamic(&self, class: &CssClass, files_with_content: &Arc<Vec<(PathBuf, String)>>, dynamic_patterns: &Arc<Vec<DynamicPattern>>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        for pattern in dynamic_patterns.iter() {
            if pattern.matching_classes.contains(&class.name) {
                // Check if the pattern is used in any file
                let processor = TextProcessor::new();
                for (_, content) in files_with_content.iter() {
                    if processor.find_pattern_usage(content, pattern) {
                        return Ok(true); // Class is used via pattern
                    }
                }
            }
        }
        Ok(false)
    }
}

impl UnusedReport {
    pub fn print_summary(&self) {
        println!("\n📋 UNUSED CSS CLASSES REPORT");
        print_header_line(50);
        println!("Total classes analyzed: {}", self.total_classes);
        println!("Unused classes: {}", self.unused_classes.len());
        println!("Used classes: {}", self.used_classes.len());
        
        if self.total_classes > 0 {
            let percentage = (self.unused_classes.len() as f64 / self.total_classes as f64) * 100.0;
            println!("Unused percentage: {:.1}%", percentage);
        }
    }
    /* ========================================================================================== */
    
    pub fn print_detailed(&self) {
        self.print_summary();
        
        if self.unused_classes.is_empty() {
            return;
        }
        
        println!("\n🗑️  UNUSED CLASSES:");
        print_section_line(30);
        
        self.print_unused_classes_by_file();
        println!("\n💡 TIP: Review these unused classes and consider removing them to clean up your CSS.");
    }
    /* ========================================================================================== */

    pub fn print_by_file(&self) {
        self.print_summary();
        println!("\n📁 BY FILE BREAKDOWN:");
        print_section_line(40);
        
        let mut files: Vec<_> = self.by_file.keys().collect();
        files.sort();
        
        for file in files {
            self.print_file_breakdown(file);
        }
    }
    /* ========================================================================================== */

    fn print_unused_classes_by_file(&self) {
        let mut files: Vec<_> = self.by_file.keys().collect();
        files.sort();
        
        for file in files {
            let unused_in_file = self.get_unused_classes_in_file(file);
            
            if unused_in_file.is_empty() {
                continue;
            }
            
            println!("\n📁 {}:", file);
            for unused in unused_in_file {
                println!("   .{} (line {})", unused.class.name, unused.class.line);
            }
        }
    }
    /* ========================================================================================== */

    fn print_file_breakdown(&self, file: &str) {
        let classes = &self.by_file[file];
        let unused_count = classes.iter().filter(|c| c.is_unused).count();
        let total_count = classes.len();
        
        println!("\n{}", file);
        println!("  Total: {}, Unused: {}, Used: {}", 
            total_count, unused_count, total_count - unused_count);
        
        if unused_count == 0 {
            return;
        }
        
        println!("  Unused classes:");
        for class in classes.iter().filter(|c| c.is_unused) {
            println!("    .{} (line {})", class.class.name, class.class.line);
        }
    }
    /* ========================================================================================== */

    fn get_unused_classes_in_file(&self, file: &str) -> Vec<&UnusedClass> {
        self.by_file[file]
            .iter()
            .filter(|c| c.is_unused)
            .collect()
    }
    /* ========================================================================================== */
}