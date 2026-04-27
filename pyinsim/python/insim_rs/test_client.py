"""
TestClient — a drop-in replacement for ``Insim`` that lets you inject
synthetic packets in unit tests without a real LFS connection.

Usage::

    import pytest
    from insim_rs.test_client import TestClient
    from insim_rs.handler import Handler
    from insim_rs._types import Ncn
    import json

    def test_ncn_handler() -> None:
        handler = Handler()
        seen: list[Ncn] = []

        @handler.on(Ncn)
        def on_ncn(p: Ncn) -> None:
            seen.append(p)

        tc = TestClient()
        tc.include_handler(handler)

        tc.inject(json.dumps({
            "type": "Ncn",
            "reqi": 0,
            "ucid": 1,
            "uname": "testuser",
            "pname": "Test Player",
            "admin": False,
            "total": 1,
            "flags": [],
        }))

        assert len(seen) == 1
        assert seen[0].ucid == 1
"""

from __future__ import annotations

from typing import Callable

from insim_rs.dispatcher import AnyPacket, dispatch
from insim_rs.handler import Handler

MiddlewareFn = Callable[[str, AnyPacket], None]


class TestClient:
    """Minimal test double for ``Insim``.

    Shares the same ``include_handler`` / ``middleware`` interface so test
    code can be written against the same contract as production code.
    No network connection, no Rust runtime — driven by ``inject()``.
    """

    def __init__(self) -> None:
        self._handlers: list[Handler] = []
        self._middleware: list[MiddlewareFn] = []

    def include_handler(self, handler: Handler) -> None:
        """Register a handler, mirroring ``Insim.include_handler``."""
        self._handlers.append(handler)

    def middleware(self, fn: MiddlewareFn) -> MiddlewareFn:
        """Register middleware, mirroring ``Insim.middleware``."""
        self._middleware.append(fn)
        return fn

    def inject(self, raw_json: str) -> None:
        """Parse *raw_json* and run it through the full middleware + handler chain."""
        packet = dispatch(raw_json)
        packet_type_name = type(packet).__name__

        for mw in self._middleware:
            mw(packet_type_name, packet)

        for h in self._handlers:
            h.handle(packet)
