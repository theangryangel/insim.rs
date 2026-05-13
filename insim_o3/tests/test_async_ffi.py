"""
Smoke tests for the ``App`` -> ``_Insim`` async FFI bridge.

These tests catch regressions the offline ``TestApp`` cannot:
- ``App.connect()`` raising ``ValueError`` for malformed config,
- ``App.connect()`` raising ``RuntimeError`` from the awaitable on a
  refused connection,
- ``App.serve()`` rejecting use before a connection has been established.

End-to-end packet round-trip tests require a real LFS instance and live
elsewhere.
"""

from __future__ import annotations

import pytest
from insim_o3 import App


async def test_connect_invalid_flag_raises_value_error() -> None:
    """Bad flag values surface as ValueError when entering the context."""
    app = App(flags=["NOT_A_REAL_FLAG"])  # type: ignore[list-item]
    with pytest.raises(ValueError):
        async with app.connect("127.0.0.1:0"):
            pass


async def test_serve_before_connect_raises() -> None:
    """serve() before connect() tells the user how to drive the App."""
    app = App()
    with pytest.raises(RuntimeError, match="connect"):
        await app.serve()


async def test_connect_to_nothing_raises_runtime_error() -> None:
    """Loopback port 1 is virtually never listening - connect awaitable fails."""
    app = App()
    with pytest.raises(RuntimeError):
        async with app.connect("127.0.0.1:1"):
            pass
