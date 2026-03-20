import type { Object3D } from "three";
import {
  formatCollectedWithRespawnBound,
  formatBoostDisplayAmount,
  toBoostDisplayUnits,
} from "../boostFormatting.ts";
import { createZoneBoundaryLines } from "../overlays.ts";
import type { PlayerStatsSnapshot } from "../statsTimeline.ts";
import type { StatModuleContext } from "./types.ts";

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

export const zoneBoundaryOverlayManager = createSharedZoneBoundaryOverlayManager();

function getPositioningZoneTime(
  positioning: PlayerStatsSnapshot["positioning"],
  zone: "defensive" | "neutral" | "offensive",
): number | undefined {
  switch (zone) {
    case "defensive":
      return positioning?.time_defensive_zone;
    case "neutral":
      return positioning?.time_neutral_zone;
    case "offensive":
      return positioning?.time_offensive_zone;
  }
}

function formatInteger(value: number | undefined): string {
  return value === undefined || Number.isNaN(value)
    ? "?"
    : `${Math.round(value)}`;
}

function formatNumber(
  value: number | undefined,
  digits = 1,
  suffix = "",
): string {
  return value === undefined || Number.isNaN(value)
    ? "?"
    : `${value.toFixed(digits)}${suffix}`;
}

function formatPercentage(
  value: number | undefined,
  digits = 0,
): string {
  return formatNumber(value, digits, "%");
}

function asNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value)
    ? value
    : undefined;
}

function getPositioningTrackedTime(
  positioning: PlayerStatsSnapshot["positioning"],
): number | undefined {
  return asNumber(positioning?.tracked_time);
}

function getPositioningPercentage(
  positioning: PlayerStatsSnapshot["positioning"],
  percentFieldName: string,
  timeFieldName: string,
): number | undefined {
  const serializedPercentage = asNumber(
    positioning?.[percentFieldName as keyof NonNullable<typeof positioning>],
  );
  if (serializedPercentage !== undefined) {
    return serializedPercentage;
  }

  const trackedTime = getPositioningTrackedTime(positioning);
  const timeValue = asNumber(
    positioning?.[timeFieldName as keyof NonNullable<typeof positioning>],
  );
  if (trackedTime === undefined || trackedTime <= 0 || timeValue === undefined) {
    return undefined;
  }

  return (timeValue * 100) / trackedTime;
}

function getPositioningAverage(
  positioning: PlayerStatsSnapshot["positioning"],
  averageFieldName: string,
  sumFieldName: string,
): number | undefined {
  const serializedAverage = asNumber(
    positioning?.[averageFieldName as keyof NonNullable<typeof positioning>],
  );
  if (serializedAverage !== undefined) {
    return serializedAverage;
  }

  const trackedTime = getPositioningTrackedTime(positioning);
  const sumValue = asNumber(
    positioning?.[sumFieldName as keyof NonNullable<typeof positioning>],
  );
  if (trackedTime === undefined || trackedTime <= 0 || sumValue === undefined) {
    return undefined;
  }

  return sumValue / trackedTime;
}

export function renderRelativePositioningStats(
  positioning: PlayerStatsSnapshot["positioning"],
): string {
  return `
    <div class="stat-row"><span class="label">Most back</span><span class="value">${formatPercentage(getPositioningPercentage(positioning, "percent_most_back", "time_most_back"))}</span></div>
    <div class="stat-row"><span class="label">Most forward</span><span class="value">${formatPercentage(getPositioningPercentage(positioning, "percent_most_forward", "time_most_forward"))}</span></div>
    <div class="stat-row"><span class="label">Mid role</span><span class="value">${formatPercentage(getPositioningPercentage(positioning, "percent_mid_role", "time_mid_role"))}</span></div>
    <div class="stat-row"><span class="label">Other role</span><span class="value">${formatPercentage(getPositioningPercentage(positioning, "percent_other_role", "time_other_role"))}</span></div>
    <div class="stat-row"><span class="label">Closest to ball</span><span class="value">${formatPercentage(getPositioningPercentage(positioning, "percent_closest_to_ball", "time_closest_to_ball"))}</span></div>
    <div class="stat-row"><span class="label">Farthest from ball</span><span class="value">${formatPercentage(getPositioningPercentage(positioning, "percent_farthest_from_ball", "time_farthest_from_ball"))}</span></div>
    <div class="stat-row"><span class="label">Behind ball</span><span class="value">${formatPercentage(getPositioningPercentage(positioning, "percent_behind_ball", "time_behind_ball"))}</span></div>
    <div class="stat-row"><span class="label">In front of ball</span><span class="value">${formatPercentage(getPositioningPercentage(positioning, "percent_in_front_of_ball", "time_in_front_of_ball"))}</span></div>
  `;
}

export function renderAbsolutePositioningStats(
  positioning: PlayerStatsSnapshot["positioning"],
): string {
  return `
    <div class="stat-row"><span class="label">Defensive zone</span><span class="value">${formatNumber(getPositioningZoneTime(positioning, "defensive"), 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Neutral zone</span><span class="value">${formatNumber(getPositioningZoneTime(positioning, "neutral"), 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Offensive zone</span><span class="value">${formatNumber(getPositioningZoneTime(positioning, "offensive"), 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Defensive half</span><span class="value">${formatNumber(asNumber(positioning?.time_defensive_half), 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Offensive half</span><span class="value">${formatNumber(asNumber(positioning?.time_offensive_half), 1, "s")}</span></div>
    <div class="stat-row"><span class="label">To teammates</span><span class="value">${formatNumber(getPositioningAverage(positioning, "average_distance_to_teammates", "sum_distance_to_teammates"), 0)}</span></div>
    <div class="stat-row"><span class="label">To ball</span><span class="value">${formatNumber(getPositioningAverage(positioning, "average_distance_to_ball", "sum_distance_to_ball"), 0)}</span></div>
  `;
}

export function renderCoreStats(core: PlayerStatsSnapshot["core"]): string {
  return `
    <div class="stat-row"><span class="label">Score</span><span class="value">${formatInteger(core?.score)}</span></div>
    <div class="stat-row"><span class="label">Goals</span><span class="value">${formatInteger(core?.goals)}</span></div>
    <div class="stat-row"><span class="label">Assists</span><span class="value">${formatInteger(core?.assists)}</span></div>
    <div class="stat-row"><span class="label">Saves</span><span class="value">${formatInteger(core?.saves)}</span></div>
    <div class="stat-row"><span class="label">Shots</span><span class="value">${formatInteger(core?.shots)}</span></div>
    <div class="stat-row"><span class="label">Backboard hits</span><span class="value">${formatInteger(core?.attacking_backboard_hit_count)}</span></div>
    <div class="stat-row"><span class="label">Double taps</span><span class="value">${formatInteger(core?.double_tap_count)}</span></div>
    <div class="stat-row"><span class="label">Shooting %</span><span class="value">${formatPercentage(asNumber(core?.shooting_percentage))}</span></div>
  `;
}

export function renderBallCarryStats(
  ballCarry: PlayerStatsSnapshot["ball_carry"],
): string {
  return `
    <div class="stat-row"><span class="label">Carries</span><span class="value">${formatInteger(ballCarry?.carry_count)}</span></div>
    <div class="stat-row"><span class="label">Total time</span><span class="value">${formatNumber(ballCarry?.total_carry_time, 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Longest</span><span class="value">${formatNumber(ballCarry?.longest_carry_time, 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Furthest</span><span class="value">${formatNumber(ballCarry?.furthest_carry_distance, 0)}</span></div>
    <div class="stat-row"><span class="label">Avg gap</span><span class="value">${formatNumber(asNumber(ballCarry?.average_horizontal_gap), 0)}</span></div>
  `;
}

export function renderPowerslideStats(
  powerslide: PlayerStatsSnapshot["powerslide"],
): string {
  return `
    <div class="stat-row"><span class="label">Presses</span><span class="value">${formatInteger(powerslide?.press_count)}</span></div>
    <div class="stat-row"><span class="label">Total time</span><span class="value">${formatNumber(powerslide?.total_duration, 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Avg duration</span><span class="value">${formatNumber(asNumber(powerslide?.average_duration), 2, "s")}</span></div>
  `;
}

export function renderDemoStats(demo: PlayerStatsSnapshot["demo"]): string {
  return `
    <div class="stat-row"><span class="label">Inflicted</span><span class="value">${formatInteger(demo?.demos_inflicted)}</span></div>
    <div class="stat-row"><span class="label">Taken</span><span class="value">${formatInteger(demo?.demos_taken)}</span></div>
  `;
}

export function renderDodgeResetStats(
  dodgeReset: PlayerStatsSnapshot["dodge_reset"],
): string {
  return `
    <div class="stat-row"><span class="label">Count</span><span class="value">${formatInteger(dodgeReset?.count)}</span></div>
    <div class="stat-row"><span class="label">On ball</span><span class="value">${formatInteger(dodgeReset?.on_ball_count)}</span></div>
  `;
}

export function renderMustyFlickStats(
  mustyFlick: PlayerStatsSnapshot["musty_flick"],
): string {
  return `
    <div class="stat-row"><span class="label">Attempts</span><span class="value">${formatInteger(mustyFlick?.count)}</span></div>
    <div class="stat-row"><span class="label">High conf</span><span class="value">${formatInteger(mustyFlick?.high_confidence_count)}</span></div>
    <div class="stat-row"><span class="label">Last quality</span><span class="value">${formatNumber(asNumber(mustyFlick?.last_quality), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Avg quality</span><span class="value">${formatNumber(asNumber(mustyFlick?.average_quality), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Best quality</span><span class="value">${formatNumber(asNumber(mustyFlick?.best_quality), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Since last</span><span class="value">${formatNumber(asNumber(mustyFlick?.time_since_last_musty_flick), 2, "s")}</span></div>
  `;
}

export function renderSpeedFlipStats(
  speedFlip: PlayerStatsSnapshot["speed_flip"],
): string {
  return `
    <div class="stat-row"><span class="label">Attempts</span><span class="value">${formatInteger(speedFlip?.count)}</span></div>
    <div class="stat-row"><span class="label">High conf</span><span class="value">${formatInteger(speedFlip?.high_confidence_count)}</span></div>
    <div class="stat-row"><span class="label">Last quality</span><span class="value">${formatNumber(asNumber(speedFlip?.last_quality), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Avg quality</span><span class="value">${formatNumber(asNumber(speedFlip?.average_quality), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Best quality</span><span class="value">${formatNumber(asNumber(speedFlip?.best_quality), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Since last</span><span class="value">${formatNumber(asNumber(speedFlip?.time_since_last_speed_flip), 2, "s")}</span></div>
  `;
}

export function renderBoostStats(boost: PlayerStatsSnapshot["boost"]): string {
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
    <div class="stat-row"><span class="label">Avg boost</span><span class="value">${avgBoost}</span></div>
    <div class="stat-row"><span class="label">Time @ 0</span><span class="value">${boost?.time_zero_boost?.toFixed(1) ?? "?"}s</span></div>
    <div class="stat-row"><span class="label">Time @ 100</span><span class="value">${boost?.time_hundred_boost?.toFixed(1) ?? "?"}s</span></div>
  `;
}
