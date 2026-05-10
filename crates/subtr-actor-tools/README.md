# subtr-actor-tools

Development-only diagnostics and comparison tools for the core `subtr-actor`
crate.

Run tools from the workspace root with:

```sh
cargo run -p subtr-actor-tools --bin replay_probe -- metadata assets/rlcs.replay
cargo run -p subtr-actor-tools --bin ballchasing_breakdown -- assets/example.replay assets/example.ballchasing.json
```
