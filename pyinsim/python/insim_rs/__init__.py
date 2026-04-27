"""
pyinsim — Python bindings for the insim.rs InSim library.

Packet types live in ``insim_rs._types`` (auto-generated — do not edit).
Regenerate after Rust changes with::

    cargo xtask pyinsim codegen

Quick start::

    from insim_rs import InsimClient, Handler
    from insim_rs._types import Ncn, Mso
"""

from insim_rs.client import InsimClient
from insim_rs.dispatcher import AnyPacket
from insim_rs.handler import Handler

__all__ = ["AnyPacket", "InsimClient", "Handler"]
