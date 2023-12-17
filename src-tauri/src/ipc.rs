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

    async fn get_user(&self) -> Result<Option<User>, GetUserErrorCode> {
        let app_state = self.tauri_state.0.lock().await;
        let mut state = app_state.rust_push.lock().await;
        if state.client.users.len() == 0 {
            return Err(GetUserErrorCode::NotLoggedIn);
        } else {
            match state.get_active_user().await {
                Some((user, handle)) => Ok(Some(User {
                    handles: user.handles.clone(),
                    selected_handle: handle,
                    user_id: user.user_id.clone(),
                })),
                None => Ok(None),
            }
        }
    }

    async fn select_handle(&self, handle: String) -> Option<SelectHandleErrorCode> {
        let app_state = self.tauri_state.0.lock().await;
        let mut state = app_state.rust_push.lock().await;
        match state.get_user_by_handle(&handle).await {
            Some(_) => {
                state.active_handle = Some(handle);
                None
            }
            None => Some(SelectHandleErrorCode::HandleNotFound),
        }
    }
}
