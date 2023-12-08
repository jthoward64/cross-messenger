use std::sync::Arc;

use dirs::config_dir;
use rustpush::{APNSConnection, APNSState, IDSUser};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::imessage::user::{register_users, RegisterError};

// Saved to $XDG_CONFIG_HOME/rustpush/state.json

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
    pub users: Mutex<Vec<IDSUser>>,
}

impl ApplicationState {
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

    pub async fn update_users(&self) -> Result<(), RegisterError> {
        let mut users = self.users.lock().await;
        register_users(users.as_mut(), self.apns_connection.clone()).await
    }
}
