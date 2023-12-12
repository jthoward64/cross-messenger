use std::sync::Arc;

use rustpush::{ConversationData, Message, NormalMessage};
use tauri::ipc::InvokeError;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{imessage::messenger::send_text_message, state::rustpushstate::RustPushState};

pub async fn do_send_message(
    state: Arc<Mutex<RustPushState>>,
    message: String,
    // conversation: ConversationData,
    to: String,
) -> Result<bool, InvokeError> {
    let client = state.lock().await.client.clone();
    match send_text_message(
        client,
        ConversationData {
            participants: vec![to],
            cv_name: None,
            sender_guid: Some(Uuid::new_v4().to_string()),
        },
        Message::Message(NormalMessage::new(message)),
    )
    .await
    {
        Ok(_) => Ok(true),
        Err(e) => Err(InvokeError::from(e.to_string())),
    }
}
