use std::fs;
use std::path::Path;

/* ============================================================================================== */
pub fn print_header_line(width: usize) {
    println!("{spacer:=>width$}", spacer="=", width = width);
}

/* ============================================================================================== */
pub fn print_section_line(width: usize) {
    println!("{spacer:->width$}", spacer="-", width = width);
}

/* ============================================================================================== */
pub fn print_banner(banner_file: Option<&str>) {
    // Read banner from file and yeet it out
    let banner_content = match banner_file {
        Some(file_path) => read_banner_from_file(file_path),
        None => read_banner_from_file("banner.txt"),
    };
    
    match banner_content {
        Ok(content) => {
            println!("{}", content);
            let max_width = get_max_line_length(&content);
            print_header_line(max_width); // Add a separator line after banner
        }
        Err(_) => {
            // Fallback to default banner if file not found
            print_default_banner();
        }
    }
}

/* ============================================================================================== */
fn read_banner_from_file(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    if !Path::new(file_path).exists() {
        return Err("Banner file not found".into());
    }
    
    let content = fs::read_to_string(file_path)?;
    Ok(content.trim_end().to_string()) 
}

/* ============================================================================================== */
fn print_default_banner() {
    println!(r#"
╔════════════════════════════════════════════════════════╗
║                    🎯 TAG FINDER 🎯                    ║
║                                                        ║
║            Find unused CSS classes and tags            ║
║              Clean up your codebase! 🧹               ║
╚════════════════════════════════════════════════════════╝
    "#);
    print_section_line(60);
}

/* ============================================================================================== */
fn get_max_line_length(content: &str) -> usize {
    content
        .lines()
        .map(|line| {
            // Count visible characters, ignoring ANSI escape codes and Unicode box drawing
            line.chars()
                .filter(|c| !c.is_control() && *c != '\u{001b}') // Filter control chars
                .count()
        })
        .max()
        .unwrap_or(60) // Default to 60 if somehow empty
}