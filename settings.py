from LOLZTEAM import Forum
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file='.env', env_file_encoding='utf-8')

    lolz_token: str
    bot_token: str
    admin_id: int
    redis_url: str = "redis://redis:6379/0"
    database_url: str = "postgresql+asyncpg://postgres:postgres@postgres:5432/lolzup"


settings = Settings()
forum = Forum(token=settings.lolz_token)
