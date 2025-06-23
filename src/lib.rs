pub mod scanner;
pub mod css_parser;
pub mod unused_detector;
pub mod utils;
pub mod file_walker;
pub mod text_processor;
pub mod progress_reporter;

pub use scanner::{FileScanner, ScanResult};
pub use css_parser::*;
pub use unused_detector::*;
pub use utils::*;
pub use file_walker::*;
pub use text_processor::*;
pub use progress_reporter::*;

/* =============================== Some clean wrappers for the GUI ============================== */
pub fn analyze_directory_gui(directory: &str) -> Result<UnusedReport, Box<dyn std::error::Error>> {
    // Detector invokes file walkers as needed
    let detector = UnusedDetector::new(directory.to_string());
    detector.generate_report()
}

/* ============================================================================================== */
pub fn find_word_gui(word: &str, directory: &str) -> Result<ScanResult, Box<dyn std::error::Error>> {
    // Need to manually invoke walker ourselves
    let mut scanner = FileScanner::new();
    let mut walker = FileWalker::new(directory.to_string());
    let threads = None;
    
    if let Some(thread_count) = threads {
        scanner = scanner.with_thread_count(thread_count);
        walker = walker.with_thread_count(thread_count)
    }

    let files_with_content = walker.walk_with_content_parallel()?;

    scanner.scan(word.to_string(), files_with_content)
}