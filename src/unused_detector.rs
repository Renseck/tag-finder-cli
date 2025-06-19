use crate::css_parser::{CssClass, CssParser};
use crate::{print_header_line, print_section_line};
use crate::scanner::FileScanner;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
pub struct UnusedDetector {
    directory: String,
}

#[derive(Debug, Serialize, Deserialize)]
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
        Self { directory }
    }

    /* ========================================================================================== */
    pub fn generate_report(&self) -> Result<UnusedReport, Box<dyn std::error::Error>> {
        let classes = self.extract_all_classes()?;
        let (unused_classes, used_classes, by_file) = self.analyze_class_usage(&classes)?;
        
        Ok(UnusedReport {
            total_classes: classes.len(),
            unused_classes,
            used_classes,
            by_file,
        })
    }

    /* ========================================================================================== */
    fn extract_all_classes(&self) -> Result<Vec<CssClass>, Box<dyn std::error::Error>> {
        println!("🔍 Extracting CSS classes...");
        let css_parser = CssParser::new(self.directory.clone());
        let classes = css_parser.extract_classes()?;
        println!("📊 Found {} CSS classes. Checking usage...", classes.len());
        Ok(classes)
    }
    /* ========================================================================================== */

    fn analyze_class_usage(
        &self,
        classes: &[CssClass],
    ) -> Result<(Vec<CssClass>, Vec<CssClass>, HashMap<String, Vec<UnusedClass>>), Box<dyn std::error::Error>> {
        let mut unused_classes = Vec::new();
        let mut used_classes = Vec::new();
        let mut by_file: HashMap<String, Vec<UnusedClass>> = HashMap::new();
        
        for (i, class) in classes.iter().enumerate() {
            self.print_progress(i, classes.len());
            
            let is_unused = self.is_class_unused(class)?;
            let unused_class = UnusedClass {
                class: class.clone(),
                is_unused,
            };
            
            by_file
                .entry(class.file.clone())
                .or_default()
                .push(unused_class);
            
            if is_unused {
                unused_classes.push(class.clone());
            } else {
                used_classes.push(class.clone());
            }
        }
        
        println!("✅ Analysis complete!");
        Ok((unused_classes, used_classes, by_file))
    }
    /* ========================================================================================== */

    fn is_class_unused(&self, class: &CssClass) -> Result<bool, Box<dyn std::error::Error>> {
        let scanner = FileScanner::new(class.name.clone(), self.directory.clone());
        let result = scanner.scan()?;
        Ok(result.is_css_only)
    }
    /* ========================================================================================== */

    fn print_progress(&self, current: usize, total: usize) {
        if current % 10 == 0 && current > 0 {
            println!("   Processed {}/{} classes...", current, total);
        }
    }
    /* ========================================================================================== */
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