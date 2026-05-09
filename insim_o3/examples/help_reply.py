"""
Example: reply to !help with a text message via Mtc.

Run against a local or remote LFS host::

    python examples/help_reply.py <host:port> [--admin-password PASSWORD]
"""

import argparse
import asyncio

from insim_o3 import Insim, strip_colours, unescape
from insim_o3.packets import IsiFlag, Mso, MsoUserType, Mtc, SoundType


async def main() -> None:
    parser = argparse.ArgumentParser(description="Reply to !help via Mtc.")
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

        @client.on(Mso)
        async def on_mso(packet: Mso) -> None:
            if packet.usertype != MsoUserType.Prefix:
                return
            command = unescape(strip_colours(packet.msg[packet.textstart :]))
            if command != "!help":
                return
            try:
                await client.send(
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

        await client.run()


if __name__ == "__main__":
    asyncio.run(main())
