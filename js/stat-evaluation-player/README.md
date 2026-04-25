# subtr-actor-stats-player

`subtr-actor-stats-player` packages the stats-focused replay viewer UI
from this repository as a reusable npm library.

## Installation

```bash
npm install subtr-actor-stats-player three
```

`three` is a peer dependency. The published package pulls in the matching
`@colonelpanic8/subtr-actor` and `subtr-actor-player` versions automatically.

## Usage

```ts
import { mountStatEvaluationPlayer } from "subtr-actor-stats-player";

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

To preload a replay from a URL, pass the replay file URL in the query string.
GitHub raw replay URLs work well because they are stable and include permissive
CORS headers:

```text
https://example.com/stats-player/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/fix-legacy-rigidbody-normalization/assets/dodges_refreshed_counter.replay
```

For shorter share links, encode the replay URL into the compressed `r=` query
parameter. This uses raw deflate plus base64url encoding, so it is deterministic
and does not depend on an external URL shortener:

```ts
import { encodeCompressedReplayUrl } from "subtr-actor-stats-player";

const replayParam = encodeCompressedReplayUrl(
  "https://raw.githubusercontent.com/rlrml/subtr-actor/fix-legacy-rigidbody-normalization/assets/dodges_refreshed_counter.replay",
);
const link = `https://example.com/stats-player/?r=${replayParam}`;
```

The aliases `replay_url` and `replay` are also accepted for readable URLs.
The aliases `replayUrlZ` and `replay_url_z` are also accepted for compressed
URLs. Remote replay files must be served with CORS headers that allow the viewer
origin to fetch them.

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
