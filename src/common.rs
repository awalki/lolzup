use frankenstein::client_reqwest::Bot;
use frankenstein::methods::SendMessageParams;
use frankenstein::{AsyncTelegramApi, ParseMode};

pub async fn send_message(chat_id: i64, message: impl Into<String>, bot: Bot) {
    let send_message_params = SendMessageParams::builder()
        .chat_id(chat_id)
        .text(message)
        .parse_mode(ParseMode::MarkdownV2)
        .build();

    if let Err(why) = bot.send_message(&send_message_params).await {
        println!("Failed to send a message: {:?}", why);
    }
}
