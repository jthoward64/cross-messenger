use std::sync::Arc;

use dirs::config_dir;
use rustpush::{APNSConnection, APNSState, IDSUser, IMClient};
use serde::{Deserialize, Serialize};
use tauri::InvokeError;
use tokio::sync::Mutex;

use crate::{
    emulated::bindings::ValidationDataError,
    imessage::user::{register_users, RegisterError},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedState {
    pub push: APNSState,
    pub users: Vec<IDSUser>,
}

pub fn retrieve_saved_state() -> Option<SavedState> {
    let state_path = config_dir()?.join("rustpush").join("state.json");
    if !state_path.exists() {
        return None;
    }
    let state_file = std::fs::File::open(state_path).ok()?;
    let state: SavedState = serde_json::from_reader(state_file).ok()?;
    Some(state)
}

pub struct ApplicationState {
    pub apns_connection: Arc<APNSConnection>,
    pub users: Arc<Mutex<Vec<IDSUser>>>,
    pub active_handle: Arc<Option<String>>,
    client: Arc<Option<IMClient>>,
}

#[derive(Debug)]
pub enum IMClientError {
    NoUsers,
    NoClient,
    PushError(rustpush::PushError),
    ValidationDataError(ValidationDataError),
}

impl From<rustpush::PushError> for IMClientError {
    fn from(error: rustpush::PushError) -> Self {
        IMClientError::PushError(error)
    }
}
impl From<ValidationDataError> for IMClientError {
    fn from(error: ValidationDataError) -> Self {
        IMClientError::ValidationDataError(error)
    }
}
impl Into<InvokeError> for IMClientError {
    fn into(self) -> InvokeError {
        match self {
            IMClientError::NoUsers => InvokeError::from("No logged in users"),
            IMClientError::NoClient => InvokeError::from("No client configured"),
            IMClientError::PushError(error) => InvokeError::from(error.to_string()),
            IMClientError::ValidationDataError(error) => error.into(),
        }
    }
}

impl ApplicationState {
    pub async fn new(
        saved_state: Option<SavedState>,
    ) -> Result<Mutex<ApplicationState>, IMClientError> {
        let (apns_connection, users) = match saved_state {
            Some(saved_state) => {
                let apns_connection = Arc::new(APNSConnection::new(Some(saved_state.push)).await?);
                let users = saved_state.users;
                (apns_connection, users)
            }
            None => {
                let apns_connection = Arc::new(APNSConnection::new(None).await?);
                let users: Vec<IDSUser> = Vec::new();
                (apns_connection, users)
            }
        };

        let active_handle = None;
        let client = None;
        Ok(Mutex::new(ApplicationState {
            apns_connection,
            users: Arc::new(Mutex::new(users)),
            active_handle: Arc::new(active_handle),
            client: Arc::new(client),
        }))
    }
    pub async fn get_user_by_handle(&self, handle: &str) -> Option<IDSUser> {
        let users = self.users.lock().await;
        users
            .iter()
            .find(|user| user.handles.contains(&handle.to_owned()))
            .cloned()
    }

    pub async fn get_user_by_id(&self, id: &str) -> Option<IDSUser> {
        let users = self.users.lock().await;
        users.iter().find(|user| user.user_id == id).cloned()
    }

    pub async fn update_users(&mut self) -> Result<(), IMClientError> {
        self.client = Arc::new(None);
        let mut users = self.users.lock().await;
        match register_users(users.as_mut(), self.apns_connection.clone()).await {
            Ok(_) => Ok(()),
            Err(RegisterError::PushError(error)) => return Err(error.into()),
            Err(RegisterError::ValidationDataError(error)) => return Err(error.into()),
        }
    }

    async fn ensure_client(&mut self) -> Result<(), IMClientError> {
        if self.client.is_none() {
            let users = self.users.lock().await;
            let client =
                IMClient::new(self.apns_connection.clone(), Arc::new(users.to_owned())).await;
            self.client = Arc::new(Some(client));
        }
        Ok(())
    }

    pub async fn get_client(&mut self) -> Result<&IMClient, IMClientError> {
        self.ensure_client().await?;
        match self.client.as_ref() {
            Some(client) => Ok(client),
            None => Err(IMClientError::NoClient),
        }
    }
}
