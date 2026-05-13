"""
Example: connection and player tracking using PresenceMiddleware.

Demonstrates:
- Querying live state from the presence dict
- Handling synthetic events emitted by PresenceMiddleware

Run against a local or remote LFS host::

    python examples/presence.py <host:port> [--admin-password PASSWORD]
"""

import argparse

from insim_o3 import App, Connection
from insim_o3.handler import Handler, on
from insim_o3.packets import IsiFlag
from insim_o3.presence import (
    Connected,
    Disconnected,
    PlayerJoined,
    PlayerLeft,
    PresenceMiddleware,
    Renamed,
)


class PresenceBot(Handler):
    def __init__(self, presence: PresenceMiddleware) -> None:
        super().__init__()
        self._presence = presence

    @on
    async def on_connected(self, event: Connected, conn: Connection) -> None:
        total = len(self._presence.connections)
        print(f"[+] {event.conn.uname} ({event.conn.pname}) - {total} online")

    @on
    async def on_disconnected(self, event: Disconnected, conn: Connection) -> None:
        total = len(self._presence.connections)
        print(f"[-] {event.conn.uname} ({event.conn.pname}) - {total} remaining")

    @on
    async def on_renamed(self, event: Renamed, conn: Connection) -> None:
        print(f"[~] {event.uname} renamed to {event.new_pname}")

    @on
    async def on_player_joined(self, event: PlayerJoined, conn: Connection) -> None:
        print(f"[track+] {event.player.pname} in {event.player.vehicle}")

    @on
    async def on_player_left(self, event: PlayerLeft, conn: Connection) -> None:
        print(f"[track-] {event.player.pname}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Presence tracking example.")
    parser.add_argument("addr", help="InSim address, e.g. 127.0.0.1:29999")
    parser.add_argument("--admin-password", default=None)
    args = parser.parse_args()

    presence = PresenceMiddleware()

    app = App(
        admin_password=args.admin_password,
        flags=[IsiFlag.MSO_COLS],
        middleware=[presence],
        handlers=[PresenceBot(presence)],
    )
    app.run(args.addr)


if __name__ == "__main__":
    main()
