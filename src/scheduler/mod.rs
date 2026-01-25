use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use frankenstein::client_reqwest::Bot;
use sqlx::{Sqlite, SqlitePool};
use tracing::{error, info};

use crate::{bump::BumpService, common::send_message, error::LolzUpError, lolz::lolz::LolzClient};

#[derive(sqlx::FromRow)]
pub struct Task {
    pub id: u32,
    pub thread_id: i64,
    pub run_at: DateTime<Utc>,
}

pub struct Scheduler<T>
where
    T: LolzClient,
{
    bot: Arc<Bot>,
    client: Arc<BumpService<T>>,
    pool: Arc<SqlitePool>,

    admin_id: Arc<i64>,
}

impl<T> Scheduler<T>
where
    T: LolzClient + Send + Sync + 'static,
{
    pub async fn new(
        client: Arc<BumpService<T>>,
        pool: Arc<SqlitePool>,
        bot: Arc<Bot>,
        admin_id: Arc<i64>,
    ) -> Result<Self, LolzUpError> {
        Ok(Self {
            client: client,
            pool: pool,
            bot,
            admin_id,
        })
    }

    pub async fn run_scheduler(&self) -> Result<(), LolzUpError> {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            let now = Utc::now();

            info!("Looking for tasks that we can bump");

            let tasks = sqlx::query_as::<Sqlite, Task>(
                "SELECT id, thread_id, run_at FROM tasks WHERE run_at <= ?",
            )
            .bind(now)
            .fetch_all(&*self.pool)
            .await?;

            for task in tasks {
                let client_clone = self.client.clone();
                let pool_clone = self.pool.clone();
                let bot_clone = self.bot.clone();
                let admin_id_clone = self.admin_id.clone();

                info!(
                    "Got a task that we are going to up, thread id: {}",
                    task.thread_id
                );

                tokio::spawn(async move {
                    match client_clone.try_bump_thread(task.thread_id).await {
                        Ok(data) => {
                            info!("Next task run: {}", data.0);

                            let update_result =
                                sqlx::query("UPDATE tasks SET run_at = ? WHERE id = ?")
                                    .bind(data.0)
                                    .bind(task.id)
                                    .execute(&*pool_clone)
                                    .await;

                            if let Err(e) = update_result {
                                error!("Failed to update database for task {}: {:?}", task.id, e);
                                let _ = send_message(
                                    *admin_id_clone,
                                    format!(
                                        "Failed to update database for task {}: {:?}",
                                        task.id, e
                                    ),
                                    bot_clone.clone(),
                                )
                                .await;
                            }
                            if data.1 {
                                let _ = send_message(
                                    *admin_id_clone,
                                    format!("Sucessfully bumped the thread: {}", task.thread_id),
                                    bot_clone,
                                )
                                .await;
                                info!("Successfully bumped the thread: {}", task.thread_id)
                            }
                        }
                        Err(e) => {
                            error!("Failed to bump thread {}: {:?}", task.thread_id, e);
                            let _ = send_message(
                                *admin_id_clone,
                                format!("Failed to bump thread {}: {:?}", task.thread_id, e),
                                bot_clone,
                            )
                            .await;
                        }
                    }
                });
            }
        }
    }
}
