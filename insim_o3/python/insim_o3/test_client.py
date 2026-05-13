"""
TestClient - a drop-in async replacement for ``Insim`` that lets you inject
synthetic packets in unit tests without a real LFS connection.

Usage::

    import pytest
    from insim_o3.test_client import TestClient
    from insim_o3.handler import Handler, on
    from insim_o3.packets import Ncn
    import json

    @pytest.mark.asyncio
    async def test_ncn_handler() -> None:
        class Bot(Handler):
            def __init__(self) -> None:
                super().__init__()
                self.seen: list[Ncn] = []

            @on
            def on_ncn(self, p: Ncn) -> None:
                self.seen.append(p)

        bot = Bot()

        tc = TestClient()
        tc.handlers.add(bot)

        await tc.inject(json.dumps({
            "type": "Ncn",
            "reqi": 0,
            "ucid": 1,
            "uname": "testuser",
            "pname": "Test Player",
            "admin": False,
            "total": 1,
            "flags": [],
        }))

        assert len(bot.seen) == 1
        assert bot.seen[0].ucid == 1
"""

from __future__ import annotations

import inspect
from collections.abc import Awaitable, Callable

from insim_o3._registry import Registry
from insim_o3.dispatcher import AnyPacket, dispatch
from insim_o3.handler import ErrorFn, Handler, default_on_error

MiddlewareFn = Callable[[AnyPacket], None | Awaitable[None]]


class TestClient:
    """
    Minimal test double for ``Insim``.

    Mirrors ``Insim``'s public ``handlers`` and ``middleware`` registries, so
    test code can be written against the same contract as production code.
    No network connection, no Rust runtime - driven by ``inject()``.
    """

    def __init__(self, *, on_error: ErrorFn | None = None) -> None:
        self.handlers: Registry[Handler] = Registry()
        self.middleware: Registry[MiddlewareFn] = Registry()
        self._on_error: ErrorFn = on_error or default_on_error

    async def inject(self, raw_json: str) -> None:
        """
        Parse *raw_json* and run it through the full middleware + handler chain.
        """
        packet = dispatch(raw_json)

        # Iterating a Registry snapshots the underlying list, so callbacks
        # that mutate the registry mid-dispatch cannot skip siblings.
        for mw in self.middleware:
            try:
                result = mw(packet)
                if inspect.isawaitable(result):
                    await result
            except Exception as exc:
                self._on_error(exc, packet, mw)

        await asyncio.gather(*(h.handle(packet) for h in self.handlers))
