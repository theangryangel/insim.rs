"""
Internal: a small list-backed collection that doubles as a decorator target.

Both ``Insim.handlers`` and ``Insim.middleware`` are exposed as ``Registry``
instances so users write::

    client.handlers.add(my_handler)
    client.handlers.remove(my_handler)

    @client.middleware.add
    async def log(packet): ...

Iteration snapshots the underlying list, so a callback that mutates the
registry mid-dispatch (e.g. one-shot removal) does not skip siblings.
"""

from __future__ import annotations

from collections.abc import Iterator


class Registry[R]:
    """Ordered list of items with ``add`` / ``remove`` / iteration."""

    __slots__ = ("_items",)

    def __init__(self) -> None:
        self._items: list[R] = []

    def add(self, item: R) -> R:
        """Append *item* and return it unchanged so this works as a decorator."""
        self._items.append(item)
        return item

    def remove(self, item: R) -> bool:
        """
        Remove the first occurrence of *item*.

        Returns ``True`` if found and removed, ``False`` otherwise.
        """
        try:
            self._items.remove(item)
        except ValueError:
            return False
        return True

    def __iter__(self) -> Iterator[R]:
        # Snapshot so callbacks that mutate the registry during dispatch do
        # not skip siblings.
        return iter(tuple(self._items))

    def __len__(self) -> int:
        return len(self._items)

    def __contains__(self, item: object) -> bool:
        return item in self._items

    def __repr__(self) -> str:
        return f"Registry({self._items!r})"
