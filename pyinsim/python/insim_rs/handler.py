"""
Class-based routing for InSim packets.

Usage::

    from insim_rs.handler import Handler
    from insim_rs._types import Ncn

    handler = Handler()

    @handler.on(Ncn)
    def on_new_connection(packet: Ncn) -> None:
        print(f"Connected: {packet.pname} ({packet.uname})")
"""

from __future__ import annotations

from collections import defaultdict
from typing import Any, Callable, TypeVar

from pydantic import BaseModel

from insim_rs.dispatcher import AnyPacket

_T = TypeVar("_T", bound=BaseModel)
_HandlerFn = Callable[[Any], None]


class Handler:
    """Collects packet handlers registered with ``@handler.on(PacketType)``."""

    def __init__(self) -> None:
        self._handlers: dict[type[AnyPacket], list[_HandlerFn]] = defaultdict(list)

    def on(
        self,
        packet_class: type[_T],
    ) -> Callable[[Callable[[_T], None]], Callable[[_T], None]]:
        """Decorator that registers *fn* to be called whenever *packet_class* arrives."""

        def decorator(fn: Callable[[_T], None]) -> Callable[[_T], None]:
            self._handlers[packet_class].append(fn)  # type: ignore[arg-type]
            return fn

        return decorator

    def handle(self, packet: AnyPacket) -> None:
        """Dispatch *packet* to all registered handlers."""
        for fn in self._handlers.get(type(packet), []):
            fn(packet)

    def __repr__(self) -> str:
        counts = {cls.__name__: len(h) for cls, h in self._handlers.items()}
        return f"Handler({counts})"
