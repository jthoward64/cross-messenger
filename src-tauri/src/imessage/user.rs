use std::sync::Arc;

use rustpush::{register, APNSConnection, IDSAppleUser, IDSUser, PushError};
use tauri::InvokeError;

use crate::emulated::bindings::{generate_validation_data, ValidationDataError};

pub enum LoginError {
    PushError(PushError),
}

impl From<PushError> for LoginError {
    fn from(e: PushError) -> Self {
        LoginError::PushError(e)
    }
}

impl Into<InvokeError> for LoginError {
    fn into(self) -> InvokeError {
        match self {
            LoginError::PushError(e) => InvokeError::from(e.to_string()),
        }
    }
}

/**
 * Login to Apple with a username and password
 *
 * If the user has 2FA enabled, this will return a TwoFaError if no 2FA code is provided
 */
pub async fn login(
    connection: Arc<APNSConnection>,
    username: &str,
    password: &str,
    two_factor_code: Option<&str>,
) -> Result<IDSUser, LoginError> {
    let password_plus_2fa = match two_factor_code {
        Some(code) => password.trim().to_string() + &code.trim(),
        None => password.trim().to_string(),
    };
    match IDSAppleUser::authenticate(connection.clone(), username.trim(), &password_plus_2fa).await
    {
        Ok(user) => Ok(user),
        Err(e) => Err(LoginError::PushError(e)),
    }
}

#[derive(Debug)]
pub enum RegisterError {
    PushError(PushError),
    ValidationDataError(ValidationDataError),
}

impl From<PushError> for RegisterError {
    fn from(e: PushError) -> Self {
        RegisterError::PushError(e)
    }
}

impl From<ValidationDataError> for RegisterError {
    fn from(e: ValidationDataError) -> Self {
        RegisterError::ValidationDataError(e)
    }
}

/**
 * Register a new user given a validation code
 *
 * This will return a new user
 */
pub async fn register_users(
    users: &mut Vec<IDSUser>,
    connection: Arc<APNSConnection>,
) -> Result<(), RegisterError> {
    let validation = generate_validation_data()?;
    match register(validation.as_str(), users, connection).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
