"""
Connection - a live TCP connection to an LFS host.

Wraps the raw ``_Insim`` FFI handle and exposes only the ability to send
packets.  Injected into handler callbacks that declare a ``conn: Connection``
parameter.
"""

from __future__ import annotations

from pydantic import BaseModel

from insim_o3._insim import _Insim
from insim_o3.packets import Mst, Msx, Mtc, SoundType


class Connection:
    """Live connection to an LFS host.  Passed to handlers that need to send."""

    def __init__(self, inner: _Insim) -> None:
        self._inner = inner

    async def send(self, packet: BaseModel) -> None:
        """Send *packet* to LFS."""
        await self._inner.send(packet.model_dump_json())

    async def send_command(self, command: str) -> None:
        """Send *command* as an ``Mst`` packet (e.g. ``"/msg hello"``)."""
        await self.send(Mst(reqi=0, msg=command))

    async def send_message(
        self,
        text: str,
        *,
        ucid: int | None = None,
        plid: int = 0,
        sound: SoundType = SoundType.Silent,
    ) -> None:
        """
        Send a chat message.

        With ``ucid=None`` (default) the message is broadcast as ``Msx``.
        With ``ucid`` set the message is delivered to that connection as an
        ``Mtc`` (and may carry a ``sound`` cue).
        """
        if ucid is None:
            await self.send(Msx(reqi=0, msg=text))
        else:
            await self.send(Mtc(reqi=0, ucid=ucid, plid=plid, sound=sound, text=text))
