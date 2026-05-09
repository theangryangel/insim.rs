"""
insim_o3 - Python bindings for the insim.rs Insim library.

Quick start::

    from insim_o3 import Insim, handler
    from insim_o3.packets import Ncn, Mso

    class Bot(handler.Handler):
        @handler.on(Ncn)
        async def join(self, packet: Ncn) -> None:
            print(packet.pname)
"""

from importlib.metadata import PackageNotFoundError
from importlib.metadata import version as _pkg_version

from insim_o3 import handler
from insim_o3._insim import colour_spans, escape, strip_colours, unescape
from insim_o3.client import Insim
from insim_o3.dispatcher import AnyPacket
from insim_o3.test_client import TestClient

try:
    __version__ = _pkg_version("insim_o3")
except (
    PackageNotFoundError
):  # pragma: no cover - source checkouts without install metadata
    __version__ = "0.0.0+unknown"

__all__ = [
    "__version__",
    "AnyPacket",
    "Insim",
    "TestClient",
    "colour_spans",
    "escape",
    "handler",
    "strip_colours",
    "unescape",
]
