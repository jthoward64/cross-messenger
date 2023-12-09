use rustpush::{ConversationData, Message, NormalMessage};
use tauri::InvokeError;
use tokio::sync::Mutex;

use crate::{imessage::messenger::send_text_message, state::ApplicationState};

#[tauri::command]
pub async fn send_message(
    state: tauri::State<'_, Mutex<ApplicationState>>,
    message: String,
    // conversation: ConversationData,
    to: String,
) -> Result<bool, InvokeError> {
    if let Some(active_handle) = state.lock().await.active_handle.as_ref() {
        match state.lock().await.get_client().await {
            Ok(client) => {
                match send_text_message(
                    client,
                    ConversationData {
                        participants: vec![to],
                        cv_name: None,
                        sender_guid: None,
                    },
                    &active_handle,
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
                return Ok(false);
            }
        }
    } else {
        Ok(false)
    }
}
