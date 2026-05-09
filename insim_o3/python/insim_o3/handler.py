"""
Class-based routing for InSim packets.

Subclass ``handler.Handler`` and decorate methods with
``@handler.on(PacketType)`` to register callbacks at class definition time::

    from insim_o3 import handler
    from insim_o3.packets import Ncn, Mso

    class JoinsAndChat(handler.Handler):
        @handler.on(Ncn)
        async def join(self, packet: Ncn) -> None:
            print(f"join: {packet.pname}")

        @handler.on(Mso)
        async def chat(self, packet: Mso) -> None:
            print(f"chat: {packet.msg}")

Attach the instance to a client::

    client.handlers.add(JoinsAndChat())

The same verb ``on`` is used as the client-instance decorator
(``@client.on(PacketType)``) - in both places it registers a callback for a
packet type, just at different scopes.
"""

from __future__ import annotations

import inspect
import logging
from collections.abc import Awaitable, Callable
from typing import Any, ClassVar, TypeVar

from pydantic import BaseModel

from insim_o3._registry import Registry
from insim_o3.dispatcher import AnyPacket

_T = TypeVar("_T", bound=BaseModel)
# A handler callback may be sync (returning ``None``) or ``async def`` (returning
# an awaitable that resolves to ``None``).  ``Handler.handle`` awaits the result
# if it is awaitable.
HandlerFn = Callable[[_T], None | Awaitable[None]]
_HandlerFn = HandlerFn[Any]
# Third arg is the failing callback - typed loosely because both Handler
# (sync handler functions) and Insim (sync or async middleware) feed into it
# purely for repr/logging.
ErrorFn = Callable[[BaseException, AnyPacket, Callable[..., Any]], None]

_log = logging.getLogger(__name__)


def default_on_error(
    exc: BaseException, packet: AnyPacket, fn: Callable[..., Any]
) -> None:
    """Default ``ErrorFn``: log the exception and continue.

    Used by both ``Handler`` (for handler callbacks) and ``Insim`` (for
    middleware callbacks).  Override by passing ``on_error=...`` to either.
    """
    _log.exception(
        "callback %s raised on %s; continuing",
        fn,
        type(packet).__name__,
        exc_info=exc,
    )


_ON_ATTR = "_insim_o3_on"

_F = TypeVar("_F", bound=Callable[..., Any])


def on(*packet_classes: type[BaseModel]) -> Callable[[_F], _F]:
    """Decorator that registers a ``Handler`` method as a callback for one or
    more packet types.

    Used inside a ``Handler`` subclass body, the tag is picked up by
    ``Handler.__init_subclass__`` at class creation time.  Method names are
    arbitrary and a single method may register for multiple packet types::

        class HelpHandler(Handler):
            @on(Ncn)
            async def announce_join(self, packet: Ncn) -> None: ...

            @on(Mso, Cnl)
            async def session_event(self, packet) -> None: ...

    The same verb is used as ``client.on(PacketType)`` on an ``Insim``
    instance to register a callback at runtime against the client's default
    handler.
    """
    if not packet_classes:
        raise TypeError("on() requires at least one packet class")

    def decorator(fn: _F) -> _F:
        existing: tuple[type[BaseModel], ...] = getattr(fn, _ON_ATTR, ())
        # Setting a custom attribute on a callable trips the type-checker; the
        # tag is read back via ``getattr`` so the dynamic attribute is fine.
        setattr(fn, _ON_ATTR, existing + tuple(packet_classes))
        return fn

    return decorator


def _scan_class(cls: type) -> tuple[tuple[type[BaseModel], str], ...]:
    """Scan ``cls.__dict__`` for ``@on(...)`` tags.

    Returns ``(packet_class, attribute_name)`` pairs in source order.  The
    attribute name is resolved against the instance via ``getattr`` at
    construction time so subclass overrides win.
    """
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

    Subclasses register callbacks at class definition time using the
    ``@on(PacketType, ...)`` decorator (imported from ``insim_o3.handler``
    or ``insim_o3``).  Method names are arbitrary; one method may register
    for multiple packet types::

        class Joins(Handler):
            @on(Ncn)
            async def announce(self, packet: Ncn) -> None: ...

            @on(Mso, Cnl)
            async def session_event(self, packet) -> None: ...

    Registrations are scanned once when the subclass is defined (via
    ``__init_subclass__``) and bound to each new instance in ``__init__``,
    so creating many instances is cheap and ``MyHandler._registrations`` is
    inspectable without instantiating.

    A failing handler is logged and skipped so one bad callback does not kill
    the dispatch loop.  Pass ``on_error`` to override the default behaviour
    (e.g. re-raise during tests).
    """

    #: Cached ``(packet_class, attribute_name)`` pairs collected by
    #: ``__init_subclass__`` for this class and inherited from bases.
    _registrations: ClassVar[tuple[tuple[type[BaseModel], str], ...]] = ()

    def __init_subclass__(cls, **kwargs: Any) -> None:
        super().__init_subclass__(**kwargs)
        # Inherit from the nearest base that has declared registrations,
        # then layer on this class's own declarations.  De-duplicate
        # ``(packet_class, name)`` pairs in case a subclass tags or names a
        # method that the parent already registered for the same type.
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
        # Per-type buckets are ``Registry`` instances so iteration during
        # ``handle`` is safe under mutation.
        self._handlers: dict[type[BaseModel], Registry[_HandlerFn]] = {}
        self._on_error: ErrorFn = on_error or default_on_error
        for packet_class, attr_name in type(self)._registrations:
            self._bucket(packet_class).add(getattr(self, attr_name))

    def _bucket(self, packet_class: type[BaseModel]) -> Registry[_HandlerFn]:
        bucket = self._handlers.get(packet_class)
        if bucket is None:
            bucket = Registry()
            self._handlers[packet_class] = bucket
        return bucket

    async def handle(self, packet: AnyPacket) -> None:
        """
        Dispatch *packet* to all registered handlers.

        Sync and ``async def`` handlers are both supported - if a handler
        returns a coroutine it is awaited.  Exceptions raised by individual
        handlers are routed through ``on_error`` and do not interrupt the
        remaining handlers.
        """
        bucket = self._handlers.get(type(packet))
        if bucket is None:
            return
        for fn in bucket:
            try:
                result = fn(packet)
                if inspect.isawaitable(result):
                    await result
            except Exception as exc:
                self._on_error(exc, packet, fn)

    def __repr__(self) -> str:
        counts = {cls.__name__: len(h) for cls, h in self._handlers.items()}
        return f"{type(self).__name__}({counts})"
