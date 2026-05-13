"""
insim_o3 - Python bindings for the insim.rs InSim library.

Quick start::

    from insim_o3 import App
    from insim_o3.handler import Handler, on
    from insim_o3.packets import IsiFlag, Ncn

    class Bot(Handler):
        @on
        async def join(self, packet: Ncn, conn: Connection) -> None:
            print(packet.pname)

    app = App(flags=[IsiFlag.MSO_COLS], handlers=[Bot()])
    app.run("127.0.0.1:29999")
"""

from importlib.metadata import PackageNotFoundError
from importlib.metadata import version as _pkg_version

from insim_o3 import commands, handler
from insim_o3._insim import colour_spans, escape, strip_colours, unescape
from insim_o3.app import App
from insim_o3.connection import Connection
from insim_o3.dispatcher import AnyPacket
from insim_o3.handler import Handler, on
from insim_o3.middleware import Middleware

try:
    __version__ = _pkg_version("insim_o3")
except (
    PackageNotFoundError
):  # pragma: no cover - source checkouts without install metadata
    __version__ = "0.0.0+unknown"

__all__ = [
    "__version__",
    "AnyPacket",
    "App",
    "Connection",
    "Handler",
    "Middleware",
    "colour_spans",
    "commands",
    "escape",
    "handler",
    "on",
    "strip_colours",
    "unescape",
]
