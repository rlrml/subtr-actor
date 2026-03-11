# Stats Timeline Viewer

This is a dedicated replay-debugging app for `subtr-actor`.

It combines:

- the existing Ballchasing-style 3D replay player
- `ReplayDataCollector` frame data for rendering
- `StatsTimelineCollector` cumulative snapshots for live stat inspection

## Quick Start

From the repository root:

```bash
cd js/stats-timeline-viewer
npm install
npm run build-wasm
npm run dev
```

Open `http://localhost:5173`, then load a `.replay` file.

## What It Is For

Use this when you want to:

- watch the replay and inspect stat accumulation over time
- debug disagreements between reducer outputs and Ballchasing
- scrub to a specific moment and inspect the exact cumulative snapshot at that time

## Notes

- The app imports the shared WASM package from `js/pkg/`.
- It reuses the existing `js/example/src/player.js` viewer script, but the surrounding app is otherwise cleanly separated.
