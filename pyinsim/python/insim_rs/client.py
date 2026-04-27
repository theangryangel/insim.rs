"""
InsimClient — the main public facade for the pyinsim library.

The user-facing API deliberately hides the raw ``_Insim`` FFI object.  All
packet I/O flows through the dispatcher so handlers always receive fully typed
Pydantic models.

Example::

    import asyncio
    from insim_rs import InsimClient, Handler
    from insim_rs._types import Ncn, Mso

    handler = Handler()

    @handler.on(Ncn)
    async def on_ncn(packet: Ncn) -> None:
        print(f"[join] {packet.pname}")

    @handler.on(Mso)
    async def on_mso(packet: Mso) -> None:
        print(f"[chat] {packet.msg}")

    async def main() -> None:
        async with InsimClient("127.0.0.1:29999") as client:
            client.include_handler(handler)
            await client.run()

    asyncio.run(main())
"""

from __future__ import annotations

import asyncio
from concurrent.futures import ThreadPoolExecutor
from typing import Any, AsyncGenerator, Callable, Awaitable, TypeVar

from pydantic import BaseModel

from insim_rs._insim import _Insim
from insim_rs.dispatcher import AnyPacket, dispatch
from insim_rs.handler import Handler

_T = TypeVar("_T", bound=BaseModel)

# Type alias for middleware callables.
# Middleware receives the variant name string and the parsed packet model.
MiddlewareFn = Callable[[str, AnyPacket], Awaitable[None]]


class InsimClient:
    """High-level InSim client.

    Wraps the raw ``_Insim`` FFI handle, dispatches packets to registered
    ``Handler`` instances, and supports one-shot ``wait_for`` queries and a
    middleware chain.
    """

    def __init__(self, addr: str) -> None:
        self._inner = _Insim.connect(addr)
        self._default_handler = Handler()
        self._handlers: list[Handler] = [self._default_handler]
        self._middleware: list[MiddlewareFn] = []
        # A single dedicated thread drives the blocking recv() call so the
        # asyncio event loop is never blocked.
        self._executor = ThreadPoolExecutor(
            max_workers=1, thread_name_prefix="pyinsim-recv"
        )

    def on(
        self,
        packet_class: type[_T],
    ) -> Callable[[Callable[[_T], Awaitable[None]]], Callable[[_T], Awaitable[None]]]:
        """Decorator that registers a handler directly on the client.

        Equivalent to creating a ``Handler`` and calling ``include_handler``,
        but more concise for simple setups::

            @client.on(Ncn)
            async def on_ncn(packet: Ncn) -> None:
                print(packet.pname)
        """
        return self._default_handler.on(packet_class)

    def include_handler(self, handler: Handler) -> None:
        """Add a ``Handler`` whose callbacks will be called for matching packets."""
        self._handlers.append(handler)

    def middleware(self, fn: MiddlewareFn) -> MiddlewareFn:
        """Register a middleware coroutine called for every received packet.

        Middleware runs before handlers and receives the raw variant name and
        the parsed packet::

            @client.middleware
            async def log_all(packet_type: str, packet: AnyPacket) -> None:
                print(f"← {packet_type}")
        """
        self._middleware.append(fn)
        return fn

    async def send(self, packet: BaseModel) -> None:
        """Send *packet* to LFS.

        The packet must be one of the models from ``insim_rs._types``.  Its
        ``model_dump(mode="json")`` is enriched with the ``"type"`` field and
        forwarded to the Rust FFI layer.
        """
        import json

        d = packet.model_dump(mode="json")
        # The generated models carry a ``type`` field with a Literal value, so
        # it is already present in the dump — no injection needed.
        raw = json.dumps(d)
        loop = asyncio.get_running_loop()
        await loop.run_in_executor(self._executor, lambda: self._inner.send(raw))

    async def _dispatch(self, raw: str) -> None:
        packet = dispatch(raw)
        packet_type_name = type(packet).__name__

        for mw in self._middleware:
            await mw(packet_type_name, packet)

        for h in self._handlers:
            await h.handle(packet)

    async def run(self) -> None:
        """Receive and dispatch packets until the connection drops or shutdown is called.

        Runs the blocking ``_Insim.recv()`` call in a ``ThreadPoolExecutor``
        so the asyncio event loop remains responsive.
        """
        loop = asyncio.get_running_loop()
        while True:
            raw = await loop.run_in_executor(self._executor, self._inner.recv)
            await self._dispatch(raw)

    def shutdown(self) -> None:
        """Signal the Rust actor to close the connection and clean up."""
        self._inner.shutdown()
        self._executor.shutdown(wait=False)

    async def stream(self, packet_class: type[_T]) -> AsyncGenerator[_T, None]:
        """Async generator that yields every *packet_class* packet as it arrives.

        Usage::

            async for packet in client.stream(Mso):
                print(packet.msg)

        The generator unregisters its internal handler when closed, so
        breaking out of the loop does not leave a dangling subscription.
        """
        queue: asyncio.Queue[_T] = asyncio.Queue()

        @self._default_handler.on(packet_class)
        async def _enqueue(packet: _T) -> None:
            await queue.put(packet)

        try:
            while True:
                yield await queue.get()
        finally:
            self._default_handler._handlers[packet_class].remove(_enqueue)

    async def __aenter__(self) -> "InsimClient":
        return self

    async def __aexit__(self, *_: object) -> None:
        self.shutdown()
