use std::sync::Arc;

use dirs::{data_local_dir, home_dir};
use rustpush::{init_logger, APNSConnection, APNSState, IDSUser, IMClient};
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
    let state_path = data_local_dir()
        .unwrap_or(home_dir().unwrap().join(".crossmessenger"))
        .join("crossmessenger")
        .join("state.json");
    if !state_path.exists() {
        return None;
    }
    let state_file = std::fs::File::open(state_path).ok()?;
    let state: SavedState = serde_json::from_reader(state_file).ok()?;
    Some(state)
}

pub struct RustPushState {
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

impl RustPushState {
    pub async fn new(saved_state: Option<SavedState>) -> Result<RustPushState, IMClientError> {
        init_logger();

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
        let application_state = RustPushState {
            apns_connection,
            users: Arc::new(Mutex::new(users)),
            active_handle: Arc::new(active_handle),
            client: Arc::new(client),
        };
        if let Err(e) = application_state.save_to_file().await {
            println!("Error saving state: {:?}", e);
        }
        Ok(application_state)
    }

    pub async fn to_saved_state(&self) -> SavedState {
        SavedState {
            push: self.apns_connection.state.clone(),
            users: self.users.lock().await.to_owned(),
        }
    }

    pub async fn save_to_file(&self) -> Result<(), std::io::Error> {
        let state_path = data_local_dir()
            .unwrap_or(home_dir().unwrap().join(".crossmessenger"))
            .join("crossmessenger");
        std::fs::create_dir_all(&state_path)?;
        let state_path = state_path.join("state.json");
        let state_file = std::fs::File::create(state_path)?;
        let state = self.to_saved_state().await;
        serde_json::to_writer(state_file, &state)?;
        Ok(())
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
        if users.len() != 0 {
            println!("Updating users {:?}", users.len());
            match register_users(users.as_mut(), self.apns_connection.clone()).await {
                Ok(_) => {
                    self.active_handle =
                        Arc::new(users.first().map(|user| user.handles[0].clone()));
                    if let Err(e) = self.save_to_file().await {
                        println!("Error saving state: {:?}", e);
                    }
                    Ok(())
                }
                Err(RegisterError::PushError(error)) => return Err(error.into()),
                Err(RegisterError::ValidationDataError(error)) => return Err(error.into()),
            }
        } else {
            Ok(())
        }
    }

    async fn ensure_client(&mut self) -> Result<(), IMClientError> {
        if self.client.is_none() {
            let users = self.users.lock().await;
            let client =
                IMClient::new(self.apns_connection.clone(), Arc::new(users.to_owned())).await;
            self.client = Arc::new(Some(client));
            if let Err(e) = self.save_to_file().await {
                println!("Error saving state: {:?}", e);
            }
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
