pub mod command;
pub mod common;
pub mod lolz;
pub mod scheduler;

use crate::command::Command;
use crate::common::send_message;
use crate::lolz::Lolz;
use effectum::{Error, Job, JobRunner, Queue, RunningJob, Worker};
use frankenstein::AsyncTelegramApi;
use frankenstein::client_reqwest::Bot;
use frankenstein::methods::GetUpdatesParams;
use frankenstein::types::{AllowedUpdate, Message};
use frankenstein::updates::UpdateContent;
use serde::{Deserialize, Serialize};
use std::{env, fmt};
use std::path::PathBuf;
use std::sync::Arc;
use time::OffsetDateTime;

pub struct JobContext {
    queue: Arc<Queue>,
    bot: Bot,
    lolz: Lolz,
}

impl fmt::Debug for JobContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MyContext").finish()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct BumpPayload {
    thread_id: String,
    chat_id: String,
}

async fn is_job_exists(context: &JobContext, thread_id: &str) -> bool {
    let name = format!("bump_job_{}", thread_id);

    match context.queue.get_jobs_by_name(name, 1).await {
        Ok(jobs) => {
            if let Some(job) = jobs.first() {
                if format!("{:?}", job.state) == "Cancelled" {
                    return false;
                }
                return true;
            }
            false
        }
        Err(_) => false,
    }
}

async fn schedule_bump_job(
    context: &JobContext,
    thread_id: String,
    chat_id: String,
    run_at_timestamp: i64,
) -> Result<(), Error> {
    if is_job_exists(context, &thread_id).await {
        return Ok(());
    }

    let run_at = OffsetDateTime::from_unix_timestamp(run_at_timestamp)
        .map_err(|_| Error::InvalidJobState("Invalid timestamp".to_string()))?;

    let name = format!("bump_job_{}", thread_id);

    Job::builder("bump_job")
        .run_at(run_at)
        .name(name)
        .json_payload(&BumpPayload { thread_id, chat_id })?
        .add_to(&context.queue)
        .await?;

    Ok(())
}

fn extract_run_at(thread_data: &serde_json::Value) -> Option<i64> {
    thread_data["thread"]["permissions"]["bump"]["next_available_time"].as_i64()
}

async fn bump_job(job: RunningJob, context: Arc<JobContext>) -> Result<(), Error> {
    let payload: BumpPayload = job.json_payload()?;

    if let Err(e) = context
        .lolz
        .bump_thread(payload.thread_id.parse().unwrap())
        .await
    {
        println!("Error bumping thread: {}", e);
        return Ok(());
    }

    if let Ok(thread) = context
        .lolz
        .get_thread(payload.thread_id.parse().unwrap())
        .await
    {
        if let Some(next_bump) = extract_run_at(&thread) {
            schedule_bump_job(
                &context,
                payload.thread_id.clone(),
                payload.chat_id.clone(),
                next_bump,
            )
            .await?;
        }
    }

    send_message(
        payload.chat_id.parse().unwrap(),
        "WORK WORK WORK",
        context.bot.clone(),
    )
    .await;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let bot_token = env::var("BOT_TOKEN")
        .expect("BOT_TOKEN must be set in .env");
    let lolz_token = env::var("LOLZ_TOKEN")
        .expect("LOLZ_TOKEN must be set in .env");
    let admin_id: i64 = env::var("ADMIN_ID")
        .expect("ADMIN_ID must be set")
        .parse()
        .expect("ADMIN_ID must be a valid integer");

    let bot = Bot::new(&*bot_token);

    let lolz_client = Lolz::new(lolz_token);

    println!("Alles Bonita!");

    let queue = Arc::from(Queue::new(&PathBuf::from("lolzup.db")).await.unwrap());
    let a_job = JobRunner::builder("bump_job", bump_job).build();
    let context = Arc::new(JobContext {
        bot: bot.clone(),
        queue: queue.clone(),
        lolz: lolz_client.clone(),
    });

    let _worker = Worker::builder(&*queue, context.clone())
        .max_concurrency(10)
        .jobs([a_job])
        .build();

    let mut update_params = GetUpdatesParams::builder()
        .allowed_updates(vec![AllowedUpdate::Message, AllowedUpdate::CallbackQuery])
        .build();

    loop {
        if let Ok(response) = bot.get_updates(&update_params).await {
            for update in response.result {
                if let UpdateContent::Message(message) = update.content {
                    if message.chat.id != admin_id {
                        update_params.offset = Some(i64::from(update.update_id) + 1);
                        continue;
                    }

                    handle_message(message, &context).await;
                }
                update_params.offset = Some(i64::from(update.update_id) + 1);
            }
        }
    }
}

async fn process_delete(message: Box<Message>, arg: String, context: Arc<JobContext>) {
    let chat_id = message.chat.id;
    let bot = context.bot.clone();

    if arg.parse::<i64>().is_err() {
        send_message(chat_id, "Please provide a valid numeric thread ID", bot).await;
        return;
    }

    match delete_job(&context, &arg).await {
        Ok(true) => {
            send_message(chat_id, format!("Task for thread {} deleted", arg), bot).await;
        }
        Ok(false) => {
            send_message(chat_id, "No active task found for this thread", bot).await;
        }
        Err(_) => {
            send_message(chat_id, "⚠Error while trying to delete the task", bot).await;
        }
    }
}

async fn delete_job(context: &JobContext, thread_id: &str) -> Result<bool, Error> {
    let name = format!("bump_job_{}", thread_id);

    let job = context.queue.get_jobs_by_name(name, 1).await?;

    if job.is_empty() {
        return Ok(false);
    }

    context.queue.cancel_job(job[0].id).await?;

    Ok(true)
}

async fn handle_message(message: Box<Message>, context: &Arc<JobContext>) {
    let text = match &message.text {
        Some(t) if t.starts_with('/') => t,
        _ => return,
    };

    let bot = context.bot.clone();
    match Command::parse(text) {
        Ok(Command::Start) => {
            send_message(message.chat.id, "Welcome to LolzUP Reborn\n\nCommands:\n\\- `/new {thread_id}`\n\\- `/del {thread_id}`", bot).await;
        }
        Ok(Command::New(id_str)) => {
            process_new(message, id_str, context.clone()).await;
        }
        Ok(Command::Delete(id_str)) => {
            process_delete(message, id_str, context.clone()).await;
        }
        Err(_) => {
            process_error(*message, bot).await;
        }
    }
}

async fn process_new(message: Box<Message>, arg: String, context: Arc<JobContext>) {
    let chat_id = message.chat.id;
    let bot = context.bot.clone();

    let Ok(thread_id_int) = arg.parse::<i64>() else {
        send_message(chat_id, "Only numbers", bot).await;
        return;
    };

    if is_job_exists(&context, &arg).await {
        send_message(chat_id, "This thread is already being tracked", bot).await;
        return;
    }

    match context.lolz.get_thread(thread_id_int).await {
        Ok(data) => {
            let next_bump = extract_run_at(&data);

            match next_bump {
                None => {
                    if context.lolz.bump_thread(thread_id_int).await.is_ok() {
                        let updated = context.lolz.get_thread(thread_id_int).await.unwrap();
                        if let Some(new_time) = extract_run_at(&updated) {
                            let _ = schedule_bump_job(&context, arg, chat_id.to_string(), new_time)
                                .await;
                        }
                        send_message(chat_id, "First bump done Scheduled next", bot).await;
                    }
                }
                Some(timestamp) => {
                    let _ = schedule_bump_job(&context, arg, chat_id.to_string(), timestamp).await;
                    send_message(
                        chat_id,
                        format!("Thread queued, Next bump at: {}", timestamp),
                        bot,
                    )
                    .await;
                }
            }
        }
        Err(_) => {
            send_message(chat_id, "Invalid thread id or API error", bot).await;
        }
    }
}

async fn process_error(message: Message, bot: Bot) {
    send_message(message.chat.id, "Command error or invalid format", bot).await;
}
