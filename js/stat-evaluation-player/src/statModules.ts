import { ReplayPlayer } from "subtr-actor-player";
import type {
  FrameRenderInfo,
  ReplayModel,
  ReplayTimelineEvent,
  ReplayTimelineRange,
} from "subtr-actor-player";
import type { Object3D } from "three";
import { FiftyFiftyOverlay } from "./fiftyFiftyOverlay.ts";
import {
  renderFiftyFiftySummary,
  renderPlayerFiftyFiftyStats,
} from "./fiftyFiftyFormatting.ts";
import {
  formatCollectedWithRespawnBound,
  formatBoostDisplayAmount,
  toBoostDisplayUnits,
} from "./boostFormatting.ts";
import { renderMovementStats } from "./movementFormatting.ts";
import type { MovementBreakdownClass } from "./movementFormatting.ts";
import {
  HalfFieldOverlay,
  ThresholdZoneOverlay,
  createZoneBoundaryLines,
} from "./overlays.ts";
import { renderPossessionStats } from "./possessionFormatting.ts";
import type { PossessionBreakdownClass } from "./possessionFormatting.ts";
import { renderPressureStats } from "./pressureFormatting.ts";
import { renderRushStats } from "./rushFormatting.ts";
import { SpeedFlipOverlay } from "./speedFlipOverlay.ts";
import { getStatsFrameForReplayFrame } from "./statsTimeline.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsTimeline,
} from "./statsTimeline.ts";
import { renderTouchStats } from "./touchFormatting.ts";
import type { TouchBreakdownClass } from "./touchFormatting.ts";
import { TouchEventOverlay, playerIdToString } from "./touchOverlay.ts";
import {
  buildFiftyFiftyTimelineEvents,
  buildMustyFlickTimelineEvents,
  buildRushTimelineEvents,
} from "./timelineMarkers.ts";
import {
  buildPossessionTimelineRanges,
  buildPressureTimelineRanges,
  buildTimeInZoneTimelineRanges,
} from "./timelineRanges.ts";

export interface StatModuleContext {
  player: ReplayPlayer;
  replay: ReplayModel;
  statsTimeline: StatsTimeline;
  statsFrameLookup: Map<number, StatsFrame>;
  fieldScale: number;
}

export interface StatModule {
  readonly id: string;
  readonly label: string;
  setup(ctx: StatModuleContext): void;
  teardown(): void;
  onBeforeRender(info: FrameRenderInfo): void;
  getTimelineEvents?(ctx: StatModuleContext): ReplayTimelineEvent[];
  getTimelineRanges?(ctx: StatModuleContext): ReplayTimelineRange[];
  renderStats(frameIndex: number, ctx: StatModuleContext): string;
  renderSettings?(ctx: StatModuleContext | null): HTMLElement | null;
  renderFocusedPlayerStats(
    playerId: string,
    frameIndex: number,
    ctx: StatModuleContext,
  ): string;
}

export interface StatModuleRuntime {
  rerenderCurrentState(): void;
}

const MOST_BACK_FORWARD_THRESHOLD_Y = 236.0;

export const RELATIVE_POSITIONING_MODULE_ID = "relative-positioning";

export type Role = "back" | "forward" | "other" | "mid";

export const ROLE_LABELS: Record<Role, string> = {
  back: "Back",
  forward: "Fwd",
  other: "Other",
  mid: "Mid",
};

export function getTeamClass(isTeamZero: boolean): string {
  return isTeamZero ? "team-blue" : "team-orange";
}

type StatCardTone = "blue" | "orange" | "shared";

function renderStatCard(
  name: string,
  bodyHtml: string,
  options: {
    metaHtml?: string;
    tone: StatCardTone;
  },
): string {
  const toneClass = options.tone === "shared"
    ? "shared"
    : options.tone === "blue"
      ? "team-blue"
      : "team-orange";

  return `<div class="player-card ${toneClass}">
    <div class="player-card-header">
      <span class="player-name">${name}</span>
      ${options.metaHtml ?? ""}
    </div>
    ${bodyHtml}
  </div>`;
}

function renderPlayerCard(
  name: string,
  isTeamZero: boolean,
  bodyHtml: string,
  metaHtml = "",
): string {
  return renderStatCard(name, bodyHtml, {
    metaHtml,
    tone: isTeamZero ? "blue" : "orange",
  });
}

function renderSharedCard(
  name: string,
  bodyHtml: string,
  metaHtml = "",
): string {
  return renderStatCard(name, bodyHtml, {
    metaHtml,
    tone: "shared",
  });
}

export function getStatsPlayerSnapshot(
  ctx: StatModuleContext,
  frameIndex: number,
  playerId: string,
): PlayerStatsSnapshot | null {
  const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
  if (!statsFrame) return null;

  return statsFrame.players.find(
    (player) => playerIdToString(player.player_id) === playerId,
  ) ?? null;
}

export function getCurrentRole(
  replay: ReplayModel,
  playerId: string,
  frameIndex: number,
): Role {
  const player = replay.players.find((candidate) => candidate.id === playerId);
  if (!player) return "mid";

  const frame = player.frames[frameIndex];
  if (!frame?.position) return "mid";

  const isTeamZero = player.isTeamZero;
  const teamRosterCount = replay.players.filter(
    (candidate) => candidate.isTeamZero === isTeamZero,
  ).length;
  const allYs: number[] = [];
  let normalizedY = 0;

  for (const candidate of replay.players) {
    if (candidate.isTeamZero !== isTeamZero) continue;
    const candidateFrame = candidate.frames[frameIndex];
    if (!candidateFrame?.position) continue;

    const value = isTeamZero
      ? candidateFrame.position.y
      : -candidateFrame.position.y;
    allYs.push(value);
    if (candidate.id === playerId) {
      normalizedY = value;
    }
  }

  if (teamRosterCount < 2 || allYs.length !== teamRosterCount) return "mid";

  const minY = Math.min(...allYs);
  const maxY = Math.max(...allYs);
  const spread = maxY - minY;

  if (spread <= MOST_BACK_FORWARD_THRESHOLD_Y) return "other";

  const nearBack = normalizedY - minY <= MOST_BACK_FORWARD_THRESHOLD_Y;
  const nearFront = maxY - normalizedY <= MOST_BACK_FORWARD_THRESHOLD_Y;

  if (nearBack && !nearFront) return "back";
  if (nearFront && !nearBack) return "forward";
  return "mid";
}

function disposeIfPossible(value: unknown): void {
  if (
    value &&
    typeof value === "object" &&
    "dispose" in value &&
    typeof value.dispose === "function"
  ) {
    value.dispose();
  }
}

function disposeZoneBoundaryLines(
  zoneBoundaryLines: ReturnType<typeof createZoneBoundaryLines> | null,
): void {
  if (!zoneBoundaryLines) {
    return;
  }

  zoneBoundaryLines.removeFromParent();
  zoneBoundaryLines.traverse((node: Object3D) => {
    const geometry = "geometry" in node ? node.geometry : null;
    disposeIfPossible(geometry);

    const material = "material" in node ? node.material : null;
    if (Array.isArray(material)) {
      for (const entry of material) {
        disposeIfPossible(entry);
      }
    } else {
      disposeIfPossible(material);
    }
  });
}

function createSharedZoneBoundaryOverlayManager() {
  let refCount = 0;
  let zoneBoundaryLines: ReturnType<typeof createZoneBoundaryLines> | null = null;

  return {
    acquire(ctx: StatModuleContext): void {
      if (!zoneBoundaryLines) {
        zoneBoundaryLines = createZoneBoundaryLines(
          ctx.player.sceneState.scene,
          ctx.fieldScale,
        );
      }
      refCount += 1;
    },

    release(): void {
      if (refCount <= 0) {
        return;
      }

      refCount -= 1;
      if (refCount === 0) {
        disposeZoneBoundaryLines(zoneBoundaryLines);
        zoneBoundaryLines = null;
      }
    },
  };
}

const zoneBoundaryOverlayManager = createSharedZoneBoundaryOverlayManager();

function renderRelativePositioningStats(
  pos: PlayerStatsSnapshot["positioning"],
): string {
  return `
    <div class="stat-row"><span class="label">Active</span><span class="value">${pos?.active_game_time?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Back</span><span class="value">${pos?.time_most_back?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Forward</span><span class="value">${pos?.time_most_forward?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Mid</span><span class="value">${pos?.time_mid_role?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Other</span><span class="value">${pos?.time_other_role?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">No teammate</span><span class="value">${pos?.time_no_teammates?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Demoed</span><span class="value">${pos?.time_demolished?.toFixed(1) ?? "?"}s</span></div>
  `;
}

function renderAbsolutePositioningStats(
  pos: PlayerStatsSnapshot["positioning"],
): string {
  const defensiveThird = getPositioningZoneTime(
    pos,
    "time_defensive_third",
    "time_defensive_zone",
  );
  const neutralThird = getPositioningZoneTime(
    pos,
    "time_neutral_third",
    "time_neutral_zone",
  );
  const offensiveThird = getPositioningZoneTime(
    pos,
    "time_offensive_third",
    "time_offensive_zone",
  );

  return `
    <div class="stat-row"><span class="label">Def third</span><span class="value">${defensiveThird?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Neutral third</span><span class="value">${neutralThird?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Off third</span><span class="value">${offensiveThird?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Def half</span><span class="value">${pos?.time_defensive_half?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Off half</span><span class="value">${pos?.time_offensive_half?.toFixed(1) ?? "?"}s</span></div>
  `;
}

function getPositioningZoneTime(
  pos: PlayerStatsSnapshot["positioning"],
  primaryKey:
    | "time_defensive_third"
    | "time_neutral_third"
    | "time_offensive_third",
  fallbackKey:
    | "time_defensive_zone"
    | "time_neutral_zone"
    | "time_offensive_zone",
): number | undefined {
  const primaryValue = pos?.[primaryKey];
  if (typeof primaryValue === "number" && Number.isFinite(primaryValue)) {
    return primaryValue;
  }

  const fallbackValue = (pos as Record<string, unknown> | undefined)?.[
    fallbackKey
  ];
  return typeof fallbackValue === "number" && Number.isFinite(fallbackValue)
    ? fallbackValue
    : undefined;
}

function formatInteger(value: number | undefined): string {
  if (value === undefined || Number.isNaN(value)) {
    return "?";
  }

  return `${Math.round(value)}`;
}

function formatNumber(
  value: number | undefined,
  digits = 1,
  suffix = "",
): string {
  if (value === undefined || Number.isNaN(value)) {
    return "?";
  }

  return `${value.toFixed(digits)}${suffix}`;
}

function formatPercentage(
  numerator: number | undefined,
  denominator: number | undefined,
  digits = 1,
): string {
  if (
    numerator === undefined ||
    denominator === undefined ||
    Number.isNaN(numerator) ||
    Number.isNaN(denominator) ||
    denominator <= 0
  ) {
    return "?";
  }

  return `${(numerator * 100 / denominator).toFixed(digits)}%`;
}

function renderCoreStats(core: PlayerStatsSnapshot["core"]): string {
  const shootingPercentage = core
    ? core.shots > 0
      ? formatPercentage(core.goals, core.shots)
      : "0.0%"
    : "?";

  return `
    <div class="stat-row"><span class="label">Score</span><span class="value">${formatInteger(core?.score)}</span></div>
    <div class="stat-row"><span class="label">Goals</span><span class="value">${formatInteger(core?.goals)}</span></div>
    <div class="stat-row"><span class="label">Assists</span><span class="value">${formatInteger(core?.assists)}</span></div>
    <div class="stat-row"><span class="label">Saves</span><span class="value">${formatInteger(core?.saves)}</span></div>
    <div class="stat-row"><span class="label">Shots</span><span class="value">${formatInteger(core?.shots)}</span></div>
    <div class="stat-row"><span class="label">Shooting</span><span class="value">${shootingPercentage}</span></div>
    <div class="stat-row"><span class="label">GA as last</span><span class="value">${formatInteger(core?.goals_conceded_while_last_defender)}</span></div>
  `;
}

function renderBallCarryStats(
  ballCarry: PlayerStatsSnapshot["ball_carry"],
): string {
  const carryCount = ballCarry?.carry_count;
  return `
    <div class="stat-row"><span class="label">Carries</span><span class="value">${formatInteger(carryCount)}</span></div>
    <div class="stat-row"><span class="label">Total time</span><span class="value">${formatNumber(ballCarry?.total_carry_time, 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Avg carry</span><span class="value">${carryCount ? formatNumber(ballCarry ? ballCarry.total_carry_time / carryCount : undefined, 1, "s") : carryCount === 0 ? "0.0s" : "?"}</span></div>
    <div class="stat-row"><span class="label">Longest</span><span class="value">${formatNumber(ballCarry?.longest_carry_time, 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Furthest</span><span class="value">${formatNumber(ballCarry?.furthest_carry_distance, 0, " uu")}</span></div>
    <div class="stat-row"><span class="label">Avg straight</span><span class="value">${carryCount ? formatNumber(ballCarry ? ballCarry.total_straight_line_distance / carryCount : undefined, 0, " uu") : carryCount === 0 ? "0 uu" : "?"}</span></div>
    <div class="stat-row"><span class="label">Avg path</span><span class="value">${carryCount ? formatNumber(ballCarry ? ballCarry.total_path_distance / carryCount : undefined, 0, " uu") : carryCount === 0 ? "0 uu" : "?"}</span></div>
    <div class="stat-row"><span class="label">Fastest</span><span class="value">${formatNumber(ballCarry?.fastest_carry_speed, 0, " uu/s")}</span></div>
    <div class="stat-row"><span class="label">Avg speed</span><span class="value">${carryCount ? formatNumber(ballCarry ? ballCarry.carry_speed_sum / carryCount : undefined, 0, " uu/s") : carryCount === 0 ? "0 uu/s" : "?"}</span></div>
    <div class="stat-row"><span class="label">Avg h gap</span><span class="value">${carryCount ? formatNumber(ballCarry ? ballCarry.average_horizontal_gap_sum / carryCount : undefined, 0, " uu") : carryCount === 0 ? "0 uu" : "?"}</span></div>
    <div class="stat-row"><span class="label">Avg v gap</span><span class="value">${carryCount ? formatNumber(ballCarry ? ballCarry.average_vertical_gap_sum / carryCount : undefined, 0, " uu") : carryCount === 0 ? "0 uu" : "?"}</span></div>
  `;
}

function renderPowerslideStats(
  powerslide: PlayerStatsSnapshot["powerslide"],
): string {
  const pressCount = powerslide?.press_count;
  const averageDuration = powerslide && pressCount && pressCount > 0
    ? powerslide.total_duration / pressCount
    : pressCount === 0
      ? 0
      : undefined;

  return `
    <div class="stat-row"><span class="label">Presses</span><span class="value">${formatInteger(pressCount)}</span></div>
    <div class="stat-row"><span class="label">Total duration</span><span class="value">${formatNumber(powerslide?.total_duration, 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Avg duration</span><span class="value">${formatNumber(averageDuration, 2, "s")}</span></div>
  `;
}

function renderDemoStats(demo: PlayerStatsSnapshot["demo"]): string {
  const differential = demo
    ? demo.demos_inflicted - demo.demos_taken
    : undefined;

  return `
    <div class="stat-row"><span class="label">Inflicted</span><span class="value">${formatInteger(demo?.demos_inflicted)}</span></div>
    <div class="stat-row"><span class="label">Taken</span><span class="value">${formatInteger(demo?.demos_taken)}</span></div>
    <div class="stat-row"><span class="label">Diff</span><span class="value">${differential === undefined ? "?" : `${differential > 0 ? "+" : ""}${differential}`}</span></div>
  `;
}

function renderDodgeResetStats(
  dodgeReset: PlayerStatsSnapshot["dodge_reset"],
): string {
  return `
    <div class="stat-row"><span class="label">Resets</span><span class="value">${formatInteger(dodgeReset?.count)}</span></div>
    <div class="stat-row"><span class="label">On-ball</span><span class="value">${formatInteger(dodgeReset?.on_ball_count)}</span></div>
  `;
}

function renderMustyFlickStats(
  mustyFlick: PlayerStatsSnapshot["musty_flick"],
): string {
  return `
    <div class="stat-row"><span class="label">Musties</span><span class="value">${formatInteger(mustyFlick?.count)}</span></div>
    <div class="stat-row"><span class="label">Aerial</span><span class="value">${formatInteger(mustyFlick?.aerial_count)}</span></div>
    <div class="stat-row"><span class="label">High conf</span><span class="value">${formatInteger(mustyFlick?.high_confidence_count)}</span></div>
    <div class="stat-row"><span class="label">Last conf</span><span class="value">${formatNumber(mustyFlick?.last_confidence, 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Avg conf</span><span class="value">${formatNumber(mustyFlick?.average_confidence, 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Best conf</span><span class="value">${formatNumber(mustyFlick?.best_confidence, 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Since last</span><span class="value">${formatNumber(mustyFlick?.time_since_last_musty, 2, "s")}</span></div>
  `;
}

function renderSpeedFlipStats(
  speedFlip: PlayerStatsSnapshot["speed_flip"],
): string {
  return `
    <div class="stat-row"><span class="label">Attempts</span><span class="value">${formatInteger(speedFlip?.count)}</span></div>
    <div class="stat-row"><span class="label">High conf</span><span class="value">${formatInteger(speedFlip?.high_confidence_count)}</span></div>
    <div class="stat-row"><span class="label">Last quality</span><span class="value">${formatNumber(speedFlip?.last_quality, 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Avg quality</span><span class="value">${formatNumber(speedFlip?.average_quality, 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Best quality</span><span class="value">${formatNumber(speedFlip?.best_quality, 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Since last</span><span class="value">${formatNumber(speedFlip?.time_since_last_speed_flip, 2, "s")}</span></div>
  `;
}

function createPlayerStatsModule<T>(options: {
  id: string;
  label: string;
  select: (player: PlayerStatsSnapshot) => T | undefined;
  render: (stats: T | undefined, player: PlayerStatsSnapshot) => string;
}): StatModule {
  return {
    id: options.id,
    label: options.label,

    setup() {},

    teardown() {},

    onBeforeRender() {},

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => renderPlayerCard(
        player.name,
        player.is_team_0,
        options.render(options.select(player), player),
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return options.render(options.select(player), player);
    },
  };
}

function createRelativePositioningModule(): StatModule {
  let thresholdZoneOverlay: ThresholdZoneOverlay | null = null;
  let fieldScale = 1;

  return {
    id: RELATIVE_POSITIONING_MODULE_ID,
    label: "Relative Positioning",

    setup(ctx) {
      fieldScale = ctx.fieldScale;
      thresholdZoneOverlay = new ThresholdZoneOverlay(
        ctx.player.sceneState.scene,
        ctx.replay,
        fieldScale,
      );
    },

    teardown() {
      thresholdZoneOverlay?.dispose();
      thresholdZoneOverlay = null;
    },

    onBeforeRender(info) {
      thresholdZoneOverlay?.update(info, fieldScale);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => {
        const pos = player.positioning;
        const role = getCurrentRole(
          ctx.replay,
          playerIdToString(player.player_id),
          frameIndex,
        );
        return renderPlayerCard(
          player.name,
          player.is_team_0,
          renderRelativePositioningStats(pos),
          `<span class="role-indicator role-${role}">${ROLE_LABELS[role]}</span>`,
        );
      }).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      const pos = player?.positioning;
      if (!player) return "";

      return renderRelativePositioningStats(pos);
    },
  };
}

function createAbsolutePositioningModule(): StatModule {
  return {
    id: "absolute-positioning",
    label: "Absolute Positioning",

    setup(ctx) {
      zoneBoundaryOverlayManager.acquire(ctx);
    },

    teardown() {
      zoneBoundaryOverlayManager.release();
    },

    onBeforeRender() {},

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => renderPlayerCard(
        player.name,
        player.is_team_0,
        renderAbsolutePositioningStats(player.positioning),
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderAbsolutePositioningStats(player.positioning);
    },
  };
}

function createBoostModule(): StatModule {
  return {
    id: "boost",
    label: "Boost",

    setup() {},

    teardown() {},

    onBeforeRender() {},

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => {
        const boost = player.boost;
        const avgBoost =
          boost && boost.tracked_time > 0
            ? toBoostDisplayUnits(boost.boost_integral / boost.tracked_time)
              .toFixed(0)
            : "?";
        return renderPlayerCard(
          player.name,
          player.is_team_0,
          `
          <div class="stat-row"><span class="label">Collected</span><span class="value">${formatCollectedWithRespawnBound(boost?.amount_collected, boost?.amount_respawned)}</span></div>
          <div class="stat-row"><span class="label">Big pads amt</span><span class="value">${formatBoostDisplayAmount(boost?.amount_collected_big)}</span></div>
          <div class="stat-row"><span class="label">Small pads amt</span><span class="value">${formatBoostDisplayAmount(boost?.amount_collected_small)}</span></div>
          <div class="stat-row"><span class="label">Respawns</span><span class="value">${formatBoostDisplayAmount(boost?.amount_respawned)}</span></div>
          <div class="stat-row"><span class="label">Overfill</span><span class="value">${formatBoostDisplayAmount(boost?.overfill_total)}</span></div>
          <div class="stat-row"><span class="label">Used</span><span class="value">${formatBoostDisplayAmount(boost?.amount_used)}</span></div>
          <div class="stat-row"><span class="label">Used ground</span><span class="value">${formatBoostDisplayAmount(boost?.amount_used_while_grounded)}</span></div>
          <div class="stat-row"><span class="label">Used air</span><span class="value">${formatBoostDisplayAmount(boost?.amount_used_while_airborne)}</span></div>
          <div class="stat-row"><span class="label">Big pads</span><span class="value">${boost?.big_pads_collected ?? "?"}</span></div>
          <div class="stat-row"><span class="label">Small pads</span><span class="value">${boost?.small_pads_collected ?? "?"}</span></div>
          <div class="stat-row"><span class="label">Stolen</span><span class="value">${formatBoostDisplayAmount(boost?.amount_stolen)}</span></div>
          <div class="stat-row"><span class="label">Avg boost</span><span class="value">${avgBoost}</span></div>
          <div class="stat-row"><span class="label">Time @ 0</span><span class="value">${boost?.time_zero_boost?.toFixed(1) ?? "?"}s</span></div>
          <div class="stat-row"><span class="label">Time @ 100</span><span class="value">${boost?.time_hundred_boost?.toFixed(1) ?? "?"}s</span></div>
        `,
        );
      }).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      const boost = player?.boost;
      if (!player) return "";

      const avgBoost =
        boost && boost.tracked_time > 0
          ? toBoostDisplayUnits(boost.boost_integral / boost.tracked_time)
            .toFixed(0)
          : "?";

      return `
        <div class="stat-row"><span class="label">Collected</span><span class="value">${formatCollectedWithRespawnBound(boost?.amount_collected, boost?.amount_respawned)}</span></div>
        <div class="stat-row"><span class="label">Big pads amt</span><span class="value">${formatBoostDisplayAmount(boost?.amount_collected_big)}</span></div>
        <div class="stat-row"><span class="label">Small pads amt</span><span class="value">${formatBoostDisplayAmount(boost?.amount_collected_small)}</span></div>
        <div class="stat-row"><span class="label">Respawns</span><span class="value">${formatBoostDisplayAmount(boost?.amount_respawned)}</span></div>
        <div class="stat-row"><span class="label">Overfill</span><span class="value">${formatBoostDisplayAmount(boost?.overfill_total)}</span></div>
        <div class="stat-row"><span class="label">Used</span><span class="value">${formatBoostDisplayAmount(boost?.amount_used)}</span></div>
        <div class="stat-row"><span class="label">Used ground</span><span class="value">${formatBoostDisplayAmount(boost?.amount_used_while_grounded)}</span></div>
        <div class="stat-row"><span class="label">Used air</span><span class="value">${formatBoostDisplayAmount(boost?.amount_used_while_airborne)}</span></div>
        <div class="stat-row"><span class="label">Big pads</span><span class="value">${boost?.big_pads_collected ?? "?"}</span></div>
        <div class="stat-row"><span class="label">Small pads</span><span class="value">${boost?.small_pads_collected ?? "?"}</span></div>
        <div class="stat-row"><span class="label">Stolen</span><span class="value">${formatBoostDisplayAmount(boost?.amount_stolen)}</span></div>
        <div class="stat-row"><span class="label">Avg boost</span><span class="value">${avgBoost}</span></div>
        <div class="stat-row"><span class="label">Time @ 0</span><span class="value">${boost?.time_zero_boost?.toFixed(1) ?? "?"}s</span></div>
        <div class="stat-row"><span class="label">Time @ 100</span><span class="value">${boost?.time_hundred_boost?.toFixed(1) ?? "?"}s</span></div>
      `;
    },
  };
}

function createCoreModule(): StatModule {
  return createPlayerStatsModule({
    id: "core",
    label: "Core",
    select: (player) => player.core,
    render: (core) => renderCoreStats(core),
  });
}

function createPossessionModule(runtime: StatModuleRuntime): StatModule {
  let settingsEl: HTMLDivElement | null = null;
  let breakdownReadoutEl: HTMLElement | null = null;
  const activeBreakdownClasses = new Set<PossessionBreakdownClass>();
  const orderedBreakdownClasses: PossessionBreakdownClass[] = [
    "possession_state",
  ];

  return {
    id: "possession",
    label: "Possession",

    setup() {
      syncPossessionSettingsUi();
    },

    teardown() {},

    onBeforeRender() {},

    getTimelineRanges(ctx) {
      return buildPossessionTimelineRanges(
        ctx.statsTimeline,
        undefined,
        ctx.replay,
      );
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame?.possession) return "";

      return [
        renderPlayerCard(
          "Blue Team",
          true,
          renderPossessionStats(statsFrame.possession, {
            isTeamZero: true,
            breakdownClasses: getActiveBreakdownClasses(),
          }),
        ),
        renderPlayerCard(
          "Orange Team",
          false,
          renderPossessionStats(statsFrame.possession, {
            isTeamZero: false,
            breakdownClasses: getActiveBreakdownClasses(),
          }),
        ),
      ].join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!statsFrame?.possession || !player) return "";

      return renderPossessionStats(statsFrame.possession, {
        isTeamZero: player.is_team_0,
        breakdownClasses: getActiveBreakdownClasses(),
      });
    },

    renderSettings() {
      if (!settingsEl) {
        settingsEl = document.createElement("div");
        settingsEl.className = "module-settings-card";

        const header = document.createElement("div");
        header.className = "module-settings-header";

        const text = document.createElement("div");
        const eyebrow = document.createElement("p");
        eyebrow.className = "module-settings-eyebrow";
        eyebrow.textContent = "Stat display";
        const title = document.createElement("h3");
        title.textContent = "Possession breakdown";
        text.append(eyebrow, title);

        breakdownReadoutEl = document.createElement("strong");
        breakdownReadoutEl.className = "metric-readout";
        header.append(text, breakdownReadoutEl);

        const options = document.createElement("div");
        options.className = "module-settings-options";

        const optionLabel = document.createElement("label");
        optionLabel.className = "toggle";

        const checkbox = document.createElement("input");
        checkbox.type = "checkbox";
        checkbox.dataset.breakdownClass = "possession_state";
        checkbox.addEventListener("change", () => {
          if (checkbox.checked) {
            activeBreakdownClasses.add("possession_state");
          } else {
            activeBreakdownClasses.delete("possession_state");
          }
          syncPossessionSettingsUi();
          runtime.rerenderCurrentState();
        });

        const optionText = document.createElement("span");
        optionText.textContent = "Control";
        optionLabel.append(checkbox, optionText);
        options.append(optionLabel);

        settingsEl.append(header, options);
      }

      syncPossessionSettingsUi();
      return settingsEl;
    },
  };

  function syncPossessionSettingsUi(): void {
    if (!settingsEl) {
      return;
    }

    for (const checkbox of settingsEl.querySelectorAll<HTMLInputElement>(
      "input[data-breakdown-class]",
    )) {
      const className = checkbox.dataset
        .breakdownClass as PossessionBreakdownClass | undefined;
      checkbox.checked = className
        ? activeBreakdownClasses.has(className)
        : false;
    }

    if (breakdownReadoutEl) {
      breakdownReadoutEl.textContent =
        activeBreakdownClasses.has("possession_state")
          ? "Control"
          : "Total only";
    }
  }

  function getActiveBreakdownClasses(): PossessionBreakdownClass[] {
    return orderedBreakdownClasses.filter((className) =>
      activeBreakdownClasses.has(className)
    );
  }
}

function createFiftyFiftyModule(): StatModule {
  let overlay: FiftyFiftyOverlay | null = null;

  return {
    id: "fifty-fifty",
    label: "50/50",

    setup(ctx) {
      overlay = new FiftyFiftyOverlay(
        ctx.player.sceneState,
        ctx.player.container,
        ctx.replay,
        ctx.statsTimeline,
      );
    },

    teardown() {
      overlay?.dispose();
      overlay = null;
    },

    onBeforeRender(info) {
      overlay?.update(info.currentTime);
    },

    getTimelineEvents(ctx) {
      return buildFiftyFiftyTimelineEvents(ctx.statsTimeline, ctx.replay);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      const summary = [
        renderPlayerCard(
          "Blue Team",
          true,
          renderFiftyFiftySummary(statsFrame.fifty_fifty, true),
        ),
        renderPlayerCard(
          "Orange Team",
          false,
          renderFiftyFiftySummary(statsFrame.fifty_fifty, false),
        ),
      ].join("");

      const players = statsFrame.players.map((player) => renderPlayerCard(
        player.name,
        player.is_team_0,
        renderPlayerFiftyFiftyStats(player.fifty_fifty),
      )).join("");

      return summary + players;
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderPlayerFiftyFiftyStats(player.fifty_fifty);
    },
  };
}

function createPressureModule(): StatModule {
  let halfFieldOverlay: HalfFieldOverlay | null = null;
  let replay: ReplayModel | null = null;

  return {
    id: "pressure",
    label: "Half Control",

    setup(ctx) {
      replay = ctx.replay;
      halfFieldOverlay = new HalfFieldOverlay(
        ctx.player.sceneState.scene,
        ctx.fieldScale,
      );
    },

    teardown() {
      halfFieldOverlay?.dispose();
      halfFieldOverlay = null;
      replay = null;
    },

    onBeforeRender(info) {
      const ballFrame = replay?.ballFrames[info.frameIndex];
      halfFieldOverlay?.update(ballFrame?.position?.y ?? null);
    },

    getTimelineRanges(ctx) {
      return buildPressureTimelineRanges(ctx.statsTimeline, undefined, ctx.replay);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame?.pressure) return "";

      return renderSharedCard(
        "Field State",
        renderPressureStats(statsFrame.pressure, {
          labelPerspective: {
            kind: "shared",
          },
        }),
      );
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!statsFrame?.pressure || !player) return "";

      return renderPressureStats(statsFrame.pressure, {
        labelPerspective: {
          kind: "team",
          isTeamZero: player.is_team_0,
        },
      });
    },
  };
}

function createTimeInZoneModule(): StatModule {
  return {
    id: "time-in-zone",
    label: "Time In Zone",

    setup(ctx) {
      zoneBoundaryOverlayManager.acquire(ctx);
    },

    teardown() {
      zoneBoundaryOverlayManager.release();
    },

    onBeforeRender() {},

    getTimelineRanges(ctx) {
      return buildTimeInZoneTimelineRanges(ctx.statsTimeline, ctx.replay);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => renderPlayerCard(
        player.name,
        player.is_team_0,
        renderAbsolutePositioningStats(player.positioning),
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderAbsolutePositioningStats(player.positioning);
    },
  };
}

function createRushModule(): StatModule {
  return {
    id: "rush",
    label: "Rush",

    setup() {},

    teardown() {},

    onBeforeRender() {},

    getTimelineEvents(ctx) {
      return buildRushTimelineEvents(ctx.statsTimeline, ctx.replay);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame?.rush) return "";

      return [
        renderPlayerCard(
          "Blue Team",
          true,
          renderRushStats(statsFrame.rush, true),
        ),
        renderPlayerCard(
          "Orange Team",
          false,
          renderRushStats(statsFrame.rush, false),
        ),
      ].join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!statsFrame?.rush || !player) return "";

      return renderRushStats(statsFrame.rush, player.is_team_0);
    },
  };
}

function createBallCarryModule(): StatModule {
  return createPlayerStatsModule({
    id: "ball-carry",
    label: "Ball Carry",
    select: (player) => player.ball_carry,
    render: (ballCarry) => renderBallCarryStats(ballCarry),
  });
}

function createDodgeResetModule(): StatModule {
  return createPlayerStatsModule({
    id: "dodge-reset",
    label: "Dodge Reset",
    select: (player) => player.dodge_reset,
    render: (dodgeReset) => renderDodgeResetStats(dodgeReset),
  });
}

function createMustyFlickModule(): StatModule {
  return {
    id: "musty-flick",
    label: "Musty Flick",

    setup() {},

    teardown() {},

    onBeforeRender() {},

    getTimelineEvents(ctx) {
      return buildMustyFlickTimelineEvents(ctx.statsTimeline, ctx.replay);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => renderPlayerCard(
        player.name,
        player.is_team_0,
        renderMustyFlickStats(player.musty_flick),
        player.musty_flick?.is_last_musty
          ? '<span class="role-indicator role-forward">Last Musty</span>'
          : "",
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderMustyFlickStats(player.musty_flick);
    },
  };
}

function createSpeedFlipModule(): StatModule {
  let overlay: SpeedFlipOverlay | null = null;

  return {
    id: "speed-flip",
    label: "Speed Flip",

    setup(ctx) {
      overlay = new SpeedFlipOverlay(
        ctx.player.sceneState,
        ctx.player.container,
        ctx.replay,
        ctx.statsTimeline,
      );
    },

    teardown() {
      overlay?.dispose();
      overlay = null;
    },

    onBeforeRender(info) {
      overlay?.update(info.currentTime);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => renderPlayerCard(
        player.name,
        player.is_team_0,
        renderSpeedFlipStats(player.speed_flip),
        player.speed_flip?.is_last_speed_flip
          ? '<span class="role-indicator role-forward">Last Speed Flip</span>'
          : "",
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderSpeedFlipStats(player.speed_flip);
    },
  };
}

function createTouchModule(runtime: StatModuleRuntime): StatModule {
  let overlay: TouchEventOverlay | null = null;
  let settingsEl: HTMLDivElement | null = null;
  let decayReadoutEl: HTMLElement | null = null;
  let breakdownReadoutEl: HTMLElement | null = null;
  const activeBreakdownClasses = new Set<TouchBreakdownClass>();
  const orderedBreakdownClasses: TouchBreakdownClass[] = [
    "kind",
    "height_band",
  ];

  return {
    id: "touch",
    label: "Touch",

    setup(ctx) {
      overlay = new TouchEventOverlay(
        ctx.player.sceneState,
        ctx.player.container,
        ctx.replay,
        ctx.statsTimeline,
      );
      syncTouchSettingsUi();
    },

    teardown() {
      overlay?.dispose();
      overlay = null;
    },

    onBeforeRender(info) {
      overlay?.update(info.currentTime);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => renderPlayerCard(
        player.name,
        player.is_team_0,
        renderTouchStats(player.touch, {
          breakdownClasses: getActiveBreakdownClasses(),
        }),
        player.touch?.is_last_touch
          ? '<span class="role-indicator role-forward">Last Touch</span>'
          : "",
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderTouchStats(player.touch, {
        breakdownClasses: getActiveBreakdownClasses(),
      });
    },

    renderSettings() {
      if (!settingsEl) {
        settingsEl = document.createElement("div");
        settingsEl.className = "module-settings-card";

        const header = document.createElement("div");
        header.className = "module-settings-header";

        const text = document.createElement("div");
        const eyebrow = document.createElement("p");
        eyebrow.className = "module-settings-eyebrow";
        eyebrow.textContent = "Touch markers";
        const title = document.createElement("h3");
        title.textContent = "Touch decay";
        text.append(eyebrow, title);

        decayReadoutEl = document.createElement("strong");
        decayReadoutEl.className = "metric-readout";
        header.append(text, decayReadoutEl);

        const label = document.createElement("label");
        const labelText = document.createElement("span");
        labelText.className = "label";
        labelText.textContent = "Keep each marker visible after the touch";

        const input = document.createElement("input");
        input.type = "range";
        input.min = "1";
        input.max = "10";
        input.step = "0.5";
        input.value = `${overlay?.getDecaySeconds() ?? 5}`;
        input.addEventListener("input", () => {
          const nextValue = Number(input.value);
          overlay?.setDecaySeconds(nextValue);
          syncTouchSettingsUi(nextValue);
        });

        label.append(labelText, input);
        const breakdownSection = document.createElement("div");
        breakdownSection.className = "module-settings-subgroup";

        const breakdownHeader = document.createElement("div");
        breakdownHeader.className = "module-settings-header";

        const breakdownText = document.createElement("div");
        const breakdownEyebrow = document.createElement("p");
        breakdownEyebrow.className = "module-settings-eyebrow";
        breakdownEyebrow.textContent = "Stat display";
        const breakdownTitle = document.createElement("h3");
        breakdownTitle.textContent = "Touch breakdown";
        breakdownText.append(breakdownEyebrow, breakdownTitle);

        breakdownReadoutEl = document.createElement("strong");
        breakdownReadoutEl.className = "metric-readout";
        breakdownHeader.append(breakdownText, breakdownReadoutEl);

        const breakdownOptions = document.createElement("div");
        breakdownOptions.className = "module-settings-options";

        for (const option of [
          { className: "kind", label: "Kind" },
          { className: "height_band", label: "Height" },
        ] satisfies Array<{ className: TouchBreakdownClass; label: string }>) {
          const optionLabel = document.createElement("label");
          optionLabel.className = "toggle";

          const checkbox = document.createElement("input");
          checkbox.type = "checkbox";
          checkbox.dataset.breakdownClass = option.className;
          checkbox.addEventListener("change", () => {
            if (checkbox.checked) {
              activeBreakdownClasses.add(option.className);
            } else {
              activeBreakdownClasses.delete(option.className);
            }
            syncTouchSettingsUi();
            runtime.rerenderCurrentState();
          });

          const optionText = document.createElement("span");
          optionText.textContent = option.label;
          optionLabel.append(checkbox, optionText);
          breakdownOptions.append(optionLabel);
        }

        breakdownSection.append(breakdownHeader, breakdownOptions);
        settingsEl.append(header, label, breakdownSection);
      }

      syncTouchSettingsUi();
      return settingsEl;
    },
  };

  function syncTouchSettingsUi(nextValue?: number): void {
    if (!settingsEl) {
      return;
    }

    const value = nextValue ?? overlay?.getDecaySeconds() ?? 5;
    const input = settingsEl.querySelector("input");
    if (input instanceof HTMLInputElement) {
      input.value = `${value}`;
    }
    if (decayReadoutEl) {
      decayReadoutEl.textContent = `${value.toFixed(1)}s`;
    }
    for (const checkbox of settingsEl.querySelectorAll<HTMLInputElement>(
      "input[data-breakdown-class]",
    )) {
      const className = checkbox.dataset
        .breakdownClass as TouchBreakdownClass | undefined;
      checkbox.checked = className
        ? activeBreakdownClasses.has(className)
        : false;
    }
    if (breakdownReadoutEl) {
      const active = getActiveBreakdownClasses();
      breakdownReadoutEl.textContent = active.length > 0
        ? active.map((className) => ({
          kind: "Kind",
          height_band: "Height",
        }[className])).join(" + ")
        : "Total only";
    }
  }

  function getActiveBreakdownClasses(): TouchBreakdownClass[] {
    return orderedBreakdownClasses.filter((className) =>
      activeBreakdownClasses.has(className)
    );
  }
}

function createMovementModule(runtime: StatModuleRuntime): StatModule {
  let settingsEl: HTMLDivElement | null = null;
  let breakdownReadoutEl: HTMLElement | null = null;
  const activeBreakdownClasses = new Set<MovementBreakdownClass>();
  const orderedBreakdownClasses: MovementBreakdownClass[] = [
    "speed_band",
    "height_band",
  ];

  return {
    id: "movement",
    label: "Movement",

    setup() {
      syncMovementSettingsUi();
    },

    teardown() {},

    onBeforeRender() {},

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      if (!statsFrame) return "";

      return statsFrame.players.map((player) => renderPlayerCard(
        player.name,
        player.is_team_0,
        renderMovementStats(player.movement, {
          breakdownClasses: getActiveBreakdownClasses(),
        }),
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderMovementStats(player.movement, {
        breakdownClasses: getActiveBreakdownClasses(),
      });
    },

    renderSettings() {
      if (!settingsEl) {
        settingsEl = document.createElement("div");
        settingsEl.className = "module-settings-card";

        const header = document.createElement("div");
        header.className = "module-settings-header";

        const text = document.createElement("div");
        const eyebrow = document.createElement("p");
        eyebrow.className = "module-settings-eyebrow";
        eyebrow.textContent = "Stat display";
        const title = document.createElement("h3");
        title.textContent = "Movement breakdown";
        text.append(eyebrow, title);

        breakdownReadoutEl = document.createElement("strong");
        breakdownReadoutEl.className = "metric-readout";
        header.append(text, breakdownReadoutEl);

        const options = document.createElement("div");
        options.className = "module-settings-options";

        for (const option of [
          { className: "speed_band", label: "Speed band" },
          { className: "height_band", label: "Height band" },
        ] satisfies Array<{
          className: MovementBreakdownClass;
          label: string;
        }>) {
          const optionLabel = document.createElement("label");
          optionLabel.className = "toggle";

          const checkbox = document.createElement("input");
          checkbox.type = "checkbox";
          checkbox.dataset.breakdownClass = option.className;
          checkbox.addEventListener("change", () => {
            if (checkbox.checked) {
              activeBreakdownClasses.add(option.className);
            } else {
              activeBreakdownClasses.delete(option.className);
            }
            syncMovementSettingsUi();
            runtime.rerenderCurrentState();
          });

          const optionText = document.createElement("span");
          optionText.textContent = option.label;
          optionLabel.append(checkbox, optionText);
          options.append(optionLabel);
        }

        settingsEl.append(header, options);
      }

      syncMovementSettingsUi();
      return settingsEl;
    },
  };

  function syncMovementSettingsUi(): void {
    if (!settingsEl) {
      return;
    }

    for (const checkbox of settingsEl.querySelectorAll<HTMLInputElement>(
      "input[data-breakdown-class]",
    )) {
      const className = checkbox.dataset
        .breakdownClass as MovementBreakdownClass | undefined;
      checkbox.checked = className
        ? activeBreakdownClasses.has(className)
        : false;
    }

    if (breakdownReadoutEl) {
      const active = getActiveBreakdownClasses();
      breakdownReadoutEl.textContent = active.length > 0
        ? active.map((className) => ({
          speed_band: "Speed band",
          height_band: "Height band",
        }[className])).join(" + ")
        : "Total only";
    }
  }

  function getActiveBreakdownClasses(): MovementBreakdownClass[] {
    return orderedBreakdownClasses.filter((className) =>
      activeBreakdownClasses.has(className)
    );
  }
}

function createPowerslideModule(): StatModule {
  return createPlayerStatsModule({
    id: "powerslide",
    label: "Powerslide",
    select: (player) => player.powerslide,
    render: (powerslide) => renderPowerslideStats(powerslide),
  });
}

function createDemoModule(): StatModule {
  return createPlayerStatsModule({
    id: "demo",
    label: "Demo",
    select: (player) => player.demo,
    render: (demo) => renderDemoStats(demo),
  });
}

export function createStatModules(runtime: StatModuleRuntime): StatModule[] {
  return [
    createCoreModule(),
    createPossessionModule(runtime),
    createFiftyFiftyModule(),
    createPressureModule(),
    createRushModule(),
    createRelativePositioningModule(),
    createAbsolutePositioningModule(),
    createTimeInZoneModule(),
    createSpeedFlipModule(),
    createTouchModule(runtime),
    createMustyFlickModule(),
    createDodgeResetModule(),
    createBoostModule(),
    createBallCarryModule(),
    createMovementModule(runtime),
    createPowerslideModule(),
    createDemoModule(),
  ];
}
