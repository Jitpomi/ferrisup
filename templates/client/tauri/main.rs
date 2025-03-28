#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};

// Command to get application info
#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        name: "FerrisUp App".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "A Tauri application created with FerrisUp".to_string(),
    }
}

#[derive(Serialize, Deserialize)]
struct AppInfo {
    name: String,
    version: String,
    description: String,
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_app_info])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
