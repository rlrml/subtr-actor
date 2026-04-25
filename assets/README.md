These fixtures were downloaded from the Ballchasing API on 2026-03-10.

- `recent-ranked-doubles-2026-03-10`
  - replay id: `2075de8d-3bc8-40c0-ac04-30dd28ba92a8`
  - title: `2026-03-10.18.55 Quantavious1234 Ranked Doubles Win`
  - playlist: `ranked-doubles`
- `recent-ranked-standard-2026-03-10-a`
  - replay id: `6f54b9d6-8d8c-480e-99c5-2cf458e2bcc5`
  - title: `2026-03-10.18.55 hawkrn  Ranked Standard Win`
  - playlist: `ranked-standard`
- `recent-ranked-standard-2026-03-10-b`
  - replay id: `08589b1a-c6f5-4bf4-8d05-b2e1f124e8f1`
  - title: `2026-03-10.19.56 .estarl1n Ranked Standard Win`
  - playlist: `ranked-standard`

Each fixture uses a shared filename prefix in `assets/`:

- `<fixture>.ballchasing.json`: the replay stats JSON from `GET /api/replays/{id}`
- `<fixture>.replay`: the raw replay file from `GET /api/replays/{id}/file`
- `<fixture>.replay_id.txt`: the source replay id for easy re-fetching
