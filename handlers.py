from aiogram import Router
from aiogram.filters import CommandStart
from aiogram.types import Message
from aiogram_dialog import DialogManager, StartMode, ShowMode

from dialogs import MainMenuSG

router = Router()


@router.message(CommandStart())
async def start(message: Message, dialog_manager: DialogManager):
    await dialog_manager.start(MainMenuSG.main, mode=StartMode.RESET_STACK, show_mode=ShowMode.DELETE_AND_SEND)
