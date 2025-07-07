use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

/* ============================================================================================== */
/*                                          Process utils                                         */
/* ============================================================================================== */
pub fn create_thread_pool(thread_count: Option<usize>) -> Result<rayon::ThreadPool, Box<dyn std::error::Error>> {
    let pool = match thread_count {
        Some(count) => rayon::ThreadPoolBuilder::new().num_threads(count).build()?,
        None => rayon::ThreadPoolBuilder::new().build()?,
    };
    Ok(pool)
}

/* ============================================================================================== */
pub fn separate_items_by_condition<T, F>(items: Vec<T>, condition: F) -> (Vec<T>, Vec<T>) 
where
    F: Fn(&T) -> bool,
{
    let mut true_items = Vec::new();
    let mut false_items = Vec::new();
    
    for item in items {
        if condition(&item) {
            true_items.push(item);
        } else {
            false_items.push(item);
        }
    }
    
    (true_items, false_items)
}

/* ============================================================================================== */
pub fn calculate_progress_step_size(total: usize, target_updates: usize) -> usize {
    std::cmp::max(1, total / target_updates)
}

/* ============================================================================================== */
pub fn get_thread_count_or_default(thread_count: Option<usize>) -> usize {
    thread_count.unwrap_or_else(num_cpus::get)
}
/* ============================================================================================== */
/*                                        Collection utils                                        */
/* ============================================================================================== */
pub fn extract_first_element<T, U>(items: Vec<(T, U)>) -> Vec<T> {
    items.into_iter().map(|(first, _)| first).collect()
}

/* ============================================================================================== */
pub fn extract_second_element<T, U>(items: Vec<(T, U)>) -> Vec<U> {
    items.into_iter().map(|(_, second)| second).collect()
}

/* ============================================================================================== */
pub fn filter_and_extract<T, U, F>(items: Vec<(T, U)>, predicate: F) -> Vec<T>
where
    F: Fn(&(T, U)) -> bool,
{
    items.into_iter()
        .filter(|item| predicate(item))
        .map(|(first, _)| first)
        .collect()
}
/* ============================================================================================== */
/*                                         File utils                                         */
/* ============================================================================================== */
pub fn has_extension(path: &std::path::Path, extensions: &[&str]) -> bool {
    if let Some(ext) = get_file_extension(path) {
        extensions.iter().any(|allowed| *allowed == ext)
    } else {
        false
    }
}

/* ============================================================================================== */
pub fn get_file_extension(path: &std::path::Path) -> Option<&str> {
    path.extension().and_then(|ext| ext.to_str())
}

/* ============================================================================================== */
/*                                         Printing utils                                         */
/* ============================================================================================== */
pub fn update_progress(progress_counter: &Arc<Mutex<usize>>, total: usize, step_size: usize) {
    let mut counter = progress_counter.lock().unwrap();
    *counter += 1;
    if *counter % step_size == 0 || *counter == total {
        println!("      Processed {}/{} items...", *counter, total);
    }
}

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

/* ============================================================================================== */
/*                                      Error handling utils                                      */
/* ============================================================================================== */
pub fn convert_sync_error<E: std::error::Error + Send + Sync + 'static>(
    error: E
) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
}

/* ============================================================================================== */
pub fn convert_thread_error<E: std::error::Error + Send + Sync + 'static>(
    error: E
) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
}