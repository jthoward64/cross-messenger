use async_trait::async_trait;
use rustpush::PushError;

use crate::{
    imessage::user::{login, LoginError},
    state::TauriState,
};

use self::ipc::{GetUserErrorCode, LoginErrorCode, LogoutErrorCode, SelectHandleErrorCode, User};

tauri_bindgen_host::generate!({
    path: "ipc.wit",
    async: true,
    tracing: true,
});

#[derive(Clone)]
pub struct IpcCtx {
    pub tauri_state: TauriState,
}

/*
 enum loginErrorCode {
   twoFactorRequired,
   loginFailed,
   unknown,
 }
 enum logoutErrorCode {
   notLoggedIn,
   unknown,
 }
 enum getUserErrorCode {
   notLoggedIn,
   unknown,
 }
 enum selectHandleErrorCode {
   notLoggedIn,
   handleNotFound,
   unknown,
 }
*/

#[async_trait]
impl ipc::Ipc for IpcCtx {
    async fn login(
        &self,
        username: String,
        password: String,
        code: Option<String>,
    ) -> Option<LoginErrorCode> {
        let app_state = self.tauri_state.0.lock().await;
        let mut state = app_state.rust_push.lock().await;
        match login(
            state.apns_connection.to_owned(),
            &username,
            &password,
            code.as_deref(),
        )
        .await
        {
            Ok(user) => {
                println!("Logged in as {:?}", user.user_id);
                match state.add_user(user).await {
                    Ok(_) => None,
                    Err(_) => Some(LoginErrorCode::Unknown),
                }
            }
            Err(LoginError::PushError(PushError::TwoFaError)) => {
                Some(LoginErrorCode::TwoFactorRequired)
            }
            Err(LoginError::PushError(error)) => {
                println!("Error logging in: {:?}", error);
                Some(LoginErrorCode::LoginFailed)
            }
        }
    }

    async fn logout(&self) -> Option<LogoutErrorCode> {
        Some(LogoutErrorCode::Unknown)
    }
}
