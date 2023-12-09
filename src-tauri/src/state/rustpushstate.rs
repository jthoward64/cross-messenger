use std::sync::Arc;

use dirs::{data_local_dir, home_dir};
use rustpush::{APNSConnection, APNSState, IDSUser, IMClient};
use serde::{Deserialize, Serialize};
use tauri::InvokeError;

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
    pub client: Arc<IMClient>,
}

#[derive(Debug)]
pub enum IMClientError {
    NoUsers,
    NoClient,
    PushError(rustpush::PushError),
    ValidationDataError(ValidationDataError),
    RegisterError(RegisterError),
    IOError(std::io::Error),
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
            IMClientError::RegisterError(error) => match error {
                RegisterError::PushError(error) => InvokeError::from(error.to_string()),
                RegisterError::ValidationDataError(error) => error.into(),
            },
            IMClientError::IOError(error) => InvokeError::from(error.to_string()),
        }
    }
}

impl RustPushState {
    pub async fn new(saved_state: Option<SavedState>) -> Result<RustPushState, IMClientError> {
        let (apns_connection, mut users) = match saved_state {
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

        let mut needs_reregistration = false;
        for user in users.iter() {
            println!("Checking user {:?}", user.user_id);
            if user.identity.is_none() {
                println!("User {:?} has no identity", user.user_id);
                needs_reregistration = true;
            }
        }
        if needs_reregistration {
            register_users(&mut users, apns_connection.clone())
                .await
                .unwrap();
        }

        let client = IMClient::new(apns_connection.clone(), Arc::new(users)).await;

        let application_state = RustPushState {
            apns_connection,
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
            users: self.client.users.to_vec(),
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
        let users = self.client.users.clone();
        users
            .iter()
            .find(|user| user.handles.contains(&handle.to_owned()))
            .cloned()
    }

    pub async fn get_user_by_id(&self, id: &str) -> Option<IDSUser> {
        let users = self.client.users.clone();
        users.iter().find(|user| user.user_id == id).cloned()
    }

    // pub async fn update_users(&mut self) -> Result<(), IMClientError> {
    //     let mut users = self.users.write().await;
    //     if users.len() != 0 {
    //         println!("Updating users {:?}", users.len());
    //         match register_users(users.as_mut(), self.apns_connection.clone()).await {
    //             Ok(_) => {
    //                 if let Err(e) = self.save_to_file().await {
    //                     println!("Error saving state: {:?}", e);
    //                 }
    //                 Ok(())
    //             }
    //             Err(RegisterError::PushError(error)) => return Err(error.into()),
    //             Err(RegisterError::ValidationDataError(error)) => return Err(error.into()),
    //         }
    //     } else {
    //         Ok(())
    //     }
    // }

    pub async fn add_user(&mut self, user: IDSUser) -> Result<(), IMClientError> {
        let mut users: Vec<IDSUser> = self.client.users.to_vec();
        users.push(user.clone());

        match register_users(&mut users, self.apns_connection.clone()).await {
            Ok(_) => {
                self.client =
                    Arc::new(IMClient::new(self.apns_connection.clone(), Arc::new(users)).await);
                match self.save_to_file().await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(IMClientError::IOError(e)),
                }?;
                if let Err(e) = self.save_to_file().await {
                    println!("Error saving state: {:?}", e);
                }
                Ok(())
            }
            Err(error) => return Err(IMClientError::RegisterError(error)),
        }
    }
}
