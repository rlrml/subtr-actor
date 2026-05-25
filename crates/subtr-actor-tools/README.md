# subtr-actor-tools

Development-only diagnostics and comparison tools for the core `subtr-actor`
crate.

Run tools from the workspace root with:

```sh
cargo run -p subtr-actor-tools --bin replay_probe -- metadata assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay
cargo run -p subtr-actor-tools --bin ballchasing_breakdown -- assets/example.replay assets/example.ballchasing.json
```
