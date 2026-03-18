import "./styles.css";
import {
  createBallchasingOverlayPlugin,
  createBoostPadOverlayPlugin,
  createTimelineOverlayPlugin,
  ReplayPlayer,
  loadReplayFromBytes,
} from "../../player/src/lib.ts";
import type {
  FrameRenderInfo,
  ReplayModel,
  ReplayPlayerState,
  ReplayPlayerTrack,
} from "../../player/src/lib.ts";
import {
  HalfFieldOverlay,
  ThresholdZoneOverlay,
  createZoneBoundaryLines,
} from "./overlays.ts";
import {
  TouchEventOverlay,
  playerIdToString,
} from "./touchOverlay.ts";
import { FiftyFiftyOverlay } from "./fiftyFiftyOverlay.ts";
import {
  createDynamicStatsFrameLookup,
  createStatsFrameLookup,
  getDynamicStatsFrameForReplayFrame,
  getStatsFrameForReplayFrame,
} from "./statsTimeline.ts";
import type { Object3D } from "three";
import {
  formatCollectedWithRespawnBound,
  formatBoostDisplayAmount,
  toBoostDisplayUnits,
} from "./boostFormatting.ts";
import {
  renderFiftyFiftySummary,
  renderPlayerFiftyFiftyStats,
} from "./fiftyFiftyFormatting.ts";
import {
  renderMovementStats,
} from "./movementFormatting.ts";
import type { MovementBreakdownClass } from "./movementFormatting.ts";
import { renderPossessionStats } from "./possessionFormatting.ts";
import type { PossessionBreakdownClass } from "./possessionFormatting.ts";
import { renderPressureStats } from "./pressureFormatting.ts";
import type { PressureBreakdownClass } from "./pressureFormatting.ts";
import { renderRushStats } from "./rushFormatting.ts";
import { renderTouchStats } from "./touchFormatting.ts";
import type {
  DynamicPlayerStatsSnapshot,
  DynamicStatsFrame,
  DynamicStatsTimeline,
  PlayerStatsSnapshot,
  StatsFrame,
  StatsTimeline,
} from "./statsTimeline.ts";
import type { TouchBreakdownClass } from "./touchFormatting.ts";

import * as subtrActor from "subtr-actor";

interface StatModuleContext {
  player: ReplayPlayer;
  replay: ReplayModel;
  statsTimeline: StatsTimeline;
  statsFrameLookup: Map<number, StatsFrame>;
  dynamicStatsTimeline: DynamicStatsTimeline;
  dynamicStatsFrameLookup: Map<number, DynamicStatsFrame>;
  fieldScale: number;
}

interface StatModule {
  readonly id: string;
  readonly label: string;
  setup(ctx: StatModuleContext): void;
  teardown(): void;
  onBeforeRender(info: FrameRenderInfo): void;
  renderStats(frameIndex: number, ctx: StatModuleContext): string;
  renderSettings?(ctx: StatModuleContext | null): HTMLElement | null;
  renderFocusedPlayerStats(
    playerId: string,
    frameIndex: number,
    ctx: StatModuleContext,
  ): string;
}

const MOST_BACK_FORWARD_THRESHOLD_Y = 236.0;
const DEFAULT_CAMERA_DISTANCE_SCALE = 2.25;

type Role = "back" | "forward" | "other" | "mid";

const ROLE_LABELS: Record<Role, string> = {
  back: "Back",
  forward: "Fwd",
  other: "Other",
  mid: "Mid",
};

function getTeamClass(isTeamZero: boolean): string {
  return isTeamZero ? "team-blue" : "team-orange";
}

function renderPlayerCard(
  name: string,
  isTeamZero: boolean,
  bodyHtml: string,
  metaHtml = "",
): string {
  return `<div class="player-card ${getTeamClass(isTeamZero)}">
    <div class="player-card-header">
      <span class="player-name">${name}</span>
      ${metaHtml}
    </div>
    ${bodyHtml}
  </div>`;
}

function getStatsPlayerSnapshot(
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

function getDynamicStatsPlayerSnapshot(
  ctx: StatModuleContext,
  frameIndex: number,
  playerId: string,
): DynamicPlayerStatsSnapshot | null {
  const statsFrame = getDynamicStatsFrameForReplayFrame(
    ctx.dynamicStatsFrameLookup,
    frameIndex,
  );
  if (!statsFrame) return null;

  return statsFrame.players.find(
    (player) => playerIdToString(player.player_id) === playerId,
  ) ?? null;
}

function getCurrentRole(
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

function renderRelativePositioningStats(pos: PlayerStatsSnapshot["positioning"]): string {
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

function renderAbsolutePositioningStats(pos: PlayerStatsSnapshot["positioning"]): string {
  return `
    <div class="stat-row"><span class="label">Def third</span><span class="value">${pos?.time_defensive_third?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Neutral third</span><span class="value">${pos?.time_neutral_third?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Off third</span><span class="value">${pos?.time_offensive_third?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Def half</span><span class="value">${pos?.time_defensive_half?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Off half</span><span class="value">${pos?.time_offensive_half?.toFixed(1) ?? "?"}s</span></div>
  `;
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

function formatTimeShare(
  value: number | undefined,
  total: number | undefined,
  digits = 1,
): string {
  if (value === undefined || Number.isNaN(value)) {
    return "?";
  }

  const percentage = formatPercentage(value, total, digits);
  if (percentage === "?") {
    return `${value.toFixed(digits)}s`;
  }

  return `${value.toFixed(digits)}s (${percentage})`;
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

function renderBallCarryStats(ballCarry: PlayerStatsSnapshot["ball_carry"]): string {
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

function renderPowerslideStats(powerslide: PlayerStatsSnapshot["powerslide"]): string {
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

function renderDodgeResetStats(dodgeReset: PlayerStatsSnapshot["dodge_reset"]): string {
  return `
    <div class="stat-row"><span class="label">Resets</span><span class="value">${formatInteger(dodgeReset?.count)}</span></div>
    <div class="stat-row"><span class="label">On-ball</span><span class="value">${formatInteger(dodgeReset?.on_ball_count)}</span></div>
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
    id: "relative-positioning",
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
  let zoneBoundaryLines: ReturnType<typeof createZoneBoundaryLines> | null = null;

  return {
    id: "absolute-positioning",
    label: "Absolute Positioning",

    setup(ctx) {
      zoneBoundaryLines = createZoneBoundaryLines(
        ctx.player.sceneState.scene,
        ctx.fieldScale,
      );
    },

    teardown() {
      if (zoneBoundaryLines) {
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
      zoneBoundaryLines = null;
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
            ? toBoostDisplayUnits(boost.boost_integral / boost.tracked_time).toFixed(0)
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
          <div class="stat-row"><span class="label">Avg boost</span><span class="value">${avgBoost}%</span></div>
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
          ? toBoostDisplayUnits(boost.boost_integral / boost.tracked_time).toFixed(0)
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
        <div class="stat-row"><span class="label">Avg boost</span><span class="value">${avgBoost}%</span></div>
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

function createPossessionModule(): StatModule {
  let settingsEl: HTMLDivElement | null = null;
  let breakdownReadoutEl: HTMLElement | null = null;
  const activeBreakdownClasses = new Set<PossessionBreakdownClass>();
  const orderedBreakdownClasses: PossessionBreakdownClass[] = ["possession_state"];

  return {
    id: "possession",
    label: "Possession",

    setup() {
      syncPossessionSettingsUi();
    },

    teardown() {},

    onBeforeRender() {},

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const dynamicStatsFrame = getDynamicStatsFrameForReplayFrame(
        ctx.dynamicStatsFrameLookup,
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
            exportedStats: dynamicStatsFrame?.possession,
          }),
        ),
        renderPlayerCard(
          "Orange Team",
          false,
          renderPossessionStats(statsFrame.possession, {
            isTeamZero: false,
            breakdownClasses: getActiveBreakdownClasses(),
            exportedStats: dynamicStatsFrame?.possession,
          }),
        ),
      ].join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const dynamicStatsFrame = getDynamicStatsFrameForReplayFrame(
        ctx.dynamicStatsFrameLookup,
        frameIndex,
      );
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!statsFrame?.possession || !player) return "";

      return renderPossessionStats(statsFrame.possession, {
        isTeamZero: player.is_team_0,
        breakdownClasses: getActiveBreakdownClasses(),
        exportedStats: dynamicStatsFrame?.possession,
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
          rerenderPossessionStats();
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
      const className = checkbox.dataset.breakdownClass as PossessionBreakdownClass | undefined;
      checkbox.checked = className ? activeBreakdownClasses.has(className) : false;
    }

    if (breakdownReadoutEl) {
      breakdownReadoutEl.textContent = activeBreakdownClasses.has("possession_state")
        ? "Control"
        : "Total only";
    }
  }

  function getActiveBreakdownClasses(): PossessionBreakdownClass[] {
    return orderedBreakdownClasses.filter((className) =>
      activeBreakdownClasses.has(className)
    );
  }

  function rerenderPossessionStats(): void {
    if (!replayPlayer) {
      return;
    }

    const state = replayPlayer.getState();
    renderStats(state.frameIndex);
    renderFocusedPlayerOverlay(state);
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
  let settingsEl: HTMLDivElement | null = null;
  let breakdownReadoutEl: HTMLElement | null = null;
  const activeBreakdownClasses = new Set<PressureBreakdownClass>();
  const orderedBreakdownClasses: PressureBreakdownClass[] = ["field_half"];

  return {
    id: "pressure",
    label: "Ball Side",

    setup(ctx) {
      halfFieldOverlay = new HalfFieldOverlay(
        ctx.player.sceneState.scene,
        ctx.fieldScale,
      );
      syncPressureSettingsUi();
    },

    teardown() {
      halfFieldOverlay?.dispose();
      halfFieldOverlay = null;
    },

    onBeforeRender(info) {
      const ballFrame = replayPlayer?.replay.ballFrames[info.frameIndex];
      halfFieldOverlay?.update(ballFrame?.position?.y ?? null);
    },

    renderStats(frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const dynamicStatsFrame = getDynamicStatsFrameForReplayFrame(
        ctx.dynamicStatsFrameLookup,
        frameIndex,
      );
      if (!statsFrame?.pressure) return "";

      return [
        renderPlayerCard(
          "Blue Half",
          true,
          renderPressureStats(statsFrame.pressure, {
            isTeamZero: true,
            breakdownClasses: getActiveBreakdownClasses(),
            exportedStats: dynamicStatsFrame?.pressure,
          }),
        ),
        renderPlayerCard(
          "Orange Half",
          false,
          renderPressureStats(statsFrame.pressure, {
            isTeamZero: false,
            breakdownClasses: getActiveBreakdownClasses(),
            exportedStats: dynamicStatsFrame?.pressure,
          }),
        ),
      ].join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const statsFrame = getStatsFrameForReplayFrame(
        ctx.statsFrameLookup,
        frameIndex,
      );
      const dynamicStatsFrame = getDynamicStatsFrameForReplayFrame(
        ctx.dynamicStatsFrameLookup,
        frameIndex,
      );
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!statsFrame?.pressure || !player) return "";

      return renderPressureStats(statsFrame.pressure, {
        isTeamZero: player.is_team_0,
        breakdownClasses: getActiveBreakdownClasses(),
        exportedStats: dynamicStatsFrame?.pressure,
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
        title.textContent = "Ball side breakdown";
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
        checkbox.dataset.breakdownClass = "field_half";
        checkbox.addEventListener("change", () => {
          if (checkbox.checked) {
            activeBreakdownClasses.add("field_half");
          } else {
            activeBreakdownClasses.delete("field_half");
          }
          syncPressureSettingsUi();
          rerenderPressureStats();
        });

        const optionText = document.createElement("span");
        optionText.textContent = "Field half";
        optionLabel.append(checkbox, optionText);
        options.append(optionLabel);

        settingsEl.append(header, options);
      }

      syncPressureSettingsUi();
      return settingsEl;
    },
  };

  function syncPressureSettingsUi(): void {
    if (!settingsEl) {
      return;
    }

    for (const checkbox of settingsEl.querySelectorAll<HTMLInputElement>(
      "input[data-breakdown-class]",
    )) {
      const className = checkbox.dataset.breakdownClass as PressureBreakdownClass | undefined;
      checkbox.checked = className ? activeBreakdownClasses.has(className) : false;
    }

    if (breakdownReadoutEl) {
      breakdownReadoutEl.textContent = activeBreakdownClasses.has("field_half")
        ? "Field half"
        : "Total only";
    }
  }

  function getActiveBreakdownClasses(): PressureBreakdownClass[] {
    return orderedBreakdownClasses.filter((className) =>
      activeBreakdownClasses.has(className)
    );
  }

  function rerenderPressureStats(): void {
    if (!replayPlayer) {
      return;
    }

    const state = replayPlayer.getState();
    renderStats(state.frameIndex);
    renderFocusedPlayerOverlay(state);
  }
}

function createRushModule(): StatModule {
  return {
    id: "rush",
    label: "Rush",

    setup() {},

    teardown() {},

    onBeforeRender() {},

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

function createTouchModule(): StatModule {
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
          exportedStats: getDynamicStatsPlayerSnapshot(
            ctx,
            frameIndex,
            playerIdToString(player.player_id),
          )?.stats,
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
        exportedStats: getDynamicStatsPlayerSnapshot(ctx, frameIndex, playerId)?.stats,
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
            rerenderTouchStats();
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
      const className = checkbox.dataset.breakdownClass as TouchBreakdownClass | undefined;
      checkbox.checked = className ? activeBreakdownClasses.has(className) : false;
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

  function rerenderTouchStats(): void {
    if (!replayPlayer) {
      return;
    }

    const state = replayPlayer.getState();
    renderStats(state.frameIndex);
    renderFocusedPlayerOverlay(state);
  }
}

function createMovementModule(): StatModule {
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
          exportedStats: getDynamicStatsPlayerSnapshot(
            ctx,
            frameIndex,
            playerIdToString(player.player_id),
          )?.stats,
        }),
      )).join("");
    },

    renderFocusedPlayerStats(playerId, frameIndex, ctx) {
      const player = getStatsPlayerSnapshot(ctx, frameIndex, playerId);
      if (!player) return "";

      return renderMovementStats(player.movement, {
        breakdownClasses: getActiveBreakdownClasses(),
        exportedStats: getDynamicStatsPlayerSnapshot(ctx, frameIndex, playerId)?.stats,
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
        ] satisfies Array<{ className: MovementBreakdownClass; label: string }>) {
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
            rerenderMovementStats();
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
      const className = checkbox.dataset.breakdownClass as MovementBreakdownClass | undefined;
      checkbox.checked = className ? activeBreakdownClasses.has(className) : false;
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

  function rerenderMovementStats(): void {
    if (!replayPlayer) {
      return;
    }

    const state = replayPlayer.getState();
    renderStats(state.frameIndex);
    renderFocusedPlayerOverlay(state);
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

const RELATIVE_POSITIONING_MODULE_ID = "relative-positioning";

const MODULE_FACTORIES = [
  createCoreModule,
  createPossessionModule,
  createFiftyFiftyModule,
  createPressureModule,
  createRushModule,
  createRelativePositioningModule,
  createAbsolutePositioningModule,
  createTouchModule,
  createDodgeResetModule,
  createBoostModule,
  createBallCarryModule,
  createMovementModule,
  createPowerslideModule,
  createDemoModule,
];

const MODULES = MODULE_FACTORIES.map((factory) => factory());
let activeModules: StatModule[] = [];
let activeModuleIds = new Set<string>([RELATIVE_POSITIONING_MODULE_ID]);
let removeRenderHook: (() => void) | null = null;

let replayPlayer: ReplayPlayer | null = null;
let statsTimeline: StatsTimeline | null = null;
let statsFrameLookup: Map<number, StatsFrame> | null = null;
let dynamicStatsTimeline: DynamicStatsTimeline | null = null;
let dynamicStatsFrameLookup: Map<number, DynamicStatsFrame> | null = null;
let unsubscribe: (() => void) | null = null;

function getModuleContext(): StatModuleContext | null {
  if (
    !replayPlayer ||
    !statsTimeline ||
    !statsFrameLookup ||
    !dynamicStatsTimeline ||
    !dynamicStatsFrameLookup
  ) {
    return null;
  }

  return {
    player: replayPlayer,
    replay: replayPlayer.replay,
    statsTimeline,
    statsFrameLookup,
    dynamicStatsTimeline,
    dynamicStatsFrameLookup,
    fieldScale: replayPlayer.options.fieldScale ?? 1,
  };
}

function setupActiveModules(): void {
  teardownActiveModules();

  const ctx = getModuleContext();
  if (!ctx) return;

  activeModules = MODULES.filter((mod) => activeModuleIds.has(mod.id));
  for (const mod of activeModules) {
    mod.setup(ctx);
  }

  removeRenderHook = ctx.player.onBeforeRender((info) => {
    for (const mod of activeModules) {
      mod.onBeforeRender(info);
    }
  });
}

function teardownActiveModules(): void {
  removeRenderHook?.();
  removeRenderHook = null;

  for (const mod of activeModules) {
    mod.teardown();
  }
  activeModules = [];
}

function toggleModule(id: string, enabled: boolean): void {
  if (enabled) {
    activeModuleIds.add(id);
  } else {
    activeModuleIds.delete(id);
  }

  setupActiveModules();
  renderModuleSummary();
  renderModuleSettings();
  if (replayPlayer) {
    const state = replayPlayer.getState();
    renderStats(state.frameIndex);
    renderFocusedPlayerOverlay(state);
  }
}

function mustElement<T extends HTMLElement>(selector: string): T {
  const element = document.querySelector(selector);
  if (!(element instanceof HTMLElement)) {
    throw new Error(`Missing element for selector: ${selector}`);
  }

  return element as T;
}

const app = mustElement<HTMLDivElement>("#app");

app.innerHTML = `
  <main class="shell">
    <section class="hero">
      <div>
        <p class="eyebrow">subtr-actor / stats replay viewer</p>
        <h1>Stat Evaluation Player</h1>
        <p class="lede">
          Compare stat modules against the in-replay camera view, switch to any
          player's camera profile, and scrub with the shared timeline plugin.
        </p>
      </div>
      <label class="file-picker">
        <span>Choose replay</span>
        <input id="replay-file" type="file" accept=".replay" />
      </label>
    </section>

    <section class="workspace">
      <div class="viewport-column">
        <div class="viewport-panel">
          <div id="viewport" class="viewport"></div>
          <div
            id="followed-player-overlay"
            class="followed-player-overlay"
            hidden
          ></div>
          <div id="empty-state" class="empty-state">
            Choose a replay to start the viewer.
          </div>
        </div>

        <section class="stats-panel">
          <div class="panel-heading">
            <div>
              <p class="panel-eyebrow">Module output</p>
              <h2>Per-player stats</h2>
            </div>
          </div>
          <div id="player-stats" class="player-stats-stack">
            Load a replay to see stats.
          </div>
        </section>
      </div>

      <aside class="sidebar">
        <section class="panel">
          <p class="panel-eyebrow">Camera</p>
          <h2>Replay camera</h2>
          <label>
            <span class="label">Camera profile</span>
            <select id="attached-player" disabled>
              <option value="">Free camera</option>
            </select>
          </label>
          <label>
            <span class="label">Follow distance</span>
            <input
              id="camera-distance"
              type="range"
              min="0.75"
              max="4"
              step="0.05"
              value="${DEFAULT_CAMERA_DISTANCE_SCALE}"
              disabled
            />
          </label>
          <strong id="camera-distance-readout" class="metric-readout">
            ${DEFAULT_CAMERA_DISTANCE_SCALE.toFixed(2)}x
          </strong>
          <label class="toggle">
            <input id="ball-cam" type="checkbox" disabled />
            <span>Ball cam</span>
          </label>
          <dl class="detail-grid">
            <div>
              <dt>Profile</dt>
              <dd id="camera-profile-readout">Free camera</dd>
            </div>
            <div>
              <dt>FOV</dt>
              <dd id="camera-fov-readout">--</dd>
            </div>
            <div>
              <dt>Height</dt>
              <dd id="camera-height-readout">--</dd>
            </div>
            <div>
              <dt>Pitch</dt>
              <dd id="camera-pitch-readout">--</dd>
            </div>
            <div>
              <dt>Distance</dt>
              <dd id="camera-base-distance-readout">--</dd>
            </div>
            <div>
              <dt>Stiffness</dt>
              <dd id="camera-stiffness-readout">--</dd>
            </div>
          </dl>
        </section>

        <section class="panel">
          <p class="panel-eyebrow">Modules</p>
          <h2>Overlay modules</h2>
          <p class="panel-copy">
            Toggle stat overlays independently while keeping the timeline and
            replay camera controls active.
          </p>
          <label class="toggle">
            <input id="show-followed-player-overlay" type="checkbox" />
            <span>Show followed player in viewport</span>
          </label>
          <div class="module-list" id="module-summary"></div>
          <div id="module-settings" class="module-settings" hidden></div>
        </section>

        <section class="panel">
          <p class="panel-eyebrow">Transport</p>
          <h2>Playback</h2>
          <div class="transport-row">
            <button id="toggle-playback" disabled>Play</button>
            <select id="playback-rate" disabled>
              <option value="0.25">0.25x</option>
              <option value="0.5">0.5x</option>
              <option value="1" selected>1.0x</option>
              <option value="1.5">1.5x</option>
              <option value="2">2.0x</option>
            </select>
          </div>
          <label class="toggle">
            <input id="skip-post-goal-transitions" type="checkbox" checked />
            <span>Skip post-goal resets</span>
          </label>
          <label class="toggle">
            <input id="skip-kickoffs" type="checkbox" />
            <span>Skip kickoffs</span>
          </label>
          <div class="detail-grid">
            <div>
              <dt>Time</dt>
              <dd id="time-readout">0.00s</dd>
            </div>
            <div>
              <dt>Frame</dt>
              <dd id="frame-readout">0</dd>
            </div>
            <div>
              <dt>Duration</dt>
              <dd id="duration-readout">0.00s</dd>
            </div>
            <div>
              <dt>Status</dt>
              <dd id="playback-status-readout">Stopped</dd>
            </div>
          </div>
        </section>

        <section class="panel">
          <p class="panel-eyebrow">Replay</p>
          <h2>Loaded file</h2>
          <dl class="detail-grid">
            <div>
              <dt>Status</dt>
              <dd id="status-readout">Waiting for file</dd>
            </div>
            <div>
              <dt>Players</dt>
              <dd id="players-readout">--</dd>
            </div>
            <div>
              <dt>Frames</dt>
              <dd id="frames-readout">--</dd>
            </div>
            <div>
              <dt>Timeline events</dt>
              <dd id="events-readout">--</dd>
            </div>
          </dl>
        </section>
      </aside>
    </section>
  </main>
`;

const fileInput = mustElement<HTMLInputElement>("#replay-file");
const viewport = mustElement<HTMLDivElement>("#viewport");
const emptyState = mustElement<HTMLDivElement>("#empty-state");
const togglePlayback = mustElement<HTMLButtonElement>("#toggle-playback");
const followedPlayerOverlay = mustElement<HTMLDivElement>(
  "#followed-player-overlay",
);
const playbackRate = mustElement<HTMLSelectElement>("#playback-rate");
const attachedPlayer = mustElement<HTMLSelectElement>("#attached-player");
const cameraDistance = mustElement<HTMLInputElement>("#camera-distance");
const cameraDistanceReadout = mustElement<HTMLElement>("#camera-distance-readout");
const ballCam = mustElement<HTMLInputElement>("#ball-cam");
const showFollowedPlayerOverlay = mustElement<HTMLInputElement>(
  "#show-followed-player-overlay",
);
const moduleSummaryEl = mustElement<HTMLDivElement>("#module-summary");
const moduleSettingsEl = mustElement<HTMLDivElement>("#module-settings");
const timeReadout = mustElement<HTMLElement>("#time-readout");
const frameReadout = mustElement<HTMLElement>("#frame-readout");
const durationReadout = mustElement<HTMLElement>("#duration-readout");
const playbackStatusReadout = mustElement<HTMLElement>("#playback-status-readout");
const statusReadout = mustElement<HTMLElement>("#status-readout");
const playersReadout = mustElement<HTMLElement>("#players-readout");
const framesReadout = mustElement<HTMLElement>("#frames-readout");
const eventsReadout = mustElement<HTMLElement>("#events-readout");
const playerStatsEl = mustElement<HTMLDivElement>("#player-stats");
const cameraProfileReadout = mustElement<HTMLElement>("#camera-profile-readout");
const cameraFovReadout = mustElement<HTMLElement>("#camera-fov-readout");
const cameraHeightReadout = mustElement<HTMLElement>("#camera-height-readout");
const cameraPitchReadout = mustElement<HTMLElement>("#camera-pitch-readout");
const cameraBaseDistanceReadout = mustElement<HTMLElement>("#camera-base-distance-readout");
const cameraStiffnessReadout = mustElement<HTMLElement>("#camera-stiffness-readout");
const skipPostGoalTransitions = mustElement<HTMLInputElement>(
  "#skip-post-goal-transitions",
);
const skipKickoffs = mustElement<HTMLInputElement>("#skip-kickoffs");

function renderModuleSummary(): void {
  moduleSummaryEl.replaceChildren();

  for (const mod of MODULES) {
    const active = activeModuleIds.has(mod.id);
    const item = document.createElement("button");
    item.type = "button";
    item.className = "module-summary-item";
    item.dataset.active = active ? "true" : "false";
    item.setAttribute("aria-pressed", active ? "true" : "false");
    item.addEventListener("click", () => {
      toggleModule(mod.id, !activeModuleIds.has(mod.id));
    });

    const name = document.createElement("span");
    name.textContent = mod.label;

    const state = document.createElement("strong");
    state.textContent = active ? "On" : "Off";

    item.append(name, state);
    moduleSummaryEl.append(item);
  }
}

function renderModuleSettings(): void {
  moduleSettingsEl.replaceChildren();

  const ctx = getModuleContext();
  const panels = activeModules
    .map((mod) => mod.renderSettings?.(ctx) ?? null)
    .filter((panel): panel is HTMLElement => panel instanceof HTMLElement);

  if (panels.length === 0) {
    moduleSettingsEl.hidden = true;
    return;
  }

  moduleSettingsEl.hidden = false;
  moduleSettingsEl.append(...panels);
}

function formatSetting(
  value: number | undefined,
  suffix = "",
  digits = 0,
): string {
  if (value === undefined || Number.isNaN(value)) {
    return "--";
  }

  return `${value.toFixed(digits)}${suffix}`;
}

function setTransportEnabled(enabled: boolean): void {
  togglePlayback.disabled = !enabled;
  playbackRate.disabled = !enabled;
  attachedPlayer.disabled = !enabled;
}

function syncCameraControlAvailability(state?: ReplayPlayerState): void {
  const attached = state?.attachedPlayerId ?? null;
  const hasAttachedCamera = replayPlayer !== null && attached !== null;
  cameraDistance.disabled = !hasAttachedCamera;
  ballCam.disabled = !hasAttachedCamera;
}

function populateAttachedPlayerOptions(players: ReplayPlayerTrack[]): void {
  attachedPlayer.replaceChildren();
  attachedPlayer.append(new Option("Free camera", ""));

  for (const player of players) {
    attachedPlayer.append(
      new Option(
        `${player.name} (${player.isTeamZero ? "Blue" : "Orange"})`,
        player.id,
      ),
    );
  }
}

function renderCameraProfile(attachedPlayerId: string | null): void {
  if (!replayPlayer || attachedPlayerId === null) {
    cameraProfileReadout.textContent = "Free camera";
    cameraFovReadout.textContent = "--";
    cameraHeightReadout.textContent = "--";
    cameraPitchReadout.textContent = "--";
    cameraBaseDistanceReadout.textContent = "--";
    cameraStiffnessReadout.textContent = "--";
    return;
  }

  const player = replayPlayer.replay.players.find(
    (candidate) => candidate.id === attachedPlayerId,
  );
  if (!player) {
    cameraProfileReadout.textContent = "Unknown";
    cameraFovReadout.textContent = "--";
    cameraHeightReadout.textContent = "--";
    cameraPitchReadout.textContent = "--";
    cameraBaseDistanceReadout.textContent = "--";
    cameraStiffnessReadout.textContent = "--";
    return;
  }

  const { cameraSettings } = player;
  cameraProfileReadout.textContent = player.name;
  cameraFovReadout.textContent = formatSetting(cameraSettings.fov, "", 0);
  cameraHeightReadout.textContent = formatSetting(cameraSettings.height, "", 0);
  cameraPitchReadout.textContent = formatSetting(cameraSettings.pitch, "", 0);
  cameraBaseDistanceReadout.textContent = formatSetting(
    cameraSettings.distance,
    "",
    0,
  );
  cameraStiffnessReadout.textContent = formatSetting(
    cameraSettings.stiffness,
    "",
    2,
  );
}

function renderStats(frameIndex: number): void {
  const ctx = getModuleContext();
  if (!ctx) return;

  const sections = activeModules
    .map((mod) => {
      const html = mod.renderStats(frameIndex, ctx);
      if (!html) return "";
      return `<section class="stat-module-section">
        <div class="stat-module-label">${mod.label}</div>
        <div class="player-stats-grid">${html}</div>
      </section>`;
    })
    .filter(Boolean);

  playerStatsEl.innerHTML = sections.length > 0
    ? sections.join("")
    : "No stat modules active.";
}

function renderFocusedPlayerOverlay(state?: ReplayPlayerState): void {
  const ctx = getModuleContext();
  if (!ctx || !state || !showFollowedPlayerOverlay.checked) {
    followedPlayerOverlay.hidden = true;
    followedPlayerOverlay.innerHTML = "";
    return;
  }

  const attachedPlayerId = state.attachedPlayerId;
  if (!attachedPlayerId) {
    followedPlayerOverlay.hidden = true;
    followedPlayerOverlay.innerHTML = "";
    return;
  }

  const player = getStatsPlayerSnapshot(ctx, state.frameIndex, attachedPlayerId);
  if (!player) {
    followedPlayerOverlay.hidden = true;
    followedPlayerOverlay.innerHTML = "";
    return;
  }

  const sections = activeModules.map((mod) => {
    const body = mod.renderFocusedPlayerStats(attachedPlayerId, state.frameIndex, ctx);
    if (!body) return "";

    return `<section class="focused-player-module">
      <div class="focused-player-module-label">${mod.label}</div>
      <div class="focused-player-module-body">${body}</div>
    </section>`;
  }).filter(Boolean);

  if (sections.length === 0) {
    followedPlayerOverlay.hidden = true;
    followedPlayerOverlay.innerHTML = "";
    return;
  }

  const showRoleIndicator = activeModuleIds.has(RELATIVE_POSITIONING_MODULE_ID);
  const role = showRoleIndicator
    ? getCurrentRole(ctx.replay, attachedPlayerId, state.frameIndex)
    : null;
  followedPlayerOverlay.innerHTML = `
    <div class="followed-player-overlay-card ${getTeamClass(player.is_team_0)}">
      <div class="followed-player-overlay-header">
        <div class="followed-player-overlay-title">
          <p class="followed-player-overlay-eyebrow">Follow cam</p>
          <div class="followed-player-overlay-name-row">
            <span class="player-name">${player.name}</span>
            ${role ? `<span class="role-indicator role-${role}">${ROLE_LABELS[role]}</span>` : ""}
          </div>
        </div>
        <strong class="followed-player-overlay-team">
          ${player.is_team_0 ? "Blue" : "Orange"}
        </strong>
      </div>
      <div class="followed-player-overlay-body">${sections.join("")}</div>
    </div>
  `;
  followedPlayerOverlay.hidden = false;
}

function renderSnapshot(state: ReplayPlayerState): void {
  timeReadout.textContent = `${state.currentTime.toFixed(2)}s`;
  frameReadout.textContent = `${state.frameIndex}`;
  durationReadout.textContent = `${state.duration.toFixed(2)}s`;
  playbackStatusReadout.textContent = state.playing ? "Playing" : "Paused";
  togglePlayback.textContent = state.playing ? "Pause" : "Play";
  playbackRate.value = `${state.speed}`;
  cameraDistance.value = `${state.cameraDistanceScale}`;
  cameraDistanceReadout.textContent = `${state.cameraDistanceScale.toFixed(2)}x`;
  ballCam.checked = state.ballCamEnabled;
  attachedPlayer.value = state.attachedPlayerId ?? "";
  skipPostGoalTransitions.checked = state.skipPostGoalTransitionsEnabled;
  skipKickoffs.checked = state.skipKickoffsEnabled;
  emptyState.hidden = true;

  syncCameraControlAvailability(state);
  renderCameraProfile(state.attachedPlayerId);
  renderStats(state.frameIndex);
  renderFocusedPlayerOverlay(state);
}

async function loadReplay(file: File): Promise<void> {
  statusReadout.textContent = "Parsing replay...";
  setTransportEnabled(false);
  syncCameraControlAvailability();
  emptyState.hidden = false;

  if (unsubscribe) {
    unsubscribe();
    unsubscribe = null;
  }

  teardownActiveModules();
  replayPlayer?.destroy();
  replayPlayer = null;
  statsTimeline = null;
  statsFrameLookup = null;
  dynamicStatsTimeline = null;
  dynamicStatsFrameLookup = null;
  renderModuleSettings();

  const bytes = new Uint8Array(await file.arrayBuffer());
  const { replay } = await loadReplayFromBytes(bytes);
  const maybeInit = (subtrActor as typeof subtrActor & {
    default?: () => Promise<unknown>;
  }).default;
  if (typeof maybeInit === "function") {
    await maybeInit();
  }

  statsTimeline = subtrActor.get_stats_timeline(bytes) as unknown as StatsTimeline;
  statsFrameLookup = createStatsFrameLookup(statsTimeline);
  dynamicStatsTimeline = subtrActor.get_dynamic_stats_timeline(bytes) as unknown as DynamicStatsTimeline;
  dynamicStatsFrameLookup = createDynamicStatsFrameLookup(dynamicStatsTimeline);

  replayPlayer = new ReplayPlayer(viewport, replay, {
    initialCameraDistanceScale: DEFAULT_CAMERA_DISTANCE_SCALE,
    initialAttachedPlayerId: null,
    initialBallCamEnabled: false,
    initialSkipPostGoalTransitionsEnabled: skipPostGoalTransitions.checked,
    initialSkipKickoffsEnabled: skipKickoffs.checked,
    plugins: [
      createBallchasingOverlayPlugin(),
      createBoostPadOverlayPlugin(),
      createTimelineOverlayPlugin({
        replayEventKinds: ["goal", "save", "demo"],
      }),
    ],
  });

  setupActiveModules();
  unsubscribe = replayPlayer.subscribe(renderSnapshot);

  populateAttachedPlayerOptions(replay.players);
  emptyState.hidden = true;
  statusReadout.textContent = `Loaded ${file.name}`;
  playersReadout.textContent = replay.players.map((player) => player.name).join(", ");
  framesReadout.textContent = `${replay.frameCount}`;
  eventsReadout.textContent = `${replay.timelineEvents.length}`;
  setTransportEnabled(true);
  syncCameraControlAvailability(replayPlayer.getState());
  renderSnapshot(replayPlayer.getState());
  renderModuleSettings();
}

fileInput.addEventListener("change", async () => {
  const file = fileInput.files?.[0];
  if (!file) return;

  try {
    await loadReplay(file);
  } catch (error) {
    console.error("Failed to load replay:", error);
    statusReadout.textContent =
      error instanceof Error ? error.message : "Failed to load replay";
  }
});

togglePlayback.addEventListener("click", () => {
  replayPlayer?.togglePlayback();
});

playbackRate.addEventListener("change", () => {
  replayPlayer?.setPlaybackRate(Number(playbackRate.value));
});

cameraDistance.addEventListener("input", () => {
  replayPlayer?.setCameraDistanceScale(Number(cameraDistance.value));
});

attachedPlayer.addEventListener("change", () => {
  replayPlayer?.setAttachedPlayer(attachedPlayer.value || null);
});

ballCam.addEventListener("change", () => {
  replayPlayer?.setBallCamEnabled(ballCam.checked);
});

showFollowedPlayerOverlay.addEventListener("change", () => {
  renderFocusedPlayerOverlay(replayPlayer?.getState());
});

skipPostGoalTransitions.addEventListener("change", () => {
  replayPlayer?.setSkipPostGoalTransitionsEnabled(
    skipPostGoalTransitions.checked,
  );
});

skipKickoffs.addEventListener("change", () => {
  replayPlayer?.setSkipKickoffsEnabled(skipKickoffs.checked);
});

renderModuleSummary();
renderModuleSettings();
renderCameraProfile(null);
