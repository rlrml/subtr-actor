/**
 * FPS overlay plugin — a small HUD badge showing two distinct, live frame rates:
 *
 *  - **Render FPS**: the three.js render loop rate, measured from how often
 *    `beforeRender` actually fires (the rAF tick). This is the real GPU/CPU
 *    render throughput — typically 60/120/144 depending on the display.
 *  - **Replay FPS**: how many replay frames are *currently* going by per
 *    wall-clock second, measured from the per-frame `frameIndex` delta. Unlike
 *    the replay's fixed native sample rate (~30 Hz), this tracks live playback:
 *    ~30 at 1× speed, ~60 at 2×, 0 when paused, and spikes while scrubbing.
 *
 * Both are shown so it's obvious the smooth on-screen motion (render FPS) is
 * decoupled from the rate the underlying data is being consumed (replay FPS).
 * Opt in via the factory, like every other plugin:
 *
 *   createPlayer(container, bytes, { plugins: [createFpsOverlayPlugin()] })
 */
import type { PlayerPlugin } from "../types.js";

/** A measured rate sample, emitted to `onSample` each update window. */
export interface FpsSample {
  /** three.js render-loop rate (rAF ticks per wall second). */
  renderFps: number;
  /** Live replay-frame advance rate (replay frames per wall second). */
  replayFps: number;
}

export interface FpsOverlayOptions {
  /**
   * Headless mode: receive the measured rates each window and render them
   * yourself (e.g. into host-styled fields). When set, the plugin creates no
   * DOM of its own — `mount`/`corner` are ignored.
   */
  onSample?: (sample: FpsSample) => void;
  /**
   * Where to render the built-in badge. When omitted, it floats over the
   * viewport (pinned to `corner`). Provide an element (or a getter) to mount it
   * inline somewhere host-owned — e.g. a playback/transport bar.
   */
  mount?: HTMLElement | (() => HTMLElement | null);
  /** Viewport corner for the floating badge (default "top-right"). Ignored when `mount` is set. */
  corner?: "top-left" | "top-right" | "bottom-left" | "bottom-right";
  /** How often to refresh the displayed rates, in ms (default 500). */
  updateIntervalMs?: number;
}

const CORNER_STYLES: Record<NonNullable<FpsOverlayOptions["corner"]>, string> = {
  "top-left": "top: 8px; left: 8px;",
  "top-right": "top: 8px; right: 8px;",
  "bottom-left": "bottom: 8px; left: 8px;",
  "bottom-right": "bottom: 8px; right: 8px;",
};

export function createFpsOverlayPlugin(options: FpsOverlayOptions = {}): PlayerPlugin {
  const corner = options.corner ?? "top-right";
  const updateIntervalMs = options.updateIntervalMs ?? 500;
  const resolveMount = () =>
    typeof options.mount === "function" ? options.mount() : (options.mount ?? null);

  let root: HTMLDivElement | null = null;
  let renderValue: HTMLSpanElement | null = null;
  let replayValue: HTMLSpanElement | null = null;

  // Measurement window: count render ticks and accumulate replay-frame advance
  // over a wall-clock window, then derive both rates per second.
  let framesInWindow = 0;
  let windowStart = performance.now();
  let frameIndexAtWindowStart = 0;
  let currentFrameIndex = 0;

  const headless = typeof options.onSample === "function";

  return {
    id: "fps-overlay",
    setup(ctx) {
      windowStart = performance.now();
      framesInWindow = 0;
      currentFrameIndex = ctx.player.getState().frameIndex;
      frameIndexAtWindowStart = currentFrameIndex;
      // Headless: caller renders the numbers themselves; build no DOM.
      if (headless) return;

      const mountTarget = resolveMount();
      const inline = mountTarget != null;

      root = document.createElement("div");
      root.className = "player-fps-overlay";
      // Inline (mounted in a host bar): lay the two stats out in a row and let
      // the host own positioning. Floating: a pinned, translucent corner badge.
      root.style.cssText = inline
        ? `
          display: inline-flex; gap: 10px; align-items: center;
          font: 600 11px/1.35 ui-monospace, SFMono-Regular, Menlo, monospace;
          color: #c8d4e6; letter-spacing: 0.02em; white-space: nowrap;
        `
        : `
          position: absolute; ${CORNER_STYLES[corner]}
          z-index: 30; pointer-events: none; user-select: none;
          display: flex; gap: 10px;
          font: 600 11px/1.35 ui-monospace, SFMono-Regular, Menlo, monospace;
          color: #e8f0ff; background: rgba(12, 16, 24, 0.62);
          border: 1px solid rgba(255, 255, 255, 0.12); border-radius: 6px;
          padding: 4px 8px; letter-spacing: 0.02em; white-space: nowrap;
          text-shadow: 0 1px 2px rgba(0, 0, 0, 0.6);
        `;

      const renderRow = document.createElement("span");
      renderRow.append("Render ");
      renderValue = document.createElement("span");
      renderValue.style.color = "#7fd4ff";
      renderValue.textContent = "– fps";
      renderRow.append(renderValue);

      const replayRow = document.createElement("span");
      replayRow.append("Replay ");
      replayValue = document.createElement("span");
      replayValue.style.color = "#9affc0";
      replayValue.textContent = "– fps";
      replayRow.append(replayValue);

      root.append(renderRow, replayRow);

      if (mountTarget) {
        mountTarget.appendChild(root);
      } else {
        // The floating badge is absolutely positioned; anchor it to the
        // container by ensuring the container is a positioning context.
        if (getComputedStyle(ctx.container).position === "static") {
          ctx.container.style.position = "relative";
        }
        ctx.container.appendChild(root);
      }
    },
    beforeRender(ctx) {
      framesInWindow += 1;
      currentFrameIndex = ctx.frameIndex;
      const now = performance.now();
      const elapsed = now - windowStart;
      if (elapsed < updateIntervalMs) return;

      const seconds = elapsed / 1000;
      const renderFps = framesInWindow / seconds;
      // Live replay rate: how many replay frames actually went by this window.
      // abs() so scrubbing in either direction registers as activity.
      const replayFps = Math.abs(currentFrameIndex - frameIndexAtWindowStart) / seconds;

      if (options.onSample) {
        options.onSample({ renderFps, replayFps });
      } else {
        if (renderValue) renderValue.textContent = `${renderFps.toFixed(0)} fps`;
        if (replayValue) replayValue.textContent = `${replayFps.toFixed(0)} fps`;
      }

      framesInWindow = 0;
      windowStart = now;
      frameIndexAtWindowStart = currentFrameIndex;
    },
    teardown() {
      root?.remove();
      root = null;
      renderValue = null;
      replayValue = null;
    },
  };
}
