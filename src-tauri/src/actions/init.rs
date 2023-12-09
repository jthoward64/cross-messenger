use std::sync::Arc;

use rustpush::PushError;
use tokio::sync::Mutex;

use crate::{
    imessage::user::{login, LoginError},
    state::rustpushstate::{IMClientError, RustPushState},
};

pub async fn do_login(
    state: Arc<Mutex<RustPushState>>,
    username: String,
    password: String,
    code: Option<String>,
) -> Result<bool, IMClientError> {
    match login(
        state.lock().await.apns_connection.to_owned(),
        &username,
        &password,
        code.as_deref(),
    )
    .await
    {
        Ok(user) => {
            println!("Logged in as {:?}", user.user_id);
            state.lock().await.users.lock().await.push(user);
            println!("Updating users");
            state.lock().await.update_users().await?;
            println!("Updated users");
            Ok(true)
        }
        Err(LoginError::PushError(PushError::TwoFaError)) => {
            println!("2FA required");
            Ok(false)
        }
        Err(e) => match e {
            LoginError::PushError(error) => {
                println!("Error logging in: {:?}", error);
                Err(IMClientError::PushError(error))
            }
        },
    }
}
