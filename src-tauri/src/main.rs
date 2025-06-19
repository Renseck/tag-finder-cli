// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri::command;
use tag_finder::{analyze_directory_gui, find_word_gui, UnusedReport, ScanResult};

#[derive(Serialize, Deserialize)]
struct AnalysisProgress {
  current: usize,
  total: usize,
  message: String,
}

/* ============================================================================================== */
#[command]
async fn analyze_css(directory: String) -> Result<UnusedReport, String> {
  println!("Analyzing directory: {}", directory);
  analyze_directory_gui(&directory).map_err(|e| e.to_string())
}

/* ============================================================================================== */
#[command]
async fn find_word(word: String, directory: String) -> Result<ScanResult, String> {
    println!("Finding word '{}' in directory: {}", word, directory);
    find_word_gui(&word, &directory).map_err(|e| e.to_string())
}

/* ============================================================================================== */
#[command]
fn select_directory() -> Result<Option<String>, String> {
    use std::path::PathBuf;
    
    // Use rfd (native file dialog) which is more reliable
    let result: Option<PathBuf> = rfd::FileDialog::new()
        .set_title("Select Project Directory")
        .pick_folder();
    
    match result {
        Some(path) => Ok(Some(path.to_string_lossy().to_string())),
        None => Ok(None),
    }
}

/* ============================================================================================== */
#[command]
async fn open_file_at_line(file_path: String, line: usize) -> Result<(), String> {
  // This will attempt to open the file in the default editor
  // You can customize this based on user preferences later
  println!("Opening {} at line {}", file_path, line);

  // Just in case this requires specific cases
  #[cfg(target_os = "windows")]
  {
      std::process::Command::new("code")
          .arg(&file_path)
          .arg("--goto")
          .arg(format!("{}:{}", file_path, line))
          .spawn()
          .map_err(|e| format!("Failed to open file: {}", e))?;
  }

  #[cfg(target_os = "macos")]
  {
      std::process::Command::new("code")
          .arg(&file_path)
          .arg("--goto")
          .arg(format!("{}:{}", file_path, line))
          .spawn()
          .map_err(|e| format!("Failed to open file: {}", e))?;
  }

  #[cfg(target_os = "linux")]
  {
      std::process::Command::new("code")
          .arg(&file_path)
          .arg("--goto")
          .arg(format!("{}:{}", file_path, line))
          .spawn()
          .map_err(|e| format!("Failed to open file: {}", e))?;
  }

  Ok(())
}

/* ============================================================================================== */
fn main() {
  tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![
          analyze_css,
          find_word,
          select_directory,
          open_file_at_line
      ])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}
