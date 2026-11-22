# kitcar-router-bot

A chat bot built with kitcar's `Router`: no `update` function, just a handler for each thing it cares about.

It shows:

- Typed `!` commands with `#[derive(ChatCommand)]`, including an admin-only `!kick`. Bad arguments get a usage reply for free.
- Greeting players as they join.
- A repeating announcement, and a clickable "Wave" button.
- `PenaltyClearer` running alongside it.

## Usage

kitcar is for multiplayer servers, so host a server (or use a dedicated host) first.

- Type `/insim 29999` in chat to start InSim.
- `cargo run -p kitcar-router-bot -- --addr 127.0.0.1:29999`
- Add `--admin-password <password>` if your host has one.
- Join, and type `!help`.
