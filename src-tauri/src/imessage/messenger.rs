use rustpush::{ConversationData, IMClient, Message, PushError};

/**
 * Send a plain text message to a user
 *
 * This will return the message ID
 */
pub async fn send_text_message(
    client: &IMClient,
    conversation: ConversationData,
    message: Message,
) -> Result<String, PushError> {
    let handles = client.get_handles();
    if let Some(handle) = handles.first() {
        let mut msg = client.new_msg(conversation, &handle, message).await;
        println!("Sending message: {:?}", msg.to_string());
        client.send(&mut msg).await?;
        println!("Sent message: {:?}", msg.to_string());
        Ok(msg.id)
    } else {
        // TODO: Return a proper error
        Err(PushError::TwoFaError)
    }
}
