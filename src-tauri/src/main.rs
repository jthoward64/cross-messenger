// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::actions::init::authenticate;
use std::sync::Arc;

use emulated::bindings::ValidationDataError;
use rustpush::APNSConnection;
use state::ApplicationState;
use tokio::sync::Mutex;

pub mod actions;
pub mod emulated;
pub mod imessage;
pub mod state;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_validation_data() -> Result<String, ValidationDataError> {
    let retval = emulated::bindings::generate_validation_data();
    println!("get_validation_data: {:?}", retval);
    retval
}

#[tokio::main]
async fn main() {
    let saved_state = state::retrieve_saved_state();

    let apns_connection = Arc::new(
        APNSConnection::new(saved_state.as_ref().map(|state| state.push.clone()))
            .await
            .unwrap(),
    );

    let users = if let Some(state) = saved_state.as_ref() {
        state.users.clone()
    } else {
        Vec::new()
    };

    let app_state = ApplicationState {
        apns_connection,
        users: Mutex::new(users),
    };
    if let Err(error) = app_state.update_users().await {
        println!("Error updating users: {:?}", error);
    }

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_validation_data,
            authenticate
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
