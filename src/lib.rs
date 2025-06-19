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
    let detector = UnusedDetector::new(directory.to_string());
    detector.generate_report_parallel()
}

/* ============================================================================================== */
pub fn find_word_gui(word: &str, directory: &str) -> Result<ScanResult, Box<dyn std::error::Error>> {
    let scanner = FileScanner::new(word.to_string(), directory.to_string());
    scanner.scan_parallel()
}