"""
Example: reply to !help via Mtc, using the ``app.on`` decorator pattern.

Run against a local or remote LFS host::

    python examples/help_reply_on.py <host:port> [--admin-password PASSWORD]
"""

import argparse

from insim_o3 import App, Connection, strip_colours, unescape
from insim_o3.packets import IsiFlag, Mso, MsoUserType, Mtc, Ncn, SoundType


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Reply to !help via Mtc using the app.on decorator."
    )
    parser.add_argument("addr", help="InSim address, e.g. 127.0.0.1:29999")
    parser.add_argument("--admin-password", default=None)
    args = parser.parse_args()

    app = App(
        admin_password=args.admin_password,
        flags=[IsiFlag.MSO_COLS],
        prefix="!",
    )

    @app.on
    async def announce_join(packet: Ncn, conn: Connection) -> None:
        print(f"[join] {packet.uname} ({packet.pname})")

    @app.on
    async def reply_to_help(packet: Mso, conn: Connection) -> None:
        if packet.usertype != MsoUserType.Prefix:
            return
        command = unescape(strip_colours(packet.msg[packet.textstart :]))
        if command != "!help":
            return
        await conn.send(
            Mtc(
                reqi=0,
                ucid=packet.ucid,
                plid=packet.plid,
                sound=SoundType.SysMessage,
                text="Commands: !help",
            )
        )

    app.run(args.addr)


if __name__ == "__main__":
    main()
