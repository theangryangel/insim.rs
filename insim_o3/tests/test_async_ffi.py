"""
Smoke tests that exercise the `_Insim` async FFI bridge.

These tests catch regressions the offline `TestClient` cannot:
- `connect()` raising `ValueError` synchronously (before the awaitable),
- `connect()` raising `RuntimeError` from the awaitable on a refused connection,
- the contract that `Insim` must be entered as an async context manager.

End-to-end packet round-trip tests require a real LFS instance and live
elsewhere.
"""

from __future__ import annotations

import pytest
from insim_o3 import Insim
from insim_o3.packets import Tiny


async def test_connect_invalid_flag_raises_value_error_synchronously() -> None:
    """Bad flag values surface as ValueError from connect, not as a future."""
    with pytest.raises(ValueError):
        async with Insim(
            "127.0.0.1:0",
            flags=["NOT_A_REAL_FLAG"],  # type: ignore[list-item]
        ):
            pass


async def test_send_before_connect_raises() -> None:
    """send() before __aenter__ tells the user to use the context manager."""
    client = Insim("127.0.0.1:0")
    with pytest.raises(RuntimeError, match="async with"):
        await client.send(Tiny(reqi=0, subt="Ping"))


async def test_connect_to_nothing_raises_runtime_error() -> None:
    """Loopback port 1 is virtually never listening - connect awaitable fails."""
    with pytest.raises(RuntimeError):
        async with Insim("127.0.0.1:1"):
            pass
