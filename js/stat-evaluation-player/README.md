# @rlrml/stats-player

`@rlrml/stats-player` packages the stats-focused replay viewer UI
from this repository as a reusable npm library.

## Installation

```bash
npm install @rlrml/stats-player three
```

`three` is a peer dependency. The published package pulls in the matching
`@rlrml/subtr-actor` and `@rlrml/player` versions automatically.

## Usage

```ts
import { mountStatEvaluationPlayer } from "@rlrml/stats-player";

const root = document.getElementById("app");
if (!(root instanceof HTMLElement)) {
  throw new Error("Missing mount point");
}

const player = mountStatEvaluationPlayer(root);

// Later, when you want to tear down the viewer:
player.destroy();
```

The mounted UI exposes the same replay file chooser, replay camera controls,
timeline overlays, and per-module stat panels as the in-repo demo app.

## Mechanics Review Playlists

The stats player includes the mechanics review workflow that used to live in the
dedicated review app. It accepts playlist manifests from the file picker, the
Mechanics review window URL field, or the `playlist` / `playlistUrl` query
parameters.

Generate a review playlist with:

```sh
BALLCHASING_API_KEY="$(pass show ballchasing.com | sed -n 's/^api-key: //p')" \
  cargo run -p subtr-actor-tools --bin build_mechanic_review_playlist -- \
  --count 10 \
  --playlist ranked-duels \
  --mechanic default \
  --output .cache/mechanic-review-playlists/latest-mechanic-review.json
```

Then open the stats player with the playlist URL:

```text
http://127.0.0.1:5173/?playlistUrl=/@fs/home/imalison/Projects/subtr-actor/.cache/mechanic-review-playlists/latest-mechanic-review.json
```

The GitHub Pages build also serves the stats player under `/review/` for
backward-compatible review links:

```text
https://rlrml.github.io/subtr-actor/review/?playlist=https://example.com/playlist.json
```

The package also exposes two lighter composition surfaces:

```ts
import {
  createReplayReviewDataProviderFromLocation,
  mountReplayReview,
  mountStatsReport,
} from "@rlrml/stats-player";
```

- `mountStatsReport(root, { initialData })` renders the static report-style
  stats pages from an existing compact or materialized `StatsTimeline`. Compact
  timelines carry event streams plus scaffold frames; use
  `createStatsFrameLookup(statsTimeline)` when code needs per-frame partial
  sums.
- `mountReplayReview(root, { provider })` renders the shared review shell with
  full-page Stats and Viewer modes backed by one data provider.
- `createReplayReviewDataProviderFromLocation()` creates the GitHub Pages-style
  provider for `replayUrl`, compressed replay URL, and Ballchasing query params.

Host apps such as Rocket Sense can provide their own `ReplayReviewDataProvider`
that returns backend-precomputed stats immediately and only supplies a full
`ReplayLoadBundle` when replay playback is available.

To preload a replay from a URL, pass the replay file URL in the query string.
GitHub raw replay URLs work well because they are stable and include permissive
CORS headers:

```text
https://example.com/stats-player/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/fix-legacy-rigidbody-normalization/assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay
```

For shorter share links, encode the replay URL into the compressed `r=` query
parameter. This uses raw deflate plus base64url encoding, so it is deterministic
and does not depend on an external URL shortener:

```ts
import { encodeCompressedReplayUrl } from "@rlrml/stats-player";

const replayParam = encodeCompressedReplayUrl(
  "https://raw.githubusercontent.com/rlrml/subtr-actor/fix-legacy-rigidbody-normalization/assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay",
);
const link = `https://example.com/stats-player/?r=${replayParam}`;
```

The aliases `replay_url` and `replay` are also accepted for readable URLs.
The aliases `replayUrlZ` and `replay_url_z` are also accepted for compressed
URLs. Remote replay files must be served with CORS headers that allow the viewer
origin to fetch them.

To preload a replay from Ballchasing, pass the replay UUID with `ballchasing`:

```text
https://example.com/stats-player/?ballchasing=56889c3e-c420-45db-92fd-47ce2a3604b0
```

The aliases `ballchasingId`, `ballchasingUuid`, and `ballchasingReplay` are also
accepted. `ballchasingReplay` can be either a UUID or a
`https://ballchasing.com/replay/{uuid}` URL. Ballchasing file downloads use the
same public endpoint as the website download button,
`POST https://ballchasing.com/dl/replay/{uuid}`.

To inspect the stats-player configuration loaded from a share link, add
`cfgDebug=1` to the query string or hash. On page load, the browser console logs
the exact parsed URL parameters, the selected `cfg` source, the raw `cfg` text,
and the normalized decoded configuration. If both the query string and hash
contain `cfg`, the hash value is used and the debug log includes a warning.
The `cfg` value normally uses compressed base64url encoding, but URL-encoded
raw JSON is also accepted for tooling that needs to hand off configuration
without running the web encoder.

The package also exports the stat timeline helpers and overlay utilities used by
the viewer, so consumers can build their own derived UI around the same data.

## Development

```bash
npm --prefix js/stat-evaluation-player install
npm --prefix js/stat-evaluation-player run check
npm --prefix js/stat-evaluation-player run test
npm --prefix js/stat-evaluation-player run build
npm --prefix js/stat-evaluation-player run smoke:install
```

The build refreshes the local WASM bindings, emits the library bundle in
`dist/`, and writes declaration files alongside it.
