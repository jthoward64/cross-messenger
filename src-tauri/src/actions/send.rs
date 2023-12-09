use std::sync::Arc;

use rustpush::{ConversationData, Message, NormalMessage};
use tauri::InvokeError;
use tokio::sync::Mutex;

use crate::{imessage::messenger::send_text_message, state::rustpushstate::RustPushState};

pub async fn do_send_message(
    state: Arc<Mutex<RustPushState>>,
    message: String,
    // conversation: ConversationData,
    to: String,
) -> Result<bool, InvokeError> {
    match state.lock().await.get_client().await {
        Ok(client) => {
            match send_text_message(
                client,
                ConversationData {
                    participants: vec![to],
                    cv_name: None,
                    sender_guid: None,
                },
                Message::Message(NormalMessage::new(message)),
            )
            .await
            {
                Ok(_) => Ok(true),
                Err(e) => Err(InvokeError::from(e.to_string())),
            }
        }
        Err(e) => {
            println!("Error getting client: {:?}", e);
            Ok(false)
        }
    }
}
