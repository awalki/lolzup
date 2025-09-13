import datetime
import logging

from aiogram import Bot
from taskiq import TaskiqDepends

from settings import forum, settings
from tkq import broker, redis_source


@broker.task(task_name="bump_task")
async def bump_task(thread_id: str, bot: Bot = TaskiqDepends()) -> None:
    try:
        logging.info(f"Поднятие темы {thread_id}")
        await forum.threads.bump(thread_id)
        await rerun_bump(thread_id)

        await bot.send_message(settings.admin_id, f"Тема {thread_id} была успешно поднята, задача перезапущена")
    except Exception as e:
        logging.error(e)
        await rerun_bump(thread_id)


async def rerun_bump(thread_id: str):
    logging.info(f"Задача запущена/перезапущена. Айди: {thread_id}")

    thread = await forum.threads.get(thread_id)

    data = thread.json()
    next_bump_timestamp = data["thread"]["permissions"]["bump"]["next_available_time"]

    if next_bump_timestamp is None:
        logging.info("Тему можно поднять сейчас, поднимаем!")

        await bump_task.kiq(str(thread_id))
    else:
        logging.info("Тема в КД, ждем")
        eta = datetime.datetime.fromtimestamp(next_bump_timestamp) + datetime.timedelta(seconds=15)

        await bump_task.kicker().with_schedule_id(str(thread_id)).schedule_by_time(
            redis_source,
            eta,
        )
