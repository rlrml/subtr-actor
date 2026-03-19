# subtr-actor-stat-evaluation-player

`subtr-actor-stat-evaluation-player` packages the stats-focused replay viewer UI
from this repository as a reusable npm library.

## Installation

```bash
npm install subtr-actor-stat-evaluation-player three
```

`three` is a peer dependency. The published package pulls in the matching
`subtr-actor` and `subtr-actor-player` versions automatically.

## Usage

```ts
import { mountStatEvaluationPlayer } from "subtr-actor-stat-evaluation-player";

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
