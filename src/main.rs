use clap::{Parser, Subcommand};
use tag_finder::{print_header_line, FileScanner, UnusedDetector, print_banner};

#[derive(Parser)]
#[command(name = "tag-finder")]
#[command(about = "Find unused classes in CSS/SCSS files")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Find a specific word that appears only in CSS/SCSS files
    FindWord {
        /// The word to search for (exact match)
        #[arg(short, long)]
        word: String,
        
        /// Directory to search in
        #[arg(short, long, default_value = ".")]
        directory: String,
        
        /// Show all matches, not just CSS-only ones
        #[arg(short, long)]
        all: bool,
    },
    /// Analyze all CSS classes and find unused ones
    UnusedClasses {
        /// Directory to analyze
        #[arg(short, long, default_value = ".")]
        directory: String,
        
        /// Show detailed breakdown by file
        #[arg(short, long)]
        by_file: bool,
        
        /// Show full detailed report
        #[arg(long)]
        detailed: bool,
    },
}

/* ============================================================================================== */
fn main() {
    let args = Args::parse();

    print_banner(Some("banner.txt"));
    
    match args.command {
        Commands::FindWord { word, directory, all } => {
            if let Err(e) = handle_find_word(word, directory, all) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::UnusedClasses { directory, by_file, detailed } => {
            if let Err(e) = handle_unused_classes(directory, by_file, detailed) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/* ============================================================================================== */
fn handle_find_word(word: String, directory: String, all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let scanner = FileScanner::new(word.clone(), directory);
    let result = scanner.scan()?;
    
    if should_show_results(&result, all) {
        print_word_search_results(&word, &result);
    } else if has_non_css_matches(&result) {
        println!("Word '{}' found but not CSS-only. Use --all to see details.", word);
    } else {
        println!("Word '{}' not found in any files.", word);
    }
    
    Ok(())
}

/* ============================================================================================== */
fn handle_unused_classes(directory: String, by_file: bool, detailed: bool) -> Result<(), Box<dyn std::error::Error>> {
    let detector = UnusedDetector::new(directory);
    let report = detector.generate_report()?;
    
    match (detailed, by_file) {
        (true, _) => report.print_detailed(),
        (false, true) => report.print_by_file(),
        (false, false) => print_summary_with_preview(&report),
    }
    
    Ok(())
}

/* ============================================================================================== */
fn should_show_results(result: &tag_finder::ScanResult, all: bool) -> bool {
    all || result.is_css_only
}

/* ============================================================================================== */
fn has_non_css_matches(result: &tag_finder::ScanResult) -> bool {
    !result.is_css_only && (!result.css_files.is_empty() || !result.other_files.is_empty())
}

/* ============================================================================================== */
fn print_word_search_results(word: &str, result: &tag_finder::ScanResult) {
    println!("Search results for word: '{}'", word);
    print_header_line(50);
    
    if !result.css_files.is_empty() {
        println!("Found in CSS/SCSS files:");
        for file in &result.css_files {
            println!("  ✓ {}", file);
        }
    }
    
    if !result.other_files.is_empty() {
        println!("Found in other files:");
        for file in &result.other_files {
            println!("  • {}", file);
        }
    }
    
    print_word_search_conclusion(word, result);
}

/* ============================================================================================== */
fn print_word_search_conclusion(word: &str, result: &tag_finder::ScanResult) {
    if result.is_css_only {
        println!("\n🎯 SUCCESS: '{}' appears ONLY in CSS/SCSS files!", word);
        println!("This code might be extraneous and safe to remove.");
    } else if result.css_files.is_empty() && result.other_files.is_empty() {
        println!("\n❌ Word '{}' not found in any files.", word);
    } else {
        println!("\n⚠️  Word '{}' appears in non-CSS files too.", word);
    }
}

/* ============================================================================================== */
fn print_summary_with_preview(report: &tag_finder::UnusedReport) {
    report.print_summary();
    
    if report.unused_classes.is_empty() {
        return;
    }
    
    println!("\n🗑️  UNUSED CLASSES (first 10):");
    for class in report.unused_classes.iter().take(10) {
        println!("  .{} in {} (line {})", class.name, class.file, class.line);
    }
    
    if report.unused_classes.len() > 10 {
        println!("  ... and {} more", report.unused_classes.len() - 10);
        println!("\nUse --detailed for full list or --by-file for file breakdown");
    }
}