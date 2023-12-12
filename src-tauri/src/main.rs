// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_bindgen_host::ipc_router_wip::{BuilderExt, Router};

use state::TauriState;

pub mod actions;
pub mod commands;
pub mod dataplist;
pub mod emulated;
pub mod imessage;
pub mod ipc;
pub mod state;

#[tokio::main]
async fn main() {
    let tauri_state = TauriState::new().await.unwrap();

    let mut router: Router<ipc::IpcCtx> = Router::new(ipc::IpcCtx {
        tauri_state: tauri_state.clone(),
    });
    ipc::ipc::add_to_router(&mut router, |ctx| ctx).unwrap();

    tauri::Builder::default()
        .manage(tauri_state)
        .ipc_router(router)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
