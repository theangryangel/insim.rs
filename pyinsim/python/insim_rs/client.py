"""
Insim — the main public facade for the pyinsim library.

Usage::

    from insim_rs import Insim
    from insim_rs._types import Ncn, IsiFlag

    client = Insim("127.0.0.1:29999", flags=[IsiFlag.MCI], interval_ms=500)

    @client.on(Ncn)
    def on_ncn(packet: Ncn) -> None:
        print(f"[join] {packet.pname}")

    client.run()
"""

from __future__ import annotations

import json
from typing import Callable, TypeVar

from pydantic import BaseModel

from insim_rs._insim import _Insim
from insim_rs.dispatcher import AnyPacket, dispatch
from insim_rs.handler import Handler

_T = TypeVar("_T", bound=BaseModel)

MiddlewareFn = Callable[[str, AnyPacket], None]


class Insim:
    """High-level InSim client.

    Wraps the raw ``_Insim`` FFI handle and dispatches packets to registered
    ``Handler`` instances.
    """

    def __init__(
        self,
        addr: str,
        *,
        flags: list[str] | None = None,  # pass IsiFlag enum values
        iname: str | None = None,
        admin_password: str | None = None,
        interval_ms: int | None = None,
        prefix: str | None = None,
        capacity: int = 128,
    ) -> None:
        self._inner = _Insim.connect(
            addr,
            flags=flags,
            iname=iname,
            admin_password=admin_password,
            interval_ms=interval_ms,
            prefix=prefix,
            capacity=capacity,
        )
        self._default_handler = Handler()
        self._handlers: list[Handler] = [self._default_handler]
        self._middleware: list[MiddlewareFn] = []

    def on(
        self,
        packet_class: type[_T],
    ) -> Callable[[Callable[[_T], None]], Callable[[_T], None]]:
        """Decorator that registers a handler directly on the client."""
        return self._default_handler.on(packet_class)

    def include_handler(self, handler: Handler) -> None:
        """Add a ``Handler`` whose callbacks will be called for matching packets."""
        self._handlers.append(handler)

    def middleware(self, fn: MiddlewareFn) -> MiddlewareFn:
        """Register middleware called for every received packet before handlers."""
        self._middleware.append(fn)
        return fn

    def send(self, packet: BaseModel) -> None:
        """Send *packet* to LFS (blocking, releases the GIL)."""
        raw = json.dumps(packet.model_dump(mode="json"))
        self._inner.send(raw)

    def _dispatch(self, raw: str) -> None:
        packet = dispatch(raw)
        packet_type_name = type(packet).__name__
        for mw in self._middleware:
            mw(packet_type_name, packet)
        for h in self._handlers:
            h.handle(packet)

    def run(self) -> None:
        """Receive and dispatch packets, blocking the calling thread."""
        try:
            while True:
                raw = self._inner.recv()
                self._dispatch(raw)
        except KeyboardInterrupt:
            self.shutdown()

    def shutdown(self) -> None:
        """Signal the Rust actor to close the connection and clean up."""
        self._inner.shutdown()
