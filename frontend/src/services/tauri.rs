use crate::types::{UnusedReport, ScanResult};
use serde_json::Value;
use tauri_sys::tauri;
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: &JsValue) -> JsValue;
}


pub struct TauriService;

impl TauriService {
    pub fn select_directory(callback: Callback<Option<String>>) {
        spawn_local(async move {
            match tauri::invoke("select_directory", &Value::Null).await {
                Ok(result) => {
                    let directory: Option<String> = serde_wasm_bindgen::from_value(result)
                        .unwrap_or(None);
                    callback.emit(directory);
                }
                Err(err) => {
                    log::error!("Failed to select directory: {:?}", err);
                    callback.emit(None);
                }
            }
        });
    }

    pub fn analyze_css(directory: String, callback: Callback<Result<UnusedReport, String>>) {
        spawn_local(async move {
            let args = serde_json::json!({ "directory": directory });
            
            match tauri::invoke("analyze_css", &args).await {
                Ok(result) => {
                    match serde_wasm_bindgen::from_value::<UnusedReport>(result) {
                        Ok(report) => callback.emit(Ok(report)),
                        Err(err) => {
                            let error_msg = format!("Failed to parse analysis result: {:?}", err);
                            log::error!("{}", error_msg);
                            callback.emit(Err(error_msg));
                        }
                    }
                }
                Err(err) => {
                    let error_msg = format!("Failed to analyze CSS: {:?}", err);
                    log::error!("{}", error_msg);
                    callback.emit(Err(error_msg));
                }
            }
        });
    }

    pub fn find_word(word: String, directory: String, callback: Callback<Result<ScanResult, String>>) {
        spawn_local(async move {
            let args = serde_json::json!({ 
                "word": word, 
                "directory": directory 
            });
            
            match tauri::invoke("find_word", &args).await {
                Ok(result) => {
                    match serde_wasm_bindgen::from_value::<ScanResult>(result) {
                        Ok(scan_result) => callback.emit(Ok(scan_result)),
                        Err(err) => {
                            let error_msg = format!("Failed to parse search result: {:?}", err);
                            log::error!("{}", error_msg);
                            callback.emit(Err(error_msg));
                        }
                    }
                }
                Err(err) => {
                    let error_msg = format!("Failed to find word: {:?}", err);
                    log::error!("{}", error_msg);
                    callback.emit(Err(error_msg));
                }
            }
        });
    }

    pub fn open_file_at_line(file_path: String, line: usize, callback: Callback<Result<(), String>>) {
        spawn_local(async move {
            let args = serde_json::json!({ 
                "filePath": file_path, 
                "line": line 
            });
            
            match tauri::invoke("open_file_at_line", &args).await {
                Ok(_) => callback.emit(Ok(())),
                Err(err) => {
                    let error_msg = format!("Failed to open file: {:?}", err);
                    log::error!("{}", error_msg);
                    callback.emit(Err(error_msg));
                }
            }
        });
    }
}