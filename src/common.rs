use std::sync::Arc;

use frankenstein::client_reqwest::Bot;
use frankenstein::methods::SendMessageParams;
use frankenstein::{AsyncTelegramApi, ParseMode};

use crate::error::LolzUpError;

pub async fn send_message(
    chat_id: i64,
    message: impl Into<String>,
    bot: Arc<Bot>,
) -> Result<(), LolzUpError> {
    let send_message_params = SendMessageParams::builder()
        .chat_id(chat_id)
        .text(message)
        .parse_mode(ParseMode::MarkdownV2)
        .build();

    bot.send_message(&send_message_params).await?;

    Ok(())
}