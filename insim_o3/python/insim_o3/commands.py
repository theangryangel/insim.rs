"""
Commands - chat-command middleware and ``@command`` decorator.

Attach :class:`CommandsMiddleware` to a client so ``Mso`` packets with
``usertype == Prefix`` are parsed into :class:`Command` events, then route
individual commands with ``@command("name")``::

    from insim_o3 import App, Connection
    from insim_o3.handler import Handler
    from insim_o3.commands import Command, CommandsMiddleware, command

    class Bot(Handler):
        @command("echo")
        async def echo(self, cmd: Command, conn: Connection) -> None:
            await conn.send_message(f"echo: {cmd.raw_args}", ucid=cmd.ucid)

        @command("kick")
        async def kick(self, cmd: Command, conn: Connection) -> None:
            if not cmd.args:
                return
            target = int(cmd.args[0])
            ...

    app = App(prefix="!", middleware=[CommandsMiddleware()], handlers=[Bot()])
"""

from __future__ import annotations

import inspect
import shlex
from collections.abc import Awaitable, Callable
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

from insim_o3.handler import _ON_ATTR
from insim_o3.packets import Iii, Mso, MsoUserType

if TYPE_CHECKING:
    from insim_o3.connection import Connection


@dataclass
class Command:
    """A parsed chat-command invocation."""

    name: str
    args: list[str]
    raw_args: str
    ucid: int
    plid: int
    source: Mso | Iii


def command(
    name: str,
) -> Callable[[Callable[..., Awaitable[None] | None]], Callable[..., Awaitable[None]]]:
    """Register a :class:`Handler` method as the callback for chat command *name*.

    The decorated method receives ``(self, cmd: Command, conn: Connection)``
    and is only invoked when ``cmd.name`` matches.
    """
    name = name.lower()

    def decorator(
        fn: Callable[..., Awaitable[None] | None],
    ) -> Callable[..., Awaitable[None]]:
        async def wrapper(self: Any, event: Command, conn: Connection) -> None:
            if event.name != name:
                return
            result = fn(self, event, conn)
            if inspect.isawaitable(result):
                await result

        wrapper.__name__ = fn.__name__
        wrapper.__qualname__ = fn.__qualname__
        wrapper.__doc__ = fn.__doc__
        setattr(wrapper, _ON_ATTR, (Command,))
        return wrapper

    return decorator


class CommandsMiddleware:
    """Parse chat commands into :class:`Command` events.

    Accepts three sources:

    * ``Mso`` packets with ``usertype == Prefix`` (LFS pre-filtered).
    * ``Mso`` packets with ``usertype == User`` whose text starts with
      *prefix* (if *prefix* is set).
    * ``Iii`` packets (``/i`` messages), always treated as commands.
    """

    def __init__(self, prefix: str | None = None) -> None:
        if prefix is not None and len(prefix) != 1:
            raise ValueError("prefix must be a single character or None")
        self._prefix = prefix

    async def on_connect(self, conn: Connection) -> None:
        pass

    async def on_shutdown(self) -> None:
        pass

    async def on_packet(self, packet: object) -> list[Any]:
        match packet:
            case Mso():
                body = self._extract_mso(packet)
            case Iii():
                body = packet.msg
            case _:
                return []
        if not body:
            return []
        cmd = _parse(body, packet)
        return [cmd] if cmd is not None else []

    def _extract_mso(self, packet: Mso) -> str | None:
        body = packet.msg[packet.textstart :]
        if not body:
            return None
        if packet.usertype is MsoUserType.Prefix:
            return body[1:] if body and not body[0].isalnum() else body
        if packet.usertype is MsoUserType.User and self._prefix is not None:
            if body.startswith(self._prefix):
                return body[len(self._prefix) :]
        return None


def _parse(body: str, source: Mso | Iii) -> Command | None:
    try:
        tokens = shlex.split(body)
    except ValueError:
        tokens = body.split()
    if not tokens:
        return None
    name, *args = tokens
    _, _, raw_args = body.partition(" ")
    return Command(
        name=name.lower(),
        args=args,
        raw_args=raw_args.strip(),
        ucid=source.ucid,
        plid=source.plid,
        source=source,
    )
