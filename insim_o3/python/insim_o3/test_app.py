"""
TestApp - a drop-in async replacement for ``App`` for unit tests.

Drives the full middleware + handler chain without a real LFS connection.
Use :class:`MockConnection` to inspect packets sent during a test::

    import pytest
    from insim_o3.test_app import TestApp, MockConnection
    from insim_o3.handler import Handler, on
    from insim_o3.packets import Ncn, Mtc, SoundType
    import json

    @pytest.mark.asyncio
    async def test_help_reply() -> None:
        conn = MockConnection()

        class Bot(Handler):
            @on
            async def on_ncn(self, packet: Ncn, conn: MockConnection) -> None:
                await conn.send_message(f"Welcome {packet.pname}", ucid=packet.ucid)

        app = TestApp(conn=conn, handlers=[Bot()])

        await app.inject(json.dumps({
            "type": "Ncn", "reqi": 0, "ucid": 1,
            "uname": "testuser", "pname": "Test Player",
            "admin": False, "total": 1, "flags": [],
        }))

        assert len(conn.sent) == 1
        assert conn.sent[0].ucid == 1
"""

from __future__ import annotations

from typing import Any

from pydantic import BaseModel

from insim_o3.dispatcher import dispatch
from insim_o3.handler import ErrorFn, Handler, default_on_error
from insim_o3.middleware import Middleware
from insim_o3.packets import Mst, Msx, Mtc, SoundType


class MockConnection:
    """Fake :class:`~insim_o3.connection.Connection` that records sent packets.

    Inject into :class:`TestApp` and inspect ``sent`` after driving events::

        conn = MockConnection()
        app = TestApp(conn=conn)
        await app.inject(raw_json)
        assert conn.sent[0].text == "hello"
    """

    def __init__(self) -> None:
        self.sent: list[BaseModel] = []

    async def send(self, packet: BaseModel) -> None:
        self.sent.append(packet)

    async def send_command(self, command: str) -> None:
        await self.send(Mst(reqi=0, msg=command))

    async def send_message(
        self,
        text: str,
        *,
        ucid: int | None = None,
        plid: int = 0,
        sound: SoundType = SoundType.Silent,
    ) -> None:
        if ucid is None:
            await self.send(Msx(reqi=0, msg=text))
        else:
            await self.send(Mtc(reqi=0, ucid=ucid, plid=plid, sound=sound, text=text))


class TestApp:
    """
    Minimal test double for :class:`~insim_o3.app.App`.

    Takes handlers and middleware at construction time, matching ``App``.
    No network connection - driven by :meth:`inject`.
    """

    # Tell pytest not to try to collect this as a test class despite the
    # `Test` prefix.
    __test__ = False

    def __init__(
        self,
        *,
        conn: MockConnection | None = None,
        on_error: ErrorFn | None = None,
        handlers: list[Handler] | None = None,
        middleware: list[Middleware] | None = None,
    ) -> None:
        self._handlers: tuple[Handler, ...] = tuple(handlers or ())
        self._middleware: tuple[Middleware, ...] = tuple(middleware or ())
        self._on_error: ErrorFn = on_error or default_on_error
        self.conn: MockConnection = conn or MockConnection()

    async def connect(self) -> None:
        """Call ``on_connect`` on all middleware, as the real app would."""
        for mw in self._middleware:
            await mw.on_connect(self.conn)  # type: ignore[arg-type]

    async def shutdown(self) -> None:
        """Call ``on_shutdown`` on all middleware, as the real app would."""
        for mw in self._middleware:
            await mw.on_shutdown()

    async def inject(self, raw_json: str) -> None:
        """Parse *raw_json* and run it through the full middleware + handler chain."""
        packet = dispatch(raw_json)
        events: list[Any] = [packet]
        for mw in self._middleware:
            try:
                events.extend(await mw.on_packet(packet))
            except Exception as exc:
                self._on_error(exc, packet, mw.on_packet)
        for event in events:
            for h in self._handlers:
                await h.handle(event, self.conn)
