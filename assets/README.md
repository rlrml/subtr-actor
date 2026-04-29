Replay-format fixtures use descriptive filenames:

`replay-format-<build-date>-v<major>-<minor>-net<net-or-none>-<signal>.replay`

The date in these names comes from the replay `BuildVersion` header. These
fixtures are the coverage set used by `docs/replay-format-evolution.md`.
Existing shorter fixture names remain checked in for compatibility with older
tests.

| Replay-format fixture | Source fixture |
| --- | --- |
| `replay-format-2016-07-21-v868-12-net-none-lan.replay` | `soccar-lan.replay` |
| `replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay` | `rlcs.replay` |
| `replay-format-2017-03-16-v868-17-net-none-online.replay` | `boxcars/assets/replays/good/2266.replay` |
| `replay-format-2017-11-22-v868-20-net2-legacy-vectors.replay` | `boxcars/assets/replays/good/netversion.replay` |
| `replay-format-2018-03-15-v868-20-net5-modern-vectors-legacy-rotation.replay` | `boxcars/assets/replays/good/db70.replay` |
| `replay-format-2018-05-17-v868-22-net7-modern-rigidbody.replay` | `boxcars/assets/replays/good/6cc24.replay` |
| `replay-format-2019-04-19-v868-24-net10-modern-rigidbody.replay` | `boxcars/assets/replays/good/70204.replay` |
| `replay-format-2020-09-25-v868-29-net10-tournament.replay` | `tourny.replay` |
| `replay-format-2022-09-29-v868-32-net10-legacy-boost.replay` | `old_boost_format.replay` |
| `replay-format-2025-06-10-v868-32-net10-replicated-boost.replay` | `new_boost_format.replay` |
| `replay-format-2026-01-14-v868-32-net10-demolish-extended.replay` | `new_demolition_format.replay` |
| `replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay` | `dodges_refreshed_counter.replay` |

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

Post-EAC fixtures were downloaded from public Ballchasing replay pages on
2026-04-29. The local Ballchasing API token was invalid at the time, so these
fixtures include replay files and source ids but not API stats JSON.

- `post-eac-ranked-duel-2026-04-28-a`
  - replay id: `8c0b4bf6-775b-4d4e-ac59-a0756805f94e`
  - title: `Ranked Duel Win 2026-04-28.09`
  - playlist: `ranked-duels`
- `post-eac-ranked-duel-2026-04-28-b`
  - replay id: `5a4e1f4f-8343-4ce6-948a-8353fe1a6941`
  - title: `Ranked Duel Win 2026-04-28.09`
  - playlist: `ranked-duels`
- `post-eac-ranked-doubles-2026-04-28`
  - replay id: `96f96431-634b-466e-af50-ca10521a6397`
  - title: `2026-04-28.18.07 2Fum2Tastic Ranked Doubles Loss`
  - playlist: `ranked-doubles`
- `post-eac-ranked-standard-2026-04-28`
  - replay id: `358c6147-3eae-489c-b8d4-c33f362386ca`
  - title: `2026-04-28.12.06 X2 ACID Ranked Standard Win`
  - playlist: `ranked-standard`
- `post-eac-private-2026-04-28`
  - replay id: `54272940-8859-4ee8-a37b-ca49328dbce7`
  - title: `2026-04-28.21.17 hashir Private Win`
  - playlist: `private`
