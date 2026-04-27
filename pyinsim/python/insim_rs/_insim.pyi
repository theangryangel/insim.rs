"""
Type stubs for the compiled PyO3 extension module `insim_rs._insim`.

The extension is built by maturin from `src/lib.rs`. These stubs give IDEs
and type-checkers full knowledge of the FFI boundary without requiring the
compiled `.so`/`.pyd` to be present at analysis time.
"""

class _Insim:
    """
    Raw FFI handle wrapping a spawned insim TCP connection.

    **Not part of the public API.** Use `insim_rs.client.Insim` instead.

    The FFI boundary is deliberately narrow: every method accepts or returns
    plain JSON strings so that no PyO3 lifetime or newtype complexity leaks
    into the Python layer.
    """

    @staticmethod
    def connect(
        addr: str,
        /,
        *,
        flags: list[str] | None = None,  # pass IsiFlag enum values
        iname: str | None = None,
        admin_password: str | None = None,
        interval_ms: int | None = None,
        prefix: str | None = None,
        capacity: int = 128,
    ) -> "_Insim":
        """
        Establish a TCP connection to LFS and return a ready handle.

        Blocks the calling thread while the handshake completes.
        Raises `RuntimeError` on connection failure.

        ``flags`` accepts a list of ``IsiFlag`` string-enum values from
        ``insim_rs._types`` (e.g. ``[IsiFlag.MCI, IsiFlag.NLP]``).  Because
        ``IsiFlag`` is a ``str`` subclass the values pass through the FFI
        boundary as plain strings.

        ``prefix`` must be a single character or ``None``.
        Raises ``ValueError`` for unrecognised flag names or a multi-character
        prefix.
        """
        ...

    def recv(self) -> str:
        """
        Block (releasing the GIL) until a packet arrives from LFS.

        Returns a JSON string such as::

            '{"type": "Ncn", "reqi": 0, "ucid": 1, ...}'

        Raises `RuntimeError` when the connection is closed.
        """
        ...

    def send(self, data: str, /) -> None:
        """
        Send a packet to LFS.

        `data` must be a JSON string with a ``"type"`` field matching a Rust
        ``Packet`` variant name, e.g.::

            '{"type": "Tiny", "reqi": 1, "subt": "Ping"}'

        Raises `ValueError` for malformed JSON.
        Raises `RuntimeError` if the connection is dead.
        """
        ...

    def shutdown(self) -> None:
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
