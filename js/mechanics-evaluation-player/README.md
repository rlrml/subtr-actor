# mechanics-evaluation-player

`js/mechanics-evaluation-player` is a dedicated manifest-driven replay
evaluation app for mechanic review workflows. It is intentionally separate from
[`js/example`](../example/README.md) and from the existing
`js/stat-evaluation-player`.

## Development

```bash
npm --prefix js/mechanics-evaluation-player install
npm --prefix js/mechanics-evaluation-player run dev
```

Open the Vite URL, then:

1. Choose a playlist manifest JSON file.
2. Choose the `.replay` files referenced by that manifest.
3. Click `Load Playlist`.

The app preloads all referenced replay sources by default so clip-to-clip
transitions stay fast.

To generate a manifest from the local exact flip reset training set:

```bash
cargo run --bin build_flip_reset_playlist_manifest
```

By default this writes:

```text
data/flip-reset-ground-truth-exact/flip-reset-playlist-manifest.json
```

using the first 30 positive replays under:

```text
data/flip-reset-ground-truth-exact/replays
```

## Manifest Format

The app uses the generic playlist manifest helpers from [`js/player`](../player/README.md).

Example:

```json
{
  "label": "flip reset review",
  "replays": [
    {
      "id": "game-1",
      "path": "2025-02-01/game-1.replay"
    },
    {
      "id": "game-2",
      "path": "2025-02-01/game-2.replay"
    }
  ],
  "items": [
    {
      "replay": "game-1",
      "start": { "kind": "time", "value": 41.2 },
      "end": { "kind": "time", "value": 46.8 },
      "label": "exact reset"
    },
    {
      "replay": "game-2",
      "start": { "kind": "frame", "value": 2330 },
      "end": { "kind": "frame", "value": 2475 },
      "label": "heuristic false positive"
    }
  ]
}
```

Notes:

- `items[].replay` refers to `replays[].id`.
- `replays[].path` is optional metadata used by the app when matching selected replay files.
- `start` and `end` must be explicit `{ kind, value }` bounds.
- `label` and `meta` are optional on both replay and item records.

The generated flip reset manifest also includes clip metadata such as:

- `player_id`
- `player_name`
- `event_frame`
- `event_time`
- `marker_position`

The mechanics evaluation app uses that metadata to draw a player ring and a
contact marker for exact flip reset clips.

Replay matching is intentionally simple:

- exact match on selected file name
- exact match on selected relative path when available
- fallback basename match for manifest paths

If multiple selected replay files share the same basename, use unique names or a
manifest path that resolves unambiguously.
