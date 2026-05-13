"""
Middleware Protocol - sits between the raw packet stream and handlers.

A ``Middleware`` implementation processes each incoming packet before handlers
run, can maintain state, and returns any synthetic events to inject into the
dispatch pipeline::

    from insim_o3.middleware import Middleware
    from insim_o3.packets import Ncn, Cnl

    class MyMiddleware:
        async def on_connect(self, client: Any) -> None:
            pass  # request initial state, start timers, etc.

        async def on_packet(self, packet: object) -> list[object]:
            ...
            return []

        async def on_shutdown(self) -> None:
            pass  # cleanup

Attach to a client via ``client.middleware.add(m)`` or the ``middleware=``
constructor argument.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Protocol, runtime_checkable

if TYPE_CHECKING:
    from insim_o3.connection import Connection


@runtime_checkable
class Middleware(Protocol):
    """
    Pre-dispatch hook that sits between the raw packet stream and handlers.

    All three methods are required. ``on_connect`` and ``on_shutdown`` may be
    no-ops if the middleware has no lifecycle needs.
    """

    async def on_connect(self, conn: Connection) -> None: ...

    async def on_packet(self, packet: object) -> list[Any]: ...

    async def on_shutdown(self) -> None: ...
