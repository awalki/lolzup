from aiogram import Router
from aiogram.filters import CommandStart, BaseFilter
from aiogram.types import Message
from aiogram_dialog import DialogManager, StartMode, ShowMode

from dialogs import MainMenuSG
from settings import settings


class IsAdmin(BaseFilter):
    async def __call__(self, message: Message) -> bool:
        return message.from_user.id == settings.admin_id


router = Router()


@router.message(CommandStart(), IsAdmin())
async def start(message: Message, dialog_manager: DialogManager):
    await dialog_manager.start(MainMenuSG.main, mode=StartMode.RESET_STACK, show_mode=ShowMode.DELETE_AND_SEND)
