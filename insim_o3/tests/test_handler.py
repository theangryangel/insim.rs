"""
Integration tests for the Handler + TestApp layer.

These tests run without a real LFS connection by injecting synthetic JSON
strings directly through the dispatcher.  They verify that:

- Subclass-declared handlers receive a correctly typed Pydantic model.
- ``@on`` infers packet types from annotations, including ``|`` unions.
- Inheritance and override behave as expected.
- Unregistered packet types are silently ignored.
- Multiple handlers for the same type all fire, in declaration order.
- ``on_error`` isolates a failing handler so siblings still fire.
"""

from __future__ import annotations

import json

from insim_o3 import handler
from insim_o3.packets import Mso, Ncn
from insim_o3.test_app import MockConnection, TestApp

NCN_JSON = json.dumps(
    {
        "type": "Ncn",
        "reqi": 0,
        "ucid": 42,
        "uname": "testuser",
        "pname": "Test Player",
        "admin": False,
        "total": 5,
        "flags": [],
    }
)

MSO_JSON = json.dumps(
    {
        "type": "Mso",
        "reqi": 0,
        "ucid": 0,
        "plid": 0,
        "usertype": "System",
        "textstart": 0,
        "msg": "Hello, world!",
    }
)

TINY_JSON = json.dumps({"type": "Tiny", "reqi": 1, "subt": "Ping"})


async def test_ncn_handler_receives_typed_model() -> None:
    """Handler receives an ``Ncn`` model with correct field values."""

    class _Capture(handler.Handler):
        def __init__(self) -> None:
            super().__init__()
            self.received: list[Ncn] = []

        @handler.on
        async def collect(self, packet: Ncn, conn: MockConnection) -> None:
            self.received.append(packet)

    cap = _Capture()
    app = TestApp(handlers=[cap])
    await app.inject(NCN_JSON)

    assert len(cap.received) == 1
    pkt = cap.received[0]
    assert pkt.ucid == 42
    assert pkt.uname == "testuser"
    assert pkt.pname == "Test Player"
    assert pkt.admin is False
    assert pkt.total == 5
    assert pkt.flags == []


async def test_mso_handler_receives_typed_model() -> None:
    """Handler receives an ``Mso`` model with correct field values."""

    class _Capture(handler.Handler):
        def __init__(self) -> None:
            super().__init__()
            self.received: list[Mso] = []

        @handler.on
        async def collect(self, packet: Mso, conn: MockConnection) -> None:
            self.received.append(packet)

    cap = _Capture()
    app = TestApp(handlers=[cap])
    await app.inject(MSO_JSON)

    assert len(cap.received) == 1
    assert cap.received[0].msg == "Hello, world!"
    assert cap.received[0].usertype == "System"


async def test_unregistered_type_is_ignored() -> None:
    """Injecting a packet type with no handler does not raise."""
    app = TestApp(handlers=[handler.Handler()])
    await app.inject(TINY_JSON)


async def test_on_decorator_supports_union_types() -> None:
    """@on with a ``|`` union annotation registers the method for both types."""
    seen: list[str] = []

    class Multi(handler.Handler):
        @handler.on
        async def both(self, packet: Ncn | Mso, conn: MockConnection) -> None:
            seen.append(type(packet).__name__)

    app = TestApp(handlers=[Multi()])
    await app.inject(NCN_JSON)
    await app.inject(MSO_JSON)

    assert seen == ["Ncn", "Mso"]


async def test_class_registrations_are_inheritable() -> None:
    """Subclassing a Handler subclass inherits its registrations."""
    seen: list[str] = []

    class Base(handler.Handler):
        @handler.on
        async def handle_ncn(self, packet: Ncn, conn: MockConnection) -> None:
            seen.append("base-ncn")

    class Child(Base):
        @handler.on
        async def handle_mso(self, packet: Mso, conn: MockConnection) -> None:
            seen.append("child-mso")

    app = TestApp(handlers=[Child()])
    await app.inject(NCN_JSON)
    await app.inject(MSO_JSON)

    assert seen == ["base-ncn", "child-mso"]


async def test_subclass_override_replaces_parent_method() -> None:
    """An override of a parent's @on method dispatches to the override only."""
    seen: list[str] = []

    class Base(handler.Handler):
        @handler.on
        async def handle_ncn(self, packet: Ncn, conn: MockConnection) -> None:
            seen.append("base")

    class Child(Base):
        async def handle_ncn(self, packet: Ncn, conn: MockConnection) -> None:
            seen.append("child")

    app = TestApp(handlers=[Child()])
    await app.inject(NCN_JSON)

    assert seen == ["child"]


async def test_failing_handler_does_not_block_others() -> None:
    """An exception in one handler is routed to on_error and others still fire."""
    errors: list[tuple[type, str]] = []
    seen: list[int] = []

    def on_error(exc: BaseException, packet: object, _fn: object) -> None:
        errors.append((type(exc), type(packet).__name__))

    class _H(handler.Handler):
        @handler.on
        async def boom(self, packet: Ncn, conn: MockConnection) -> None:
            raise ValueError("kaboom")

        @handler.on
        async def ok(self, packet: Ncn, conn: MockConnection) -> None:
            seen.append(1)

    app = TestApp(handlers=[_H(on_error=on_error)])
    await app.inject(NCN_JSON)

    assert seen == [1]
    assert errors == [(ValueError, "Ncn")]


async def test_multiple_handlers_fire_in_order() -> None:
    """Multiple methods for the same type fire in declaration order."""
    order: list[int] = []

    class _H(handler.Handler):
        @handler.on
        async def first(self, packet: Ncn, conn: MockConnection) -> None:
            order.append(1)

        @handler.on
        async def second(self, packet: Ncn, conn: MockConnection) -> None:
            order.append(2)

    app = TestApp(handlers=[_H()])
    await app.inject(NCN_JSON)

    assert order == [1, 2]


async def test_multiple_handler_instances_all_receive_packet() -> None:
    """Packets are dispatched to every registered Handler instance."""
    hits: list[str] = []

    class _H1(handler.Handler):
        @handler.on
        async def on_ncn(self, packet: Ncn, conn: MockConnection) -> None:
            hits.append("h1")

    class _H2(handler.Handler):
        @handler.on
        async def on_ncn(self, packet: Ncn, conn: MockConnection) -> None:
            hits.append("h2")

    app = TestApp(handlers=[_H1(), _H2()])
    await app.inject(NCN_JSON)

    assert hits == ["h1", "h2"]
