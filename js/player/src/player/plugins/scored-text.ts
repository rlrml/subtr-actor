/**
 * Scored-text overlay plugin — the big centered "<PLAYER> SCORED !!" banner the
 * original Ballcam player flashed on every goal.
 *
 * Reproduces the original look faithfully: a full-screen centered HUD banner,
 * 6rem Bourgeois (with a bold fallback), gold (#ffcb58) with the same layered
 * glow text-shadow. Opt in like any other `PlayerPlugin`, and toggle at runtime
 * via `player.addPlugin(...)` / `player.removePlugin("scored-text")`:
 *
 *   createPlayer(container, bytes, { plugins: [createScoredTextPlugin()] })
 *
 * Trigger model: the banner is wall-clock anchored — it appears the moment
 * forward playback crosses a goal and stays up for `durationSeconds` real
 * seconds. This survives the post-goal transition skip (which jumps the goal's
 * replay-time window). It *also* shows whenever the current replay time sits
 * inside a goal's window, so scrubbing onto / pausing near a goal reveals it too.
 */
import type { ReplayModel, ReplayTimelineEvent } from "../../types";
import type { PlayerPlugin, PlayerPluginContext, PlayerRenderContext } from "../types.js";

export interface ScoredTextOverlayOptions {
  /**
   * Seconds the banner stays up after a goal (default 4, matching the original
   * GameEngine's GOAL_TEXT_DURATION).
   */
  durationSeconds?: number;
  /**
   * Where to mount the banner. Defaults to the player container so it floats
   * full-screen and centered over the canvas.
   */
  mount?: HTMLElement | (() => HTMLElement | null);
  /**
   * Build the banner text from the scorer's name (empty string when unknown).
   * Default: `${name} SCORED !!` uppercased, or `GOAL !!` when no scorer.
   */
  formatText?: (scorerName: string) => string;
}

/** A goal reduced to what the banner needs. */
interface ScoredGoal {
  time: number;
  scorerName: string;
}

const DEFAULT_DURATION_SECONDS = 4;

function defaultFormatText(scorerName: string): string {
  const trimmed = scorerName.trim();
  return (trimmed ? `${trimmed} SCORED !!` : "GOAL !!").toUpperCase();
}

function collectGoals(replay: ReplayModel | null): ScoredGoal[] {
  if (!replay) return [];
  return replay.timelineEvents
    .filter((event: ReplayTimelineEvent) => event.kind === "goal")
    .map((event) => ({ time: event.time, scorerName: event.playerName ?? "" }))
    .sort((a, b) => a.time - b.time);
}

export function createScoredTextPlugin(options: ScoredTextOverlayOptions = {}): PlayerPlugin {
  const durationSeconds = options.durationSeconds ?? DEFAULT_DURATION_SECONDS;
  const formatText = options.formatText ?? defaultFormatText;
  const resolveMount = (ctx: PlayerPluginContext): HTMLElement =>
    (typeof options.mount === "function" ? options.mount() : options.mount) ?? ctx.container;

  let goals: ScoredGoal[] = [];
  let root: HTMLDivElement | null = null;
  let banner: HTMLDivElement | null = null;

  // Trigger bookkeeping (mirrors ActorManager's goal-explosion scan): fire once
  // per forward crossing, reset suppression on a backward seek / loop.
  let lastScanTime: number | null = null;
  const firedTimes = new Set<number>();
  // Wall-clock display window from the last crossed goal.
  let activeUntilMs = 0;
  let activeText = "";
  let shownText: string | null = null;

  function setVisible(text: string | null): void {
    if (!banner) return;
    if (text === shownText) return;
    shownText = text;
    if (text === null) {
      banner.style.opacity = "0";
      return;
    }
    banner.textContent = text;
    banner.style.opacity = "1";
  }

  return {
    id: "scored-text",
    setup(ctx): void {
      goals = collectGoals(ctx.replay);
      lastScanTime = null;
      firedTimes.clear();
      activeUntilMs = 0;
      activeText = "";
      shownText = null;

      const mount = resolveMount(ctx);
      // The full-screen banner is absolutely positioned; anchor it to a
      // positioned ancestor (match the fps-overlay convention).
      if (mount === ctx.container && getComputedStyle(ctx.container).position === "static") {
        ctx.container.style.position = "relative";
      }

      root = document.createElement("div");
      root.className = "player-scored-text-overlay";
      root.style.cssText = `
        position: absolute; inset: 0; z-index: 50;
        display: flex; align-items: center; justify-content: center;
        pointer-events: none; user-select: none;
      `;

      banner = document.createElement("div");
      banner.className = "player-scored-text";
      banner.style.cssText = `
        font-family: "Bourgeois", "Arial Black", system-ui, sans-serif;
        font-size: 6rem; line-height: 1; text-align: center;
        color: #ffcb58;
        text-shadow:
          0 6px 25px rgba(0, 0, 0, 0.35),
          0 0 4px #ffcb58, 0 0 8px #ffcb58, 0 0 15px #ffcb58;
        opacity: 0; transition: opacity 0.15s ease-out;
      `;

      root.appendChild(banner);
      mount.appendChild(root);
    },
    beforeRender(ctx: PlayerRenderContext): void {
      if (!banner || goals.length === 0) return;

      const now = ctx.time;
      const prev = lastScanTime;

      // Backward seek / loop: replay goals on the next pass and drop any stale
      // wall-clock window so we don't keep showing a goal we've rewound past.
      if (prev !== null && now < prev - 0.001) {
        firedTimes.clear();
        activeUntilMs = 0;
      }

      for (const goal of goals) {
        const crossedForward = prev !== null && prev < goal.time && now >= goal.time;
        if (crossedForward && !firedTimes.has(goal.time)) {
          firedTimes.add(goal.time);
          activeText = formatText(goal.scorerName);
          activeUntilMs = performance.now() + durationSeconds * 1000;
        }
      }
      lastScanTime = now;

      // Also show while the current replay time sits inside a goal's window, so
      // scrubbing onto / pausing near a goal reveals the banner (faithful to the
      // original's replay-time-window behavior).
      const windowGoal = goals.find(
        (goal) => now >= goal.time && now <= goal.time + durationSeconds,
      );
      const wallClockActive = performance.now() < activeUntilMs;

      if (windowGoal) {
        setVisible(formatText(windowGoal.scorerName));
      } else if (wallClockActive) {
        setVisible(activeText);
      } else {
        setVisible(null);
      }
    },
    teardown(): void {
      root?.remove();
      root = null;
      banner = null;
      goals = [];
      firedTimes.clear();
      shownText = null;
    },
  };
}
