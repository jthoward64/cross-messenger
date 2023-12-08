// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use emulated::bindings::ValidationDataError;

pub mod emulated;
pub mod imessage;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_validation_data() -> Result<Vec<u8>, ValidationDataError> {
    let retval = emulated::bindings::generate_validation_data();
    println!("get_validation_data: {:?}", retval);
    retval
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, get_validation_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
