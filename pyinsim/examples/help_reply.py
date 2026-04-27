"""
Example: reply to !help with a text message via Mtc.

Run against a local or remote LFS host::

    python examples/help_reply.py <host:port> [--admin-password PASSWORD]
"""

import argparse

from insim_rs import Insim, strip_colours, unescape
from insim_rs._types import IsiFlag, Mso, MsoUserType, Mtc, SoundType

parser = argparse.ArgumentParser(description="Reply to !help via Mtc.")
parser.add_argument("addr", help="InSim address, e.g. 127.0.0.1:29999")
parser.add_argument("--admin-password", default=None)
args = parser.parse_args()

client = Insim(
    args.addr,
    admin_password=args.admin_password,
    flags=[IsiFlag.MSO_COLS],
    prefix="!",
)


@client.middleware
def log(packet_type: str, packet: object) -> None:
    print(f"← {packet_type}: {packet}")


@client.on(Mso)
def on_mso(packet: Mso) -> None:
    if packet.usertype != MsoUserType.Prefix:
        return
    command = unescape(strip_colours(packet.msg[packet.textstart :]))
    if command != "!help":
        return
    client.send(
        Mtc(
            reqi=0,
            ucid=packet.ucid,
            plid=packet.plid,
            sound=SoundType.SysMessage,
            text="Commands: !help" * 128,
        )
    )


client.run()
