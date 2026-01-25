use chrono::{DateTime, Utc};

use crate::{error::LolzUpError, lolz::lolz::LolzClient};

#[derive(Debug)]
pub struct BumpService<T: LolzClient> {
    client: T,
}

impl<T> BumpService<T>
where
    T: LolzClient,
{
    pub fn new(client: T) -> Self {
        Self { client }
    }

    pub async fn try_bump_thread(
        &self,
        thread_id: i64,
    ) -> Result<(DateTime<Utc>, bool), LolzUpError> {
        let data = self.client.get_thread_by_id(thread_id).await?;

        if let Some(time) = data.thread.permissions.bump.next_available_time {
            return Ok((time, false));
        }

        let status = self.client.bump_thread(thread_id).await?;

        if status != 200 {
            return Err(LolzUpError::Bump(format!(
                "Bump failed with status: {status}"
            )));
        }

        let updated_data = self.client.get_thread_by_id(thread_id).await?;

        let next_time = updated_data
            .thread
            .permissions
            .bump
            .next_available_time
            .ok_or_else(|| LolzUpError::Bump("Timestamp expected but not found.".to_string()))?;

        Ok((next_time, true))
    }
}
