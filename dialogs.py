from aiogram import F
from aiogram.fsm.state import StatesGroup, State
from aiogram.types import Message, CallbackQuery
from aiogram_dialog import Window, Dialog, DialogManager
from aiogram_dialog.widgets.input import TextInput, ManagedTextInput
from aiogram_dialog.widgets.kbd import Start, Select, Button, Back, Column
from aiogram_dialog.widgets.text import Const, Multi, List, Format, Case

from repo import Repo
from settings import forum
from tasks import rerun_bump


class MainMenuSG(StatesGroup):
    main = State()

    thread_id = State()
    my_threads = State()
    current_thread = State()


async def success_handler(message: Message, widget: ManagedTextInput, dialog_manager: DialogManager, text: str) -> None:
    thread_id = dialog_manager.find("thread_id").get_value()
    thread = await forum.threads.get(thread_id)

    data = thread.json()
    thread_name = data["thread"]["thread_title"]

    await Repo.create_thread(int(thread_id), thread_name)
    await rerun_bump(str(thread_id))

    await dialog_manager.done()


async def getter(**kwargs):
    return {"has_threads": await Repo.has_threads(), "threads": await Repo.get_threads()}


async def on_select(callback, widget, manager, item_id: str):
    manager.dialog_data["selected_thread_id"] = int(item_id)

    await manager.switch_to(MainMenuSG.current_thread)


async def on_delete(callback: CallbackQuery, button: Button,
                    manager: DialogManager):
    thread_id = manager.dialog_data.get("selected_thread_id")

    await Repo.delete_thread(thread_id)

    await manager.done()


async def on_toggle_bump(callback: CallbackQuery, button: Button, manager: DialogManager):
    thread_id = manager.dialog_data.get("selected_thread_id")

    await Repo.toggle_thread(thread_id)

    await manager.done()


async def current_thread_getter(dialog_manager: DialogManager, **kwargs):
    thread_id = dialog_manager.dialog_data.get("selected_thread_id")
    thread = await Repo.get_thread_by_thread_id(thread_id)

    return {"thread": thread}


thread_list = List(
    Case(
        {
            True: Format("[🟢] {item.thread_id}. {item.name}"),
            False: Format("[🔴] {item.thread_id}. {item.name}"),
        },
        selector=F["item"].enabled,
    ),
    id="threads",
    items="threads",
    when="has_threads",
)

current_thread_window = Window(
    Multi(
        Format("{thread.name} [{thread.thread_id}]\n"),
        Case(
            texts={
                True: Const("Автоподнятие: 🟢"),
                False: Const("Автоподнятие: 🔴")
            },
            selector=F["thread"].enabled,
        ),
    ),
    Button(
        Case(
            texts={
                True: Const("Отключить автоподнятие"),
                False: Const("Включить автоподнятие")
            },
            selector=F["thread"].enabled,
        ),
        id="toggle_bump",
        on_click=on_toggle_bump,
    ),
    Button(
        Const("Удалить тему"),
        id="delete_thread",
        on_click=on_delete,
    ),
    Back(Const("Назад"), id="back"),
    state=MainMenuSG.current_thread,
    getter=current_thread_getter,
)

my_threads_window = Window(
    Multi(
        Const("Управление темами"),
        thread_list
    ),
    Column(
        Select(
            Format("{item.thread_id}"),
            id="thread_id",
            items="threads",
            item_id_getter=lambda item: item.thread_id,
            on_click=on_select
        )
    ),
    Start(Const("Назад"), id="main", state=MainMenuSG.main),
    state=MainMenuSG.my_threads,
    getter=getter,
)

main_window = Window(
    Multi(
        Const("LOLZ UP, автоподнятие здорового человека"),
        Const("➖➖➖➖➖➖➖➖➖\nМои темы", when="has_threads"),
        thread_list
    ),
    Start(Const("Добавить тему"), id="add_theme", state=MainMenuSG.thread_id),
    Start(Const("Мои темы"), id="my_threads", state=MainMenuSG.my_threads, when="has_threads"),
    state=MainMenuSG.main,
    getter=getter,
)

thread_id_window = Window(
    Const("Введите айди темы, которую хотите добавить в авто-поднятие"),
    TextInput(id="thread_id", on_success=success_handler, type_factory=int),
    state=MainMenuSG.thread_id,
)

main_dialog = Dialog(main_window, thread_id_window, my_threads_window, current_thread_window)
