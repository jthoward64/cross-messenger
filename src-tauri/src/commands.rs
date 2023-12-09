use tauri::InvokeError;

use crate::{
    actions::init::do_login,
    actions::send::do_send_message,
    state::{rustpushstate::IMClientError, TauriState},
};

#[tauri::command]
pub async fn authenticate(
    state: tauri::State<'_, TauriState>,
    username: String,
    password: String,
    code: Option<String>,
) -> Result<bool, IMClientError> {
    println!("authenticate: {:?} {:?} {:?}", username, password, code);
    let retval = do_login(
        state.0.lock().await.rust_push.clone(),
        username,
        password,
        code,
    )
    .await;
    println!("authenticate: {:?}", retval);
    retval
}

#[tauri::command]
pub async fn send_message(
    state: tauri::State<'_, TauriState>,
    message: String,
    to: String,
) -> Result<bool, InvokeError> {
    println!("send_message: {:?} {:?}", message, to);
    let retval = do_send_message(state.0.lock().await.rust_push.clone(), message, to).await;
    println!("send_message: {:?}", retval);
    retval
}
