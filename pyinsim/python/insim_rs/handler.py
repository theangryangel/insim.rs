"""
Class-based routing for InSim packets.

Usage::

    from insim_rs.handler import Handler
    from insim_rs._types import Ncn

    handler = Handler()

    @handler.on(Ncn)
    async def on_new_connection(packet: Ncn) -> None:
        print(f"Connected: {packet.pname} ({packet.uname})")
"""

from __future__ import annotations

from collections import defaultdict
from typing import Any, Callable, Awaitable, TypeVar

from pydantic import BaseModel

from insim_rs.dispatcher import AnyPacket

_T = TypeVar("_T", bound=BaseModel)
_HandlerFn = Callable[[Any], Awaitable[None]]


class Handler:
    """Collects async packet handlers registered with ``@handler.on(PacketType)``."""

    def __init__(self) -> None:
        self._handlers: dict[type[AnyPacket], list[_HandlerFn]] = defaultdict(list)

    def on(
        self,
        packet_class: type[_T],
    ) -> Callable[[Callable[[_T], Awaitable[None]]], Callable[[_T], Awaitable[None]]]:
        """Decorator that registers *fn* to be called whenever *packet_class* arrives.

        Multiple handlers for the same type are called in registration order.
        Handlers are always ``async def``.
        """

        def decorator(
            fn: Callable[[_T], Awaitable[None]],
        ) -> Callable[[_T], Awaitable[None]]:
            self._handlers[packet_class].append(fn)  # type: ignore[arg-type]
            return fn

        return decorator

    async def handle(self, packet: AnyPacket) -> None:
        """Dispatch *packet* to all registered handlers for its concrete type."""
        for fn in self._handlers.get(type(packet), []):
            await fn(packet)

    def __repr__(self) -> str:
        counts = {cls.__name__: len(h) for cls, h in self._handlers.items()}
        return f"Handler({counts})"
