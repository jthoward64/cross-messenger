use std::sync::Arc;

use rustpush::{register, APNSConnection, APNSState, IDSAppleUser, IDSUser, PushError};
use serde::{Deserialize, Serialize};

use crate::emulated::bindings::{generate_validation_data, ValidationDataError};

#[derive(Serialize, Deserialize, Clone)]
struct SavedState {
    push: APNSState,
    users: Vec<IDSUser>,
}

/**
 * Start logging in a new user given a username and password
 *
 * If the user does not have 2FA enabled, this will return a new user, otherwise it will return a TwoFaError
 */
pub async fn login_1fa(
    connection: Arc<APNSConnection>,
    username: &str,
    password: &str,
) -> Result<IDSUser, PushError> {
    IDSAppleUser::authenticate(
        connection.clone(),
        username.trim(),
        &(password.trim().to_string()),
    )
    .await
}

/**
 * Finish logging in a new user given a username, password, and validation code
 *
 * This will return a new user
 */
pub async fn login_2fa(
    connection: Arc<APNSConnection>,
    username: &str,
    password: &str,
    validation: &str,
) -> Result<IDSUser, PushError> {
    IDSAppleUser::authenticate(
        connection.clone(),
        username.trim(),
        &(password.trim().to_string() + &validation.trim()),
    )
    .await
}

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
pub async fn register_user(
    users: &mut Vec<IDSUser>,
    connection: Arc<APNSConnection>,
) -> Result<(), RegisterError> {
    let validation = generate_validation_data()?;
    match register(validation.as_str(), users, connection).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
