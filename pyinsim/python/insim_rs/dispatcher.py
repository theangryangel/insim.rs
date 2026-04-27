"""
Dispatcher: converts a raw JSON string from the Rust FFI into a typed Pydantic
model from the generated ``_types`` module.

``_types.py`` is auto-generated — do not edit it by hand.
Regenerate with::

    cargo xtask pyinsim codegen
"""

from __future__ import annotations

import json
from typing import TYPE_CHECKING, Any, TypeAlias

from insim_rs._types import Packet

# AnyPacket is the union of all concrete InSim packet types.
# Derived at import time from the RootModel so it stays in sync with _types.py.
AnyPacket: TypeAlias = Packet.model_fields["root"].annotation  # type: ignore[assignment]


def dispatch(raw_json: str) -> AnyPacket:
    """Parse a raw JSON string from the Rust FFI into a typed ``Packet`` model.

    Uses Pydantic's built-in discriminated-union routing via the ``"type"``
    field that the Rust serializer always injects.  The ``Packet`` RootModel
    wrapper is unwrapped so callers receive the concrete typed instance
    (e.g. ``Ncn``, ``Mso``) directly.

    Raises ``pydantic.ValidationError`` on malformed input.
    """
    data: dict[str, Any] = json.loads(raw_json)
    return Packet.model_validate(data).root  # type: ignore[return-value]
