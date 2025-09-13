from sqlalchemy import BigInteger
from sqlalchemy.ext.asyncio import AsyncAttrs, async_sessionmaker, create_async_engine
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column

engine = create_async_engine(
    url="sqlite+aiosqlite:///data/lolzup.db", echo=False
)

async_session = async_sessionmaker(bind=engine, expire_on_commit=True)


class Base(AsyncAttrs, DeclarativeBase):
    pass


class Thread(Base):
    __tablename__ = "thread"

    id: Mapped[int] = mapped_column(primary_key=True)

    enabled: Mapped[bool] = mapped_column(default=True)
    thread_id: Mapped[int] = mapped_column(BigInteger, unique=True)
    name: Mapped[str] = mapped_column()


async def init_db():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
