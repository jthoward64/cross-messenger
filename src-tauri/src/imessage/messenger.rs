use std::{io::Seek, sync::Arc};

use rustpush::{
    init_logger, register, APNSConnection, APNSState, ConversationData, IDSAppleUser, IDSUser,
    IMClient, IconChangeMessage, IndexedMessagePart, MMCSFile, Message, MessagePart, MessageParts,
    NormalMessage, PushError, RecievedMessage,
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};
use tokio::{
    fs,
    io::{self, AsyncBufReadExt, BufReader},
};
use uuid::Uuid;

/**
 * Send a plain text message to a user
 *
 * This will return the message ID
 */
pub async fn send_text_message(
    client: &mut IMClient,
    conversation: ConversationData,
    handle: &str,
    message: Message,
) -> Result<String, PushError> {
    let mut msg = client.new_msg(conversation, handle, message).await;
    client.send(&mut msg).await?;
    Ok(msg.id)
}
