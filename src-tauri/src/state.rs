use std::sync::Arc;

use tokio::sync::Mutex;

use self::rustpushstate::IMClientError;

pub mod rustpushstate;

pub struct ApplicationState {
    pub rust_push: Arc<Mutex<rustpushstate::RustPushState>>,
}

#[derive(Clone)]
pub struct TauriState(pub Arc<Mutex<ApplicationState>>);

impl TauriState {
    pub async fn new() -> Result<Self, IMClientError> {
        let rust_push = Arc::new(Mutex::new(
            rustpushstate::RustPushState::new(rustpushstate::retrieve_saved_state()).await?,
        ));
        let state = ApplicationState { rust_push };
        Ok(Self(Arc::new(Mutex::new(state))))
    }
}
