import logging

from aiogram import Bot, Dispatcher
from aiogram_dialog import setup_dialogs

from database import init_db
from dialogs import main_dialog
from handlers import router
from settings import settings
from tkq import broker, redis_source, scheduler

logging.basicConfig(level=logging.INFO)

dp = Dispatcher()
bot = Bot(token=settings.bot_token)

dp.include_routers(router, main_dialog)
setup_dialogs(dp)


@dp.startup()
async def setup_taskiq(bot: Bot, *_args, **_kwargs):
    if not broker.is_worker_process:
        logging.info("Setting up taskiq")
        await broker.startup()
        await redis_source.startup()
        await scheduler.startup()


@dp.shutdown()
async def shutdown_taskiq(bot: Bot, *_args, **_kwargs):
    if not broker.is_worker_process:
        logging.info("Shutting down taskiq")
        await broker.shutdown()


async def main():
    await init_db()

    await dp.start_polling(bot)


if __name__ == '__main__':
    import asyncio

    asyncio.run(main())
