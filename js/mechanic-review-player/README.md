# Mechanic Review Player

Headless playlist manifests plus a small Vite review UI for mechanic candidate
review. The generator emits candidates from configurable builtin Rust mechanic
heuristics.

## Generate a Flick Playlist

```sh
BALLCHASING_API_KEY="$(pass show ballchasing.com | sed -n 's/^api-key: //p')" \
  cargo run -p subtr-actor-tools --bin build_mechanic_review_playlist -- \
  --count 10 \
  --playlist ranked-duels \
  --mechanic default \
  --output .cache/mechanic-review-playlists/latest-mechanic-review.json
```

The generator downloads replay files into `.cache/mechanic-review-replays/`,
runs the selected detectors over each replay, and writes a playlist manifest
with manual item advancement and stop-at-end playback by default.

Useful options:

- `--id <ballchasing-id-or-url>` or `--ids-file <path>` to use specific
  Ballchasing replays.
- `--replay-path <path>` to use local replay files.
- `--min-confidence <f32>` to filter heuristic candidates.
- `--before-seconds <f32>` and `--after-seconds <f32>` to tune clip windows.
- `--max-items <n>` to cap the review list.
- `--mechanic <name>` or `--mechanics flick,one_timer,air_dribble` to choose
  detectors. Use `--mechanic all` for every supported event detector.
- `--list-mechanics` to print supported detector names.

## Run the Review UI

```sh
npm --prefix js/mechanic-review-player install
npm --prefix js/mechanic-review-player run dev -- --host 127.0.0.1 --port 5176
```

Then open:

```text
http://127.0.0.1:5176/?playlistUrl=/@fs/home/imalison/Projects/subtr-actor/.cache/mechanic-review-playlists/latest-mechanic-review.json
```

On GitHub Pages the same app is published under `/review/` and accepts either
`playlist` or `playlistUrl`:

```text
https://rlrml.github.io/subtr-actor/review/?playlist=https://example.com/playlist.json
```

The UI can also load a playlist through the file picker or URL field. The player
uses the playlist manifest as the source of truth, displays the reason each item
was included, and has a playlist list for direct jumps. Tagging persistence is
not implemented yet.
