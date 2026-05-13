"""
Presence - connection and player tracking middleware.

Attach to a client to maintain live connection/player state and emit
typed synthetic events that handlers can subscribe to::

    from insim_o3 import Insim
    from insim_o3.presence import (
        PresenceMiddleware, Connected,
        Disconnected, PlayerJoined, PlayerLeft
    )
    from insim_o3.handler import Handler, on

    presence = PresenceMiddleware()

    class Bot(Handler):
        @on
        async def on_join(self, event: Connected) -> None:
            print(f"{event.conn.uname} connected ({len(presence.connections)} online)")

        @on
        async def on_leave(self, event: Disconnected) -> None:
            # full snapshot - presence.connections no longer has this ucid
            print(f"{event.conn.uname} left")

        @on
        async def on_player(self, event: PlayerJoined) -> None:
            print(f"{event.player.pname} entered the track")

    client = Insim("127.0.0.1:29999", middleware=[presence], handlers=[Bot()])

"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

from insim_o3.packets import Cnl, Cpr, Ncn, Npl, Pll, Tiny, TinyType, Toc

if TYPE_CHECKING:
    from insim_o3.connection import Connection


@dataclass
class ConnectionInfo:
    ucid: int
    uname: str
    pname: str
    admin: bool
    player_ids: set[int] = field(default_factory=set)


@dataclass
class PlayerInfo:
    plid: int
    ucid: int
    pname: str
    vehicle: str


@dataclass
class Connected:
    """A new connection joined the server."""

    conn: ConnectionInfo


@dataclass
class Disconnected:
    """A connection left. Full snapshot captured before removal."""

    conn: ConnectionInfo


@dataclass
class Renamed:
    """A connection changed their display name."""

    ucid: int
    uname: str
    new_pname: str


@dataclass
class PlayerJoined:
    """A player entered the track."""

    player: PlayerInfo


@dataclass
class PlayerLeft:
    """A player left the track. Full snapshot captured before removal."""

    player: PlayerInfo


@dataclass
class TakingOver:
    """A driver swap occurred."""

    before: PlayerInfo
    after: PlayerInfo


class PresenceMiddleware:
    """
    Middleware that tracks connections and players, emitting typed synthetic
    events as state changes occur.

    Query live state directly::

        presence.connections          # dict[ucid, ConnectionInfo]
        presence.players              # dict[plid, PlayerInfo]
        presence.last_known_names     # dict[uname, pname] - persists after disconnect
    """

    def __init__(self) -> None:
        self.connections: dict[int, ConnectionInfo] = {}
        self.players: dict[int, PlayerInfo] = {}
        self.last_known_names: dict[str, str] = {}

    async def on_connect(self, conn: Connection) -> None:
        await conn.send(Tiny(reqi=255, subt=TinyType.Ncn))
        await conn.send(Tiny(reqi=255, subt=TinyType.Npl))

    async def on_shutdown(self) -> None:
        self.connections.clear()
        self.players.clear()

    async def on_packet(self, packet: object) -> list[Any]:
        match packet:
            case Ncn():
                return self._ncn(packet)
            case Cnl():
                return self._cnl(packet)
            case Cpr():
                return self._cpr(packet)
            case Npl():
                return self._npl(packet)
            case Pll():
                return self._pll(packet)
            case Toc():
                return self._toc(packet)
            case Tiny(subt=TinyType.Clr):
                return self._clr()
            case _:
                return []

    def _ncn(self, packet: Ncn) -> list[object]:
        self.last_known_names[packet.uname] = packet.pname
        conn = ConnectionInfo(
            ucid=packet.ucid,
            uname=packet.uname,
            pname=packet.pname,
            admin=packet.admin,
        )
        self.connections[packet.ucid] = conn
        return [Connected(conn=conn)]

    def _cnl(self, packet: Cnl) -> list[object]:
        conn = self.connections.pop(packet.ucid, None)
        if conn is None:
            return []
        events: list[object] = []
        for plid in conn.player_ids:
            player = self.players.pop(plid, None)
            if player is not None:
                events.append(PlayerLeft(player=player))
        events.append(Disconnected(conn=conn))
        return events

    def _cpr(self, packet: Cpr) -> list[object]:
        conn = self.connections.get(packet.ucid)
        if conn is None:
            return []
        conn.pname = packet.pname
        self.last_known_names[conn.uname] = packet.pname
        return [Renamed(ucid=packet.ucid, uname=conn.uname, new_pname=packet.pname)]

    def _npl(self, packet: Npl) -> list[object]:
        if packet.nump == 0:
            # join request - real NPL with nump > 0 follows if accepted
            return []
        player = PlayerInfo(
            plid=packet.plid,
            ucid=packet.ucid,
            pname=packet.pname,
            vehicle=str(packet.cname),
        )
        self.players[packet.plid] = player
        if (conn := self.connections.get(packet.ucid)) is not None:
            conn.player_ids.add(packet.plid)
        return [PlayerJoined(player=player)]

    def _pll(self, packet: Pll) -> list[object]:
        player = self.players.pop(packet.plid, None)
        if player is None:
            return []
        if (conn := self.connections.get(player.ucid)) is not None:
            conn.player_ids.discard(player.plid)
        return [PlayerLeft(player=player)]

    def _toc(self, packet: Toc) -> list[object]:
        player = self.players.get(packet.plid)
        if player is None:
            return []
        before = PlayerInfo(**player.__dict__)
        player.ucid = packet.newucid
        after = PlayerInfo(**player.__dict__)
        if (old := self.connections.get(packet.olducid)) is not None:
            old.player_ids.discard(packet.plid)
        if (new := self.connections.get(packet.newucid)) is not None:
            new.player_ids.add(packet.plid)
        return [TakingOver(before=before, after=after)]

    def _clr(self) -> list[object]:
        events: list[object] = [PlayerLeft(player=p) for p in self.players.values()]
        self.players.clear()
        for conn in self.connections.values():
            conn.player_ids.clear()
        return events
