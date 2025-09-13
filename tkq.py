from taskiq import TaskiqScheduler
from taskiq_aiogram import init
from taskiq_redis import RedisScheduleSource, ListQueueBroker

from settings import settings

broker = ListQueueBroker(settings.redis_url)

redis_source = RedisScheduleSource(settings.redis_url)

scheduler = TaskiqScheduler(broker, sources=[redis_source])

init(
    broker,
    "main:dp",
    "main:bot",
)
