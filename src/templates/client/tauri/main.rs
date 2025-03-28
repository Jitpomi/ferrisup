// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};

// Learn more about Tauri commands at https://tauri.app/v2/guide/commands
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Serialize, Deserialize)]
struct AppConfig {
    title: String,
    version: String,
}

fn main() {
    let config = AppConfig {
        title: "Tauri App".to_string(),
        version: "1.0.0".to_string(),
    };

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .manage(config)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
