"""
Example: reply to !help via Mtc, using the ``Handler`` subclass pattern.

Same behaviour as ``help_reply.py``, but groups the callbacks into a
reusable ``Handler`` class.  Methods are bound to packet types via the
``@on(PacketType)`` decorator; the same verb is used at the client level
(``@client.on(...)``) so the registration model is consistent.

Run against a local or remote LFS host::

    python examples/help_reply_handler.py <host:port> [--admin-password PASSWORD]
"""

import argparse
import asyncio

from insim_o3 import Insim, handler, strip_colours, unescape
from insim_o3.packets import IsiFlag, Mso, MsoUserType, Mtc, Ncn, SoundType


class HelpHandler(handler.Handler):
    """Replies to ``!help`` with a system message and announces joins."""

    def __init__(self, client: Insim) -> None:
        super().__init__()
        self._client = client

    @handler.on(Ncn)
    async def announce_join(self, packet: Ncn) -> None:
        print(f"[join] {packet.uname} ({packet.pname})")

    @handler.on(Mso)
    async def reply_to_help(self, packet: Mso) -> None:
        if packet.usertype != MsoUserType.Prefix:
            return
        command = unescape(strip_colours(packet.msg[packet.textstart :]))
        if command != "!help":
            return
        try:
            await self._client.send(
                Mtc(
                    reqi=0,
                    ucid=packet.ucid,
                    plid=packet.plid,
                    sound=SoundType.SysMessage,
                    text="Commands: !help - asdsd?",
                )
            )
        except RuntimeError as e:
            print(f"send error (non-fatal): {e}")


async def main() -> None:
    parser = argparse.ArgumentParser(
        description="Reply to !help via Mtc using a Handler subclass."
    )
    parser.add_argument("addr", help="InSim address, e.g. 127.0.0.1:29999")
    parser.add_argument("--admin-password", default=None)
    args = parser.parse_args()

    async with Insim(
        args.addr,
        admin_password=args.admin_password,
        flags=[IsiFlag.MSO_COLS],
        prefix="!",
    ) as client:

        @client.middleware.add
        async def log(packet: object) -> None:
            print(f"<- {type(packet).__name__}: {packet}")

        client.handlers.add(HelpHandler(client))

        await client.run()


if __name__ == "__main__":
    asyncio.run(main())
