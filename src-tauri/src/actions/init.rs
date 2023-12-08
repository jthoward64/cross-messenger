use rustpush::PushError;
use tauri::InvokeError;

use crate::{
    imessage::user::{login, LoginError},
    state::ApplicationState,
};

#[tauri::command]
pub async fn authenticate(
    state: tauri::State<'_, ApplicationState>,
    username: String,
    password: String,
    code: Option<String>,
) -> Result<bool, InvokeError> {
    match login(
        state.apns_connection.to_owned(),
        &username,
        &password,
        code.as_deref(),
    )
    .await
    {
        Ok(user) => {
            state.users.lock().await.push(user);
            Ok(true)
        }
        Err(LoginError::PushError(PushError::TwoFaError)) => Ok(false),
        Err(e) => Err(e.into()),
    }
}
