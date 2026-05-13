"""
App - composition root for an InSim bot.

All configuration is supplied at construction time; the server address is
passed when you actually connect::

    from insim_o3 import App
    from insim_o3.handler import Handler, on
    from insim_o3.packets import IsiFlag, Ncn

    class Bot(Handler):
        @on
        async def join(self, packet: Ncn, conn: Connection) -> None:
            print(f"[join] {packet.pname}")

    app = App(flags=[IsiFlag.MSO_COLS], prefix="!", handlers=[Bot()])
    app.run("127.0.0.1:29999")

Use ``async with app.connect(addr)`` to embed inside an existing asyncio
program::

    async def main() -> None:
        async with app.connect("127.0.0.1:29999") as conn:
            await app.serve()
"""

from __future__ import annotations

import asyncio
import contextlib
import logging
from collections.abc import AsyncIterator
from typing import Any

from insim_o3._insim import _Insim
from insim_o3.connection import Connection
from insim_o3.dispatcher import dispatch
from insim_o3.handler import ErrorFn, Handler, default_on_error
from insim_o3.middleware import Middleware
from insim_o3.packets import IsiFlag

_log = logging.getLogger(__name__)


class App:
    """High-level async InSim application.

    All handlers and middleware are passed at construction time.  No network
    connection is opened until :meth:`connect` or :meth:`run` is called.
    """

    def __init__(
        self,
        *,
        flags: list[IsiFlag] | None = None,
        iname: str | None = None,
        admin_password: str | None = None,
        interval_ms: int | None = None,
        prefix: str | None = None,
        capacity: int = 512,
        on_error: ErrorFn | None = None,
        handlers: list[Handler] | None = None,
        middleware: list[Middleware] | None = None,
    ) -> None:
        self._flags = flags
        self._iname = iname
        self._admin_password = admin_password
        self._interval_ms = interval_ms
        self._prefix = prefix
        self._capacity = capacity
        self._on_error: ErrorFn = on_error or default_on_error
        self._inner: _Insim | None = None
        self._conn: Connection | None = None
        self._handlers: tuple[Handler, ...] = tuple(handlers or ())
        self._middleware: tuple[Middleware, ...] = tuple(middleware or ())

    @contextlib.asynccontextmanager
    async def connect(self, addr: str) -> AsyncIterator[Connection]:
        """Async context manager: open the TCP connection and run middleware lifecycle.

        Yields the live :class:`~insim_o3.connection.Connection`::

            async with app.connect("127.0.0.1:29999") as conn:
                await app.serve()
        """
        self._inner = await _Insim.connect(
            addr,
            flags=self._flags,
            iname=self._iname,
            admin_password=self._admin_password,
            interval_ms=self._interval_ms,
            prefix=self._prefix,
            capacity=self._capacity,
        )
        self._conn = Connection(self._inner)
        for mw in self._middleware:
            await mw.on_connect(self._conn)
        try:
            yield self._conn
        finally:
            for mw in self._middleware:
                await mw.on_shutdown()
            await self._inner.shutdown()
            self._conn = None
            self._inner = None

    async def serve(self) -> None:
        """
        Receive and dispatch packets until the connection closes or the task is
        cancelled.
        """
        if self._inner is None:
            raise RuntimeError(
                "App is not connected; use `async with app.connect(addr): await"
                " app.serve()`"
            )
        inner = self._inner
        while True:
            raw = await inner.recv()
            await self._dispatch(raw)

    def run(self, addr: str) -> None:
        """Synchronous entry point: connect, dispatch packets, block until done or
        Ctrl+C.

        Use this for standalone bots.  For embedding inside another asyncio
        program, use ``async with app.connect(addr)`` directly.
        """

        async def _runner() -> None:
            async with self.connect(addr):
                await self.serve()

        try:
            asyncio.run(_runner())
        except KeyboardInterrupt:
            pass

    async def _dispatch(self, raw: str) -> None:
        conn = self._conn
        packet = dispatch(raw)
        events: list[Any] = [packet]
        for mw in self._middleware:
            try:
                events.extend(await mw.on_packet(packet))
            except Exception as exc:
                self._on_error(exc, packet, mw.on_packet)
        for event in events:
            for h in self._handlers:
                await h.handle(event, conn)
