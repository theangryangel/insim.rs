"""Generate python/insim_o3/packets.py from insim_schema.json.

The schema is the raw schemars output for insim::Packet. The Rust enum is
internally-tagged (`#[serde(tag = "type")]`), so the top-level shape is a
oneOf where each variant references a struct in `$defs` and carries a
discriminator const. We mirror that into Pydantic discriminated unions.

Naming rules for in-place enum variants (externally-tagged sum types like
AiInputType, Fuel, ObjectInfo, ...): each object variant becomes a class
named `<ParentName><PayloadKey>`; unit variants stay as `Literal[...]` in
the union alias.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from collections.abc import Iterable
from pathlib import Path
from typing import Any

# Integer formats from schemars map to Python `int` with bounds via Field.
# Anything wider than 32 bits is left as int - Python ints are arbitrary
# precision, the Pydantic bounds carry the actual range.
_INT_FORMATS = {
    "uint8",
    "int8",
    "uint16",
    "int16",
    "uint32",
    "int32",
    "uint64",
    "int64",
    "uint",
}
_FLOAT_FORMATS = {"float", "double"}

# Scalar `$defs` entries we inline at the use-site instead of emitting a
# class for. Mirrors the prior datamodel-codegen behavior.
_INLINE_SCALAR_KINDS = {"string", "integer", "number", "boolean"}


def main() -> int:
    script_dir = Path(__file__).resolve().parent
    insim_o3_root = script_dir.parent
    schema_path = insim_o3_root / "insim_schema.json"
    output_path = insim_o3_root / "python" / "insim_o3" / "packets.py"

    schema = json.loads(schema_path.read_text())
    gen = Generator(schema)
    output_path.write_text(gen.render())

    subprocess.run(
        ["uv", "tool", "run", "ruff", "format", str(output_path)],
        cwd=insim_o3_root,
        check=True,
    )
    print(f"wrote {output_path}")
    return 0


class Generator:
    def __init__(self, schema: dict[str, Any]) -> None:
        self.schema = schema
        self.defs: dict[str, dict[str, Any]] = schema.get("$defs", {})
        # Names of $defs entries that should be inlined at the use-site
        # rather than emitted as their own class (e.g. RequestId -> int).
        self.inline_aliases: dict[str, str] = {}
        self.imports: set[str] = set()
        self.imports_from: dict[str, set[str]] = {
            "enum": {"StrEnum"},
            "ipaddress": set(),
            "typing": {"Annotated", "Literal"},
            "pydantic": {"BaseModel", "ConfigDict", "Field"},
        }
        # Emitted class blocks, keyed by class name for dedup.
        self.blocks: dict[str, str] = {}
        # Order to emit blocks (alphabetical, but discovery order is fine
        # because we use string forward refs).
        self.block_order: list[str] = []
        # Packet variant classes in schema order (for AnyPacket).
        self.packet_variants: list[str] = []

    def render(self) -> str:
        self._classify_inline_aliases()
        self._emit_all_defs()
        self._emit_packet_union()
        return self._assemble()

    def _classify_inline_aliases(self) -> None:
        """Mark `$defs` entries that should be inlined at use-sites.

        These are simple scalar newtypes (RequestId, ClickId, Track,
        Vehicle, etc.) - types with no fields, no oneOf, no enum. Emitting
        a Pydantic class for them adds noise without adding type safety
        beyond the underlying primitive.
        """
        for name, d in self.defs.items():
            ty = d.get("type")
            if ty in _INLINE_SCALAR_KINDS and "enum" not in d and "oneOf" not in d:
                self.inline_aliases[name] = ty

    def _emit_all_defs(self) -> None:
        # Emit in alphabetical order for deterministic diffs. Pydantic
        # resolves forward references at model_rebuild time, so order does
        # not affect correctness.
        for name in sorted(self.defs):
            if name in self.inline_aliases:
                continue
            self._emit_def(name, self.defs[name])

    def _emit_def(self, name: str, d: dict[str, Any]) -> None:
        if "oneOf" in d:
            self._emit_oneof_def(name, d)
        elif "enum" in d:
            self._emit_str_enum(name, d["enum"], d.get("description"))
        elif d.get("type") == "array" and "items" in d:
            self._emit_array_alias(name, d)
        elif d.get("type") == "array" and "prefixItems" in d:
            self._emit_tuple_alias(name, d)
        elif d.get("type") == "object":
            self._emit_struct(name, d)
        else:
            raise NotImplementedError(f"unhandled $defs entry shape: {name} = {d!r}")

    def _emit_oneof_def(self, name: str, d: dict[str, Any]) -> None:
        variants = d["oneOf"]
        # Pure string-const / enum cases collapse to a StrEnum.
        if all(self._is_string_variant(v) for v in variants):
            values = self._collect_string_values(variants)
            self._emit_str_enum(name, values, d.get("description"))
            return

        # Externally-tagged enum (mixed object + const variants).
        self._emit_externally_tagged_union(name, d)

    @staticmethod
    def _is_string_variant(v: dict[str, Any]) -> bool:
        if v.get("type") != "string":
            return False
        return "const" in v or "enum" in v

    @staticmethod
    def _collect_string_values(variants: list[dict[str, Any]]) -> list[str]:
        out: list[str] = []
        for v in variants:
            if "const" in v:
                out.append(v["const"])
            elif "enum" in v:
                out.extend(v["enum"])
        return out

    def _emit_str_enum(
        self, name: str, values: Iterable[str], description: str | None = None
    ) -> None:
        lines = [f"class {name}(StrEnum):"]
        if description:
            lines.append(f"    {self._docstring(description)}")
        for v in values:
            ident = _safe_ident(v)
            lines.append(f"    {ident} = {v!r}")
        self._add_block(name, "\n".join(lines))

    def _emit_externally_tagged_union(self, name: str, d: dict[str, Any]) -> None:
        members: list[str] = []
        for v in d["oneOf"]:
            if "const" in v:
                members.append(f"Literal[{v['const']!r}]")
            elif v.get("type") == "object":
                class_name = self._emit_variant_class(name, v)
                members.append(class_name)
            else:
                raise NotImplementedError(f"unhandled variant shape in {name}: {v!r}")
        union = " | ".join(members)
        desc = d.get("description")
        block = [f"type {name} = {union}"]
        if desc:
            block.insert(0, f"# {name}: {description_first_line(desc)}")
        self._add_block(name, "\n".join(block))

    def _emit_variant_class(self, parent: str, variant: dict[str, Any]) -> str:
        # Variant shape from schemars for an externally-tagged enum:
        #   { type: object, properties: { K: <payload> }, required: [K],
        #     additionalProperties: false }
        # Name the class <Parent><K>.
        required = variant.get("required", [])
        if len(required) != 1:
            raise NotImplementedError(
                f"variant of {parent} has {len(required)} required keys, expected 1: "
                f"{variant!r}"
            )
        key = required[0]
        class_name = f"{parent}{key}"
        props = variant.get("properties", {})
        payload_schema = props.get(key, {})
        # If the payload is itself an inline anonymous object, hoist it to
        # a synthetic class so we can $ref it from the wrapper.
        if payload_schema.get("type") == "object" and "$ref" not in payload_schema:
            payload_name = f"{class_name}Payload"
            self._emit_struct(payload_name, payload_schema)
            field_type, field_extras = payload_name, {}
            if desc := variant.get("description") or payload_schema.get("description"):
                # Field-level description if the variant carries one.
                field_extras["description"] = desc
        else:
            field_type, field_extras = self._render_property(payload_schema)
        field_line = self._format_field(key, field_type, field_extras, required=True)
        body = [
            f"class {class_name}(BaseModel):",
            '    model_config = ConfigDict(extra="forbid")',
        ]
        desc = variant.get("description")
        if desc:
            body.insert(1, f"    {self._docstring(desc)}")
        body.append(f"    {field_line}")
        self._add_block(class_name, "\n".join(body))
        return class_name

    def _emit_struct(
        self,
        name: str,
        d: dict[str, Any],
        extra_fields: list[tuple[str, str, dict[str, Any], bool]] | None = None,
    ) -> None:
        """Emit a Pydantic BaseModel for an object schema.

        `extra_fields` lets the caller add synthetic fields (e.g., the
        `type: Literal["X"]` discriminator for top-level Packet variants).
        Each entry is (name, rendered_type, extras_dict, required).
        """
        properties = d.get("properties", {})
        required_set = set(d.get("required", []))
        forbid_extra = d.get("additionalProperties") is False

        lines = [f"class {name}(BaseModel):"]
        if forbid_extra:
            lines.append('    model_config = ConfigDict(extra="forbid")')
        desc = d.get("description")
        if desc:
            lines.append(f"    {self._docstring(desc)}")

        # Render in alphabetical key order - datamodel-codegen's behavior,
        # and stable across schemars releases.
        fields_out: list[tuple[str, str]] = []
        for key in sorted(properties):
            payload = properties[key]
            field_type, extras = self._render_property(payload)
            line = self._format_field(
                key, field_type, extras, required=key in required_set
            )
            fields_out.append((key, line))

        if extra_fields:
            for fname, ftype, fextras, freq in extra_fields:
                line = self._format_field(fname, ftype, fextras, required=freq)
                fields_out.append((fname, line))

        if not fields_out:
            lines.append("    pass")
        else:
            for _, line in fields_out:
                lines.append(f"    {line}")

        self._add_block(name, "\n".join(lines))

    def _emit_array_alias(self, name: str, d: dict[str, Any]) -> None:
        items = d["items"]
        inner, _ = self._render_property(items)
        self._add_block(name, f"type {name} = list[{inner}]")

    def _emit_tuple_alias(self, name: str, d: dict[str, Any]) -> None:
        # `Vector` is the only case: prefixItems with 3 floats, minItems=maxItems=3.
        items = d["prefixItems"]
        inner_types = [self._render_property(it)[0] for it in items]
        self._add_block(name, f"type {name} = tuple[{', '.join(inner_types)}]")

    def _render_property(self, p: dict[str, Any]) -> tuple[str, dict[str, Any]]:  # noqa: C901
        """Return (python_type, extras_for_Field).

        `extras` collects `description=`, `ge=`, `le=`, `min_length=`,
        `max_length=`, and similar - anything that should appear in the
        Field(...) call.
        """
        extras: dict[str, Any] = {}
        if desc := p.get("description"):
            extras["description"] = desc

        if "$ref" in p:
            inner_type, inner_extras = self._resolve_ref(p["$ref"])
            # Outer description (on the property) takes precedence; other
            # constraints from the alias (ge/le/min_length/...) flow through.
            for k, v in inner_extras.items():
                extras.setdefault(k, v)
            return inner_type, extras

        if "anyOf" in p:
            # Schemars emits Option<T> as [T, null].
            options = p["anyOf"]
            non_null = [o for o in options if o.get("type") != "null"]
            has_null = any(o.get("type") == "null" for o in options)
            if len(non_null) == 1 and has_null:
                inner, inner_extras = self._render_property(non_null[0])
                # Inner description wins if outer didn't set one.
                for k, v in inner_extras.items():
                    extras.setdefault(k, v)
                return f"{inner} | None", extras
            # General anyOf - render as union.
            parts = [self._render_property(o)[0] for o in options]
            return " | ".join(parts), extras

        if "const" in p:
            return f"Literal[{p['const']!r}]", extras

        if "enum" in p:
            parts = [f"Literal[{v!r}]" for v in p["enum"]]
            return " | ".join(parts), extras

        ty = p.get("type")
        if isinstance(ty, list):
            non_null = [t for t in ty if t != "null"]
            has_null = "null" in ty
            if len(non_null) == 1 and has_null:
                inner_schema = {**p, "type": non_null[0]}
                inner, inner_extras = self._render_property(inner_schema)
                for k, v in inner_extras.items():
                    extras.setdefault(k, v)
                return f"{inner} | None", extras
            raise NotImplementedError(f"unsupported type-array: {p!r}")
        if ty == "string":
            if p.get("format") == "ipv4":
                self.imports_from["ipaddress"].add("IPv4Address")
                return "IPv4Address", extras
            if "minLength" in p:
                extras["min_length"] = p["minLength"]
            if "maxLength" in p:
                extras["max_length"] = p["maxLength"]
            return "str", extras
        if ty == "integer":
            if "minimum" in p:
                extras["ge"] = p["minimum"]
            if "maximum" in p:
                extras["le"] = p["maximum"]
            return "int", extras
        if ty == "number":
            return "float", extras
        if ty == "boolean":
            return "bool", extras
        if ty == "array":
            inner, _ = self._render_property(p["items"])
            if "minItems" in p:
                extras["min_length"] = p["minItems"]
            if "maxItems" in p:
                extras["max_length"] = p["maxItems"]
            return f"list[{inner}]", extras
        if ty == "object":
            # Inline anonymous object - give it a synthetic name based on
            # context. None of the current schemas hit this path; if they
            # do, we want to know.
            raise NotImplementedError(f"inline anonymous object: {p!r}")
        if ty == "null":
            return "None", extras

        raise NotImplementedError(f"unrenderable property: {p!r}")

    def _resolve_ref(self, ref: str) -> tuple[str, dict[str, Any]]:
        if not ref.startswith("#/$defs/"):
            raise NotImplementedError(f"unsupported $ref: {ref}")
        name = ref[len("#/$defs/") :]
        if name in self.inline_aliases:
            # Inline scalar - fold its constraints (ge/le/min_length/...)
            # into the caller's Field extras. Drop the alias's own
            # description since the property usually has a more specific
            # one; if it doesn't, the caller has setdefault to fall back.
            inner, inner_extras = self._render_property(self.defs[name])
            inner_extras.pop("description", None)
            return inner, inner_extras
        return name, {}

    def _format_field(
        self,
        name: str,
        type_str: str,
        extras: dict[str, Any],
        *,
        required: bool,
    ) -> str:
        # Python keywords can't be field names; alias them.
        ident = name
        if _is_python_keyword(name):
            ident = f"{name}_"
            extras = {**extras, "alias": name}
        if not extras:
            annotated = type_str
        else:
            annotated = _annotated(type_str, extras)
        if not required:
            annotated = f"{annotated} | None"
            return f"{ident}: {annotated} = None"
        if "default" in extras:
            default = extras.pop("default")
            # Re-render with the default removed from Field().
            annotated = _annotated(type_str, extras) if extras else type_str
            return f"{ident}: {annotated} = {default!r}"
        return f"{ident}: {annotated}"

    @staticmethod
    def _docstring(desc: str) -> str:
        first = description_first_line(desc).replace('"""', "'''")
        return f'"""{first}"""'

    def _emit_packet_union(self) -> None:
        one_of = self.schema.get("oneOf", [])
        if not one_of:
            return
        variant_names: list[str] = []
        for variant in one_of:
            ref = variant.get("$ref")
            if not ref or not ref.startswith("#/$defs/"):
                raise NotImplementedError(f"unsupported Packet variant: {variant!r}")
            target = ref[len("#/$defs/") :]
            type_const = variant.get("properties", {}).get("type", {}).get("const")
            if type_const is None:
                raise NotImplementedError(
                    f"Packet variant missing type discriminator: {variant!r}"
                )
            # Re-emit the target def with an injected discriminator field.
            # The base def lives in self.defs (clean, schema-faithful); the
            # discriminator only exists on the Pydantic side to match the
            # wire format produced by serde's #[serde(tag = "type")].
            self._reemit_with_discriminator(target, type_const)
            variant_names.append(target)

        self.packet_variants = variant_names
        union = " | ".join(variant_names)
        self._add_block("AnyPacket", f"type AnyPacket = {union}")

    def _reemit_with_discriminator(self, name: str, type_const: str) -> None:
        """Replace the previously-emitted class for `name` with one that
        carries a `type: Literal[...]` discriminator field."""
        d = self.defs[name]
        if d.get("type") != "object":
            raise NotImplementedError(f"Packet variant {name} is not an object: {d!r}")
        # Drop any prior block for this name; we are replacing it.
        if name in self.blocks:
            del self.blocks[name]
            self.block_order.remove(name)
        extra = [
            (
                "type",
                f"Literal[{type_const!r}]",
                {"default": type_const},
                True,
            )
        ]
        self._emit_struct(name, d, extra_fields=extra)

    def _add_block(self, name: str, body: str) -> None:
        if name in self.blocks:
            # Idempotent: same name + same body is fine, otherwise complain.
            if self.blocks[name] != body:
                raise RuntimeError(f"duplicate emit for {name} with different body")
            return
        self.blocks[name] = body
        self.block_order.append(name)

    def _assemble(self) -> str:
        header = [
            "# Generated by insim_o3/scripts/generate_packets.py "
            "from insim_schema.json.",
            "# Do not edit by hand.",
            "",
            "from __future__ import annotations",
            "",
        ]
        # Build imports.
        import_lines: list[str] = []
        for module in sorted(self.imports_from):
            names = self.imports_from[module]
            if not names:
                continue
            import_lines.append(f"from {module} import {', '.join(sorted(names))}")
        body_blocks = [self.blocks[n] for n in self.block_order]
        return "\n".join(
            header + import_lines + ["", ""] + ["\n\n".join(body_blocks), ""]
        )


def _safe_ident(value: str) -> str:
    # Most enum values are already valid Python identifiers; the rare
    # exceptions need quoting or a trailing underscore for keywords.
    if value.isidentifier():
        if _is_python_keyword(value):
            return f"{value}_"
        return value
    return re.sub(r"\W|^(?=\d)", "_", value)


_PY_KEYWORDS = {
    "False",
    "None",
    "True",
    "and",
    "as",
    "assert",
    "async",
    "await",
    "break",
    "class",
    "continue",
    "def",
    "del",
    "elif",
    "else",
    "except",
    "finally",
    "for",
    "from",
    "global",
    "if",
    "import",
    "in",
    "is",
    "lambda",
    "nonlocal",
    "not",
    "or",
    "pass",
    "raise",
    "return",
    "try",
    "while",
    "with",
    "yield",
}


def _is_python_keyword(s: str) -> bool:
    return s in _PY_KEYWORDS


def _annotated(type_str: str, extras: dict[str, Any]) -> str:
    parts: list[str] = []
    for k, v in extras.items():
        if k == "default":
            continue
        parts.append(f"{k}={v!r}")
    field_call = f"Field({', '.join(parts)})"
    return f"Annotated[{type_str}, {field_call}]"


def description_first_line(desc: str) -> str:
    # Pydantic descriptions are arbitrary strings, but for docstrings on
    # the generated classes we keep just the first paragraph to avoid
    # giant comment blocks.
    return desc.split("\n", 1)[0].strip()


if __name__ == "__main__":
    sys.exit(main())
