"""
Class-based routing for InSim packets.

Subclass ``handler.Handler`` and decorate methods with ``@on`` to register
callbacks.  Packet types are inferred from the ``packet`` parameter annotation
and ``|`` unions are supported::

    from insim_o3 import handler
    from insim_o3.packets import Ncn, Mso, Cnl

    class Bot(handler.Handler):
        @handler.on
        async def join(self, packet: Ncn) -> None:
            print(f"join: {packet.pname}")

        @handler.on
        async def chat_or_leave(self, packet: Mso | Cnl) -> None:
            print(packet)

Attach the instance to a client::

    client.handlers.add(Bot())
"""

from __future__ import annotations

import inspect
import logging
import types as _types
from collections.abc import Awaitable, Callable
from typing import Any, ClassVar, Union, get_args, get_origin, get_type_hints

from pydantic import BaseModel

from insim_o3._registry import Registry
from insim_o3.dispatcher import AnyPacket

HandlerFn = Callable[..., None | Awaitable[None]]
_HandlerFn = HandlerFn
ErrorFn = Callable[[BaseException, AnyPacket, Callable[..., Any]], None]

_log = logging.getLogger(__name__)


def default_on_error(
    exc: BaseException, packet: AnyPacket, fn: Callable[..., Any]
) -> None:
    """Default ``ErrorFn``: log the exception and continue."""
    _log.exception(
        "callback %s raised on %s; continuing",
        fn,
        type(packet).__name__,
        exc_info=exc,
    )


_ON_ATTR = "_insim_o3_on"


def _infer_packet_types(fn: Callable[..., Any]) -> tuple[type[BaseModel], ...]:
    """Return the packet type(s) declared on the first non-self parameter of *fn*."""
    hints = get_type_hints(fn)
    params = [p for p in inspect.signature(fn).parameters if p not in ("self", "cls")]
    if not params:
        raise TypeError(f"'{fn.__name__}' needs a packet parameter")
    hint = hints.get(params[0], inspect.Parameter.empty)
    if hint is inspect.Parameter.empty:
        raise TypeError(f"'{fn.__name__}' packet parameter needs a type annotation")
    origin = get_origin(hint)
    if origin is Union or origin is _types.UnionType:
        packet_types: tuple[type, ...] = get_args(hint)
    else:
        packet_types = (hint,)
    for pt in packet_types:
        if not isinstance(pt, type):
            raise TypeError(f"Cannot route to non-type annotation: {pt}")
    return packet_types  # type: ignore[return-value]


def on[F: Callable[..., Any]](fn: F) -> F:
    """Decorator that registers a ``Handler`` method as a callback.

    Packet type(s) are inferred from the first parameter annotation.
    ``|`` union types route to multiple packet types.  Every callback
    receives the live ``Connection`` as its second argument::

        class Bot(Handler):
            @on
            async def join(self, packet: Ncn, conn: Connection) -> None:
                await conn.send_message(f"{packet.pname} joined")

            @on
            async def event(self, packet: Mso | Cnl, conn: Connection) -> None: ...
    """
    packet_types = _infer_packet_types(fn)
    existing: tuple[type[BaseModel], ...] = getattr(fn, _ON_ATTR, ())
    setattr(fn, _ON_ATTR, existing + packet_types)
    return fn


def _scan_class(cls: type) -> tuple[tuple[type[BaseModel], str], ...]:
    """Return ``(packet_class, attribute_name)`` pairs from ``@on``-tagged methods."""
    results: list[tuple[type[BaseModel], str]] = []
    for name, attr in cls.__dict__.items():
        if not callable(attr):
            continue
        tagged: tuple[type[BaseModel], ...] = getattr(attr, _ON_ATTR, ())
        for tag in tagged:
            results.append((tag, name))
    return tuple(results)


class Handler:
    """
    Routes packets to declared callbacks.

    Subclasses register callbacks at class definition time using ``@on``::

        class Joins(Handler):
            @on
            async def announce(self, packet: Ncn) -> None: ...

            @on
            async def session_event(self, packet: Mso | Cnl) -> None: ...

    Registrations are scanned once when the subclass is defined (via
    ``__init_subclass__``) and bound to each new instance in ``__init__``,
    so creating many instances is cheap and ``MyHandler._registrations`` is
    inspectable without instantiating.

    A failing handler is logged and skipped so one bad callback does not kill
    the dispatch loop.  Pass ``on_error`` to override (e.g. re-raise in tests).
    """

    _registrations: ClassVar[tuple[tuple[type[BaseModel], str], ...]] = ()

    def __init_subclass__(cls, **kwargs: Any) -> None:
        super().__init_subclass__(**kwargs)
        parent: tuple[tuple[type[BaseModel], str], ...] = ()
        for base in cls.__mro__[1:]:
            parent = getattr(base, "_registrations", ())
            if parent:
                break
        own = _scan_class(cls)
        seen: set[tuple[type[BaseModel], str]] = set()
        merged: list[tuple[type[BaseModel], str]] = []
        for pair in parent + own:
            if pair not in seen:
                seen.add(pair)
                merged.append(pair)
        cls._registrations = tuple(merged)

    def __init__(self, *, on_error: ErrorFn | None = None) -> None:
        self._handlers: dict[type[BaseModel], Registry[_HandlerFn]] = {}
        self._on_error: ErrorFn = on_error or default_on_error
        for packet_class, attr_name in type(self)._registrations:
            self._bucket(packet_class).add(getattr(self, attr_name))

    def register(self, fn: _HandlerFn) -> _HandlerFn:
        """Register *fn* as a callback, inferring packet type(s) from its annotation.

        Returns *fn* unchanged so this can be used as a decorator::

            @handler_instance.register
            async def join(packet: Ncn, conn: Connection) -> None: ...
        """
        for pt in _infer_packet_types(fn):
            self._bucket(pt).add(fn)
        return fn

    def _bucket(self, packet_class: type[BaseModel]) -> Registry[_HandlerFn]:
        bucket = self._handlers.get(packet_class)
        if bucket is None:
            bucket = Registry()
            self._handlers[packet_class] = bucket
        return bucket

    async def handle(self, packet: object, conn: object) -> None:
        """Dispatch *packet* to all registered callbacks.

        Sync and ``async def`` callbacks are both supported.  Exceptions are
        routed through ``on_error`` and do not interrupt remaining callbacks.

        *conn* is always forwarded as the second positional argument.
        """
        bucket = self._handlers.get(type(packet))
        if bucket is None:
            return
        for fn in bucket:
            try:
                result = fn(packet, conn)
                if inspect.isawaitable(result):
                    await result
            except Exception as exc:
                self._on_error(exc, packet, fn)

    def __repr__(self) -> str:
        counts = {cls.__name__: len(h) for cls, h in self._handlers.items()}
        return f"{type(self).__name__}({counts})"
