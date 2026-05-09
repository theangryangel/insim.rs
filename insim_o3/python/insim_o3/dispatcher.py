"""
Dispatcher: converts a raw JSON string from the Rust FFI into a typed Pydantic
model from the generated ``packets`` module.
"""

from __future__ import annotations

from typing import Annotated

from pydantic import Field, TypeAdapter

from insim_o3.packets import AnyPacket

__all__ = ["AnyPacket", "dispatch"]

# Discriminated union over the `type` tag set by every variant.  The
# TypeAdapter is built once at import time.  ``validate_json`` parses and
# dispatches in one step using pydantic-core's Rust JSON parser.
_PACKET_ADAPTER: TypeAdapter[AnyPacket] = TypeAdapter(
    Annotated[AnyPacket, Field(discriminator="type")]
)


def dispatch(raw_json: str) -> AnyPacket:
    """
    Parse a raw JSON string from the Rust FFI into a typed packet model.

    Raises ``pydantic.ValidationError`` on malformed input.
    """
    return _PACKET_ADAPTER.validate_json(raw_json)
