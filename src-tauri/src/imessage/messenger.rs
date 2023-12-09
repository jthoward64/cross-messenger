use rustpush::{ConversationData, IMClient, Message, PushError};

/**
 * Send a plain text message to a user
 *
 * This will return the message ID
 */
pub async fn send_text_message(
    client: &IMClient,
    conversation: ConversationData,
    handle: &str,
    message: Message,
) -> Result<String, PushError> {
    let mut msg = client.new_msg(conversation, handle, message).await;
    client.send(&mut msg).await?;
    Ok(msg.id)
}
