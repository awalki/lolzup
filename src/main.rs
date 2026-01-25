pub mod bump;
pub mod command;
pub mod common;
pub mod error;
pub mod lolz;

pub mod scheduler;

use crate::bump::BumpService;
use crate::command::Command;
use crate::common::{escape_md, send_message};
use crate::error::LolzUpError;
use crate::lolz::lolz::LolzHttpClient;
use crate::scheduler::{Scheduler, Task};
use chrono::{Timelike, Utc};
use frankenstein::AsyncTelegramApi;
use frankenstein::client_reqwest::Bot;
use frankenstein::methods::GetUpdatesParams;
use frankenstein::types::{AllowedUpdate, Message};
use frankenstein::updates::UpdateContent;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Sqlite, SqlitePool};
use std::env;
use std::sync::Arc;
use tracing::info;

#[derive(Debug)]
pub struct AppContext {
    bot: Arc<Bot>,
    pool: Arc<SqlitePool>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting UP LOLZ UP");

    let bot_token = env::var("BOT_TOKEN")?;
    let lolz_token = env::var("LOLZ_TOKEN")?;
    let admin_id: i64 = env::var("ADMIN_ID")?.parse()?;

    let bot = Arc::from(Bot::new(&*bot_token));
    let mut update_params = GetUpdatesParams::builder()
        .allowed_updates(vec![AllowedUpdate::Message, AllowedUpdate::CallbackQuery])
        .build();

    let options = SqliteConnectOptions::new()
        .filename("lolzup.db")
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
    let pool = Arc::from(sqlx::SqlitePool::connect_with(options).await?);

    sqlx::migrate!().run(&*pool).await?;

    let lolz_client = LolzHttpClient::new(lolz_token)?;
    let bump_service = Arc::from(BumpService::new(lolz_client));
    let scheduler = Scheduler::new(
        bump_service.clone(),
        pool.clone(),
        bot.clone(),
        Arc::from(admin_id.clone()),
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = scheduler.run_scheduler().await {
            eprintln!("Error while initiating scheduler: {:?}", e);
        }
        info!("Scheduler is active")
    });

    let context = Arc::from(AppContext {
        bot: bot.clone(),
        pool: pool.clone(),
    });

    info!("Telegram polling is active");

    loop {
        if let Ok(response) = bot.get_updates(&update_params).await {
            for update in response.result {
                if let UpdateContent::Message(message) = update.content {
                    if message.chat.id != admin_id {
                        update_params.offset = Some(i64::from(update.update_id) + 1);
                        continue;
                    }

                    handle_message(message, context.clone()).await?;
                }
                update_params.offset = Some(i64::from(update.update_id) + 1);
                info!("Got telegram update")
            }
        }
    }
}

async fn handle_message(
    message: Box<Message>,
    context: Arc<AppContext>,
) -> Result<(), LolzUpError> {
    let text = match &message.text {
        Some(t) if t.starts_with('/') => t,
        _ => return Ok(()),
    };

    let bot = context.bot.clone();
    match Command::parse(text) {
        Ok(Command::Start) => {
            send_message(message.chat.id, "Welcome to LolzUP Reborn\n\nCommands:\n\\- `/new {thread_id}`\n\\- `/del {thread_id}`\n\\- `/list`", bot).await?;
        }
        Ok(Command::List) => {
            process_list(message, context).await?;
        }
        Ok(Command::New(id_str)) => {
            process_new(message, id_str, context.clone()).await?;
        }
        Ok(Command::Delete(id_str)) => {
            process_delete(message, id_str, context.clone()).await?;
        }
        _ => {
            process_error(*message, bot).await?;
        }
    }

    Ok(())
}

async fn process_delete(
    message: Box<Message>,
    arg: String,
    context: Arc<AppContext>,
) -> Result<(), LolzUpError> {
    let result = sqlx::query("DELETE FROM tasks WHERE thread_id = (?)")
        .bind(arg)
        .execute(&*context.pool)
        .await?;

    if result.rows_affected() == 0 {
        send_message(
            message.chat.id,
            "Can't delete: Task not found",
            context.bot.clone(),
        )
        .await?;
    } else {
        send_message(message.chat.id, "Delete successfull", context.bot.clone()).await?;
    }

    Ok(())
}

async fn process_list(message: Box<Message>, context: Arc<AppContext>) -> Result<(), LolzUpError> {
    let tasks = sqlx::query_as::<Sqlite, Task>("SELECT id, thread_id, run_at FROM tasks")
        .fetch_all(&*context.pool)
        .await?;

    let response_text = if tasks.is_empty() {
        "No active tasks found".to_string()
    } else {
        let task_list = tasks
            .iter()
            .map(|task| {
                let link = format!("https://lolz.live/threads/{}", task.thread_id);
                let time = task.run_at.naive_local().to_string();

                format!("{} \\- Will bump at {}", escape_md(&link), escape_md(&time))
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "Active tasks:\n\n{}\n\nHint: use /del \\[thread\\_id\\] to delete a task",
            task_list
        )
    };

    send_message(message.chat.id, response_text, context.bot.clone()).await?;

    Ok(())
}

async fn process_new(
    message: Box<Message>,
    arg: String,
    context: Arc<AppContext>,
) -> Result<(), LolzUpError> {
    let chat_id = message.chat.id;
    let bot = context.bot.clone();

    let Ok(thread_id_int) = arg.parse::<i64>() else {
        send_message(chat_id, "Only numbers", bot.clone()).await?;
        return Ok(());
    };

    let run_at = Utc::now();

    let result = sqlx::query("INSERT INTO tasks (thread_id, run_at) VALUES (?, ?)")
        .bind(thread_id_int)
        .bind(run_at)
        .execute(&*context.pool)
        .await?;

    if result.rows_affected() == 0 {
        send_message(chat_id, "This task already exists", bot.clone()).await?;
    } else {
        send_message(chat_id, "Task added successfully", bot.clone()).await?;
    }

    Ok(())
}

async fn process_error(message: Message, bot: Arc<Bot>) -> Result<(), LolzUpError> {
    send_message(message.chat.id, "Command error or invalid format", bot).await?;

    Ok(())
}
