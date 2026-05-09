"""
Type stubs for the compiled PyO3 extension module `insim_o3._insim`.

The extension is built by maturin from `src/lib.rs`. These stubs give IDEs
and type-checkers full knowledge of the FFI boundary without requiring the
compiled `.so`/`.pyd` to be present at analysis time.
"""

from collections.abc import Awaitable

from insim_o3.packets import IsiFlag

class _Insim:
    """
    Raw FFI handle wrapping a spawned insim TCP connection.

    **Not part of the public API: use `insim_o3.client.Insim` instead.**

    The FFI boundary is deliberately narrow: every method accepts or returns
    plain JSON strings so that no PyO3 lifetime or newtype complexity leaks
    into the Python layer.  Every method is async - the awaitable resolves
    through ``pyo3-async-runtimes`` against the running asyncio loop.
    """

    @staticmethod
    def connect(
        addr: str,
        /,
        *,
        flags: list[IsiFlag] | None = None,
        iname: str | None = None,
        admin_password: str | None = None,
        interval_ms: int | None = None,
        prefix: str | None = None,
        capacity: int = 512,
    ) -> Awaitable[_Insim]:
        """
        Establish a TCP connection to LFS and return a ready handle.

        ``flags`` accepts a list of ``IsiFlag`` string-enum values from
        ``insim_o3.packets`` (e.g. ``[IsiFlag.MCI, IsiFlag.NLP]``).

        ``prefix`` must be a single character or ``None``.

        Raises ``ValueError`` for unrecognised flag names or a multi-character
        prefix (raised synchronously, before the awaitable is awaited).
        Raises ``RuntimeError`` on connection failure (raised when the
        awaitable is awaited).
        """
        ...

    def recv(self) -> Awaitable[str]:
        """
        Wait for the next packet from LFS.

        Returns a JSON string such as::

            '{"type": "Ncn", "reqi": 0, "ucid": 1, ...}'

        Raises ``RuntimeError`` when the connection is closed.
        """
        ...

    def send(self, data: str, /) -> Awaitable[None]:
        """
        Send a packet to LFS.

        ``data`` must be a JSON string with a ``"type"`` field matching a Rust
        ``Packet`` variant name, e.g.::

            '{"type": "Tiny", "reqi": 1, "subt": "Ping"}'

        Raises ``ValueError`` for malformed JSON (synchronously).
        Raises ``RuntimeError`` if the connection is dead (when awaited).
        """
        ...

    def shutdown(self) -> Awaitable[None]:
        """Signal the background network actor to stop gracefully."""
        ...

def strip_colours(input: str, /) -> str:
    """Strip LFS colour markers (``^0``–``^8``) from *input*.

    Escaped markers (``^^``) are preserved so they survive a subsequent
    :func:`unescape` call.  Always call this before :func:`unescape` when
    you need both operations.
    """
    ...

def unescape(input: str, /) -> str:
    """Unescape LFS escape sequences (e.g. ``^v`` → ``|``, ``^t`` → ``"``)."""
    ...

def escape(input: str, /) -> str:
    """Escape a string for sending to LFS (e.g. ``|`` → ``^v``, ``"`` → ``^t``)."""
    ...

def colour_spans(input: str, /) -> list[tuple[int, str]]:
    """Split *input* into ``(colour_index, text)`` spans.

    *colour_index* is ``0``–``8``, matching LFS ``^0``–``^8``.  Empty spans
    are omitted.  Text may still contain escaped sequences (``^^``); call
    :func:`unescape` on each span for the final display string.
    """
    ...
