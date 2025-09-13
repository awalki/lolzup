from sqlalchemy import select

from database import async_session, Thread
from tkq import redis_source


class Repo:
    @staticmethod
    async def create_thread(thread_id: int, name: str):
        async with async_session() as sess:
            existing = await sess.scalar(select(Thread).where(Thread.thread_id == thread_id))
            if existing:
                return existing

            new_thread = Thread(thread_id=thread_id, name=name)
            sess.add(new_thread)
            await sess.commit()
            await sess.refresh(new_thread)
            return new_thread

    @staticmethod
    async def get_threads():
        async with async_session() as sess:
            threads = await sess.scalars(select(Thread))
            return threads.all()

    @staticmethod
    async def delete_thread(thread_id: int):
        async with async_session() as sess:
            await sess.delete(await Repo.get_thread_by_thread_id(thread_id))
            await sess.commit()

    @staticmethod
    async def get_thread_by_thread_id(thread_id: int):
        async with async_session() as sess:
            thread = await sess.scalar(select(Thread).where(Thread.thread_id == thread_id))
            return thread

    @staticmethod
    async def toggle_thread(thread_id: int):
        async with async_session() as sess:
            thread = await sess.scalar(select(Thread).where(Thread.thread_id == thread_id))
            if thread.enabled:
                thread.enabled = False
                await redis_source.delete_schedule(thread.thread_id)
            else:
                thread.enabled = True
                from tasks import rerun_bump
                await rerun_bump(str(thread_id))

            await sess.commit()

    @staticmethod
    async def has_threads():
        async with async_session() as sess:
            stmt = select(select(1).select_from(Thread).exists())
            return bool(await sess.scalar(stmt))
