"""
Insim - the main public facade for the insim_o3 library.

For standalone bots, attach a ``Handler`` subclass and call
:meth:`Insim.run_forever`::

    from insim_o3 import Insim, handler
    from insim_o3.packets import Ncn

    class Bot(handler.Handler):
        @handler.on
        async def join(self, packet: Ncn) -> None:
            print(f"[join] {packet.pname}")

    client = Insim("127.0.0.1:29999")
    client.handlers.add(Bot())
    client.run_forever()

To embed inside another asyncio program, use ``Insim`` as an async context
manager directly::

    async def main() -> None:
        async with Insim("127.0.0.1:29999") as client:
            client.handlers.add(Bot())
            await client.run()
"""

from __future__ import annotations

import asyncio
import inspect
from collections.abc import Awaitable, Callable
from types import TracebackType

from pydantic import BaseModel

from insim_o3._insim import _Insim
from insim_o3._registry import Registry
from insim_o3.dispatcher import AnyPacket, dispatch
from insim_o3.handler import ErrorFn, Handler, default_on_error
from insim_o3.packets import IsiFlag, Mst, Msx, Mtc, SoundType

MiddlewareFn = Callable[[AnyPacket], None | Awaitable[None]]


class Insim:
    """High-level async InSim client.

    Wraps the raw ``_Insim`` FFI handle and dispatches packets to registered
    ``Handler`` instances. ``__init__`` only stores configuration - the actual
    TCP connection happens on ``__aenter__``.
    """

    def __init__(
        self,
        addr: str,
        *,
        flags: list[IsiFlag] | None = None,
        iname: str | None = None,
        admin_password: str | None = None,
        interval_ms: int | None = None,
        prefix: str | None = None,
        capacity: int = 512,
        on_error: ErrorFn | None = None,
    ) -> None:
        self._addr = addr
        self._flags = flags
        self._iname = iname
        self._admin_password = admin_password
        self._interval_ms = interval_ms
        self._prefix = prefix
        self._capacity = capacity
        self._inner: _Insim | None = None
        self._on_error: ErrorFn = on_error or default_on_error
        #: Attach ``Handler`` instances via ``client.handlers.add(h)``.
        self.handlers: Registry[Handler] = Registry()
        #: Attach middleware via ``client.middleware.add(fn)``.
        self.middleware: Registry[MiddlewareFn] = Registry()

    async def send(self, packet: BaseModel) -> None:
        """
        Send *packet* to LFS.
        """
        await self._require_connected().send(packet.model_dump_json())

    async def send_command(self, command: str) -> None:
        """
        Send *command* as an ``Mst`` packet (e.g. ``"/msg hello"``).
        """
        await self.send(Mst(reqi=0, msg=command))

    async def send_message(
        self,
        text: str,
        *,
        ucid: int | None = None,
        plid: int = 0,
        sound: SoundType = SoundType.Silent,
    ) -> None:
        """
        Send a chat message.

        With ``ucid=None`` (default) the message is broadcast as ``Msx``.
        With ``ucid`` set the message is delivered to that connection as an
        ``Mtc`` (and may carry a ``sound`` cue).
        """
        if ucid is None:
            await self.send(Msx(reqi=0, msg=text))
        else:
            await self.send(Mtc(reqi=0, ucid=ucid, plid=plid, sound=sound, text=text))

    async def _dispatch(self, raw: str) -> None:
        # Iterating a Registry snapshots the underlying list, so callbacks
        # that mutate the registry mid-dispatch cannot skip siblings.
        packet = dispatch(raw)
        for mw in self.middleware:
            try:
                result = mw(packet)
                if inspect.isawaitable(result):
                    await result
            except Exception as exc:
                self._on_error(exc, packet, mw)
        await asyncio.gather(*(h.handle(packet) for h in self.handlers))

    async def run(self) -> None:
        """
        Receive and dispatch packets until the connection closes or the task
        is cancelled.
        """
        inner = self._require_connected()
        while True:
            raw = await inner.recv()
            await self._dispatch(raw)

    def run_forever(self) -> None:
        """
        Synchronous entry point: connect, dispatch packets, block until the
        connection closes or Ctrl+C.

        Equivalent to::

            async def main() -> None:
                async with self:
                    await self.run()

            asyncio.run(main())

        Use this for standalone bots.  If you are embedding ``Insim`` inside
        another asyncio program (FastAPI, an MQTT client, etc.), use
        ``async with Insim(...) as client: await client.run()`` directly so
        you can compose with other tasks.
        """

        async def _runner() -> None:
            async with self:
                await self.run()

        try:
            asyncio.run(_runner())
        except KeyboardInterrupt:
            pass

    async def shutdown(self) -> None:
        """
        Signal the Rust actor to close the connection and wait for it to exit.
        """
        if self._inner is not None:
            await self._inner.shutdown()

    def _require_connected(self) -> _Insim:
        if self._inner is None:
            raise RuntimeError(
                "Insim must be used as `async with Insim(...) as client:`"
            )
        return self._inner

    async def __aenter__(self) -> Insim:
        self._inner = await _Insim.connect(
            self._addr,
            flags=self._flags,
            iname=self._iname,
            admin_password=self._admin_password,
            interval_ms=self._interval_ms,
            prefix=self._prefix,
            capacity=self._capacity,
        )
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        tb: TracebackType | None,
    ) -> None:
        await self.shutdown()
