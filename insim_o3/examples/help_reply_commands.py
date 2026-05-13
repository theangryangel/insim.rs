"""
Example: reply to !help via Mtc, using the ``commands`` middleware + decorator.

Run against a local or remote LFS host::

    python examples/help_reply_commands.py <host:port> [--admin-password PASSWORD]
"""

import argparse

from insim_o3 import App, Connection
from insim_o3.commands import Command, CommandsMiddleware, command
from insim_o3.handler import Handler, on
from insim_o3.packets import IsiFlag, Mtc, Ncn, SoundType


class HelpHandler(Handler):
    """Replies to ``!help`` with a system message and announces joins."""

    @on
    async def announce_join(self, packet: Ncn, conn: Connection) -> None:
        print(f"[join] {packet.uname} ({packet.pname})")

    @command("help")
    async def help(self, cmd: Command, conn: Connection) -> None:
        await conn.send(
            Mtc(
                reqi=0,
                ucid=cmd.ucid,
                plid=cmd.plid,
                sound=SoundType.SysMessage,
                text="Commands: !help",
            )
        )


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Reply to !help via Mtc using the commands middleware."
    )
    parser.add_argument("addr", help="InSim address, e.g. 127.0.0.1:29999")
    parser.add_argument("--admin-password", default=None)
    args = parser.parse_args()

    app = App(
        admin_password=args.admin_password,
        flags=[IsiFlag.MSO_COLS],
        prefix="!",
        middleware=[CommandsMiddleware()],
        handlers=[HelpHandler()],
    )
    app.run(args.addr)


if __name__ == "__main__":
    main()
