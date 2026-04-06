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
  positioning: PlayerStatsSnapshot["positioning"] | undefined,
  zone: "defensive" | "neutral" | "offensive",
): number | undefined {
  switch (zone) {
    case "defensive":
      return positioning?.time_defensive_third;
    case "neutral":
      return positioning?.time_neutral_third;
    case "offensive":
      return positioning?.time_offensive_third;
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

function formatTimeShare(
  value: number | undefined,
  percentage: number | undefined,
  timeDigits = 1,
  percentageDigits = 0,
): string {
  if (value === undefined || Number.isNaN(value)) {
    return formatPercentage(percentage, percentageDigits);
  }

  const timeDisplay = formatNumber(value, timeDigits, "s");
  if (percentage === undefined || Number.isNaN(percentage)) {
    return timeDisplay;
  }

  return `${timeDisplay} (${formatPercentage(percentage, percentageDigits)})`;
}

function formatTimeShareFromTrackedTime(
  value: number | undefined,
  trackedTime: number | undefined,
  timeDigits = 1,
  percentageDigits = 0,
): string {
  const percentage = (
    value !== undefined &&
    trackedTime !== undefined &&
    !Number.isNaN(value) &&
    !Number.isNaN(trackedTime) &&
    trackedTime > 0
  )
    ? (value * 100) / trackedTime
    : undefined;

  return formatTimeShare(value, percentage, timeDigits, percentageDigits);
}

function asNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value)
    ? value
    : undefined;
}

function getPositioningTrackedTime(
  positioning: PlayerStatsSnapshot["positioning"] | undefined,
): number | undefined {
  return asNumber(positioning?.tracked_time);
}

function getPositioningPercentage(
  positioning: PlayerStatsSnapshot["positioning"] | undefined,
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

function formatPositioningTimeShare(
  positioning: PlayerStatsSnapshot["positioning"] | undefined,
  percentFieldName: string,
  timeFieldName: string,
): string {
  return formatTimeShare(
    asNumber(positioning?.[timeFieldName as keyof NonNullable<typeof positioning>]),
    getPositioningPercentage(positioning, percentFieldName, timeFieldName),
  );
}

function getPositioningAverage(
  positioning: PlayerStatsSnapshot["positioning"] | undefined,
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
  positioning: PlayerStatsSnapshot["positioning"] | undefined,
): string {
  return `
    <div class="stat-row"><span class="label">Most back</span><span class="value">${formatPositioningTimeShare(positioning, "percent_most_back", "time_most_back")}</span></div>
    <div class="stat-row"><span class="label">Most forward</span><span class="value">${formatPositioningTimeShare(positioning, "percent_most_forward", "time_most_forward")}</span></div>
    <div class="stat-row"><span class="label">Mid role</span><span class="value">${formatPositioningTimeShare(positioning, "percent_mid_role", "time_mid_role")}</span></div>
    <div class="stat-row"><span class="label">Other role</span><span class="value">${formatPositioningTimeShare(positioning, "percent_other_role", "time_other_role")}</span></div>
    <div class="stat-row"><span class="label">Closest to ball</span><span class="value">${formatPositioningTimeShare(positioning, "percent_closest_to_ball", "time_closest_to_ball")}</span></div>
    <div class="stat-row"><span class="label">Farthest from ball</span><span class="value">${formatPositioningTimeShare(positioning, "percent_farthest_from_ball", "time_farthest_from_ball")}</span></div>
    <div class="stat-row"><span class="label">Behind ball</span><span class="value">${formatPositioningTimeShare(positioning, "percent_behind_ball", "time_behind_ball")}</span></div>
    <div class="stat-row"><span class="label">Level with ball</span><span class="value">${formatPositioningTimeShare(positioning, "percent_level_with_ball", "time_level_with_ball")}</span></div>
    <div class="stat-row"><span class="label">In front of ball</span><span class="value">${formatPositioningTimeShare(positioning, "percent_in_front_of_ball", "time_in_front_of_ball")}</span></div>
  `;
}

export function renderAbsolutePositioningStats(
  positioning: PlayerStatsSnapshot["positioning"] | undefined,
): string {
  return `
    <div class="stat-row"><span class="label">Defensive zone</span><span class="value">${formatPositioningTimeShare(positioning, "percent_defensive_third", "time_defensive_third")}</span></div>
    <div class="stat-row"><span class="label">Neutral zone</span><span class="value">${formatPositioningTimeShare(positioning, "percent_neutral_third", "time_neutral_third")}</span></div>
    <div class="stat-row"><span class="label">Offensive zone</span><span class="value">${formatPositioningTimeShare(positioning, "percent_offensive_third", "time_offensive_third")}</span></div>
    <div class="stat-row"><span class="label">Defensive half</span><span class="value">${formatPositioningTimeShare(positioning, "percent_defensive_half", "time_defensive_half")}</span></div>
    <div class="stat-row"><span class="label">Offensive half</span><span class="value">${formatPositioningTimeShare(positioning, "percent_offensive_half", "time_offensive_half")}</span></div>
    <div class="stat-row"><span class="label">To teammates</span><span class="value">${formatNumber(getPositioningAverage(positioning, "average_distance_to_teammates", "sum_distance_to_teammates"), 0)}</span></div>
    <div class="stat-row"><span class="label">To ball</span><span class="value">${formatNumber(getPositioningAverage(positioning, "average_distance_to_ball", "sum_distance_to_ball"), 0)}</span></div>
  `;
}

export function renderCoreStats(core: PlayerStatsSnapshot["core"] | undefined): string {
  const shootingPercentage = core && core.shots > 0
    ? (core.goals * 100) / core.shots
    : undefined;
  return `
    <div class="stat-row"><span class="label">Score</span><span class="value">${formatInteger(core?.score)}</span></div>
    <div class="stat-row"><span class="label">Goals</span><span class="value">${formatInteger(core?.goals)}</span></div>
    <div class="stat-row"><span class="label">Assists</span><span class="value">${formatInteger(core?.assists)}</span></div>
    <div class="stat-row"><span class="label">Saves</span><span class="value">${formatInteger(core?.saves)}</span></div>
    <div class="stat-row"><span class="label">Shots</span><span class="value">${formatInteger(core?.shots)}</span></div>
    <div class="stat-row"><span class="label">Shooting %</span><span class="value">${formatPercentage(shootingPercentage)}</span></div>
  `;
}

export function renderBackboardStats(
  backboard: PlayerStatsSnapshot["backboard"] | undefined,
): string {
  return `
    <div class="stat-row"><span class="label">Hits</span><span class="value">${formatInteger(backboard?.count)}</span></div>
    <div class="stat-row"><span class="label">Since last</span><span class="value">${formatNumber(asNumber(backboard?.time_since_last_backboard), 2, "s")}</span></div>
  `;
}

export function renderDoubleTapStats(
  doubleTap: PlayerStatsSnapshot["double_tap"] | undefined,
): string {
  return `
    <div class="stat-row"><span class="label">Count</span><span class="value">${formatInteger(doubleTap?.count)}</span></div>
    <div class="stat-row"><span class="label">Since last</span><span class="value">${formatNumber(asNumber(doubleTap?.time_since_last_double_tap), 2, "s")}</span></div>
  `;
}

export function renderCeilingShotStats(
  ceilingShot: PlayerStatsSnapshot["ceiling_shot"] | undefined,
): string {
  const averageConfidence = ceilingShot && ceilingShot.count > 0
    ? ceilingShot.cumulative_confidence / ceilingShot.count
    : undefined;
  return `
    <div class="stat-row"><span class="label">Attempts</span><span class="value">${formatInteger(ceilingShot?.count)}</span></div>
    <div class="stat-row"><span class="label">High conf</span><span class="value">${formatInteger(ceilingShot?.high_confidence_count)}</span></div>
    <div class="stat-row"><span class="label">Last quality</span><span class="value">${formatNumber(asNumber(ceilingShot?.last_confidence), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Avg quality</span><span class="value">${formatNumber(averageConfidence, 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Best quality</span><span class="value">${formatNumber(asNumber(ceilingShot?.best_confidence), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Since last</span><span class="value">${formatNumber(asNumber(ceilingShot?.time_since_last_ceiling_shot), 2, "s")}</span></div>
  `;
}

export function renderBallCarryStats(
  ballCarry: PlayerStatsSnapshot["ball_carry"] | undefined,
): string {
  const averageHorizontalGap = ballCarry && ballCarry.carry_count > 0
    ? ballCarry.average_horizontal_gap_sum / ballCarry.carry_count
    : undefined;
  return `
    <div class="stat-row"><span class="label">Carries</span><span class="value">${formatInteger(ballCarry?.carry_count)}</span></div>
    <div class="stat-row"><span class="label">Total time</span><span class="value">${formatNumber(ballCarry?.total_carry_time, 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Longest</span><span class="value">${formatNumber(ballCarry?.longest_carry_time, 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Furthest</span><span class="value">${formatNumber(ballCarry?.furthest_carry_distance, 0)}</span></div>
    <div class="stat-row"><span class="label">Avg gap</span><span class="value">${formatNumber(averageHorizontalGap, 0)}</span></div>
  `;
}

export function renderPowerslideStats(
  powerslide: PlayerStatsSnapshot["powerslide"] | undefined,
): string {
  const averageDuration = powerslide && powerslide.press_count > 0
    ? powerslide.total_duration / powerslide.press_count
    : undefined;
  return `
    <div class="stat-row"><span class="label">Presses</span><span class="value">${formatInteger(powerslide?.press_count)}</span></div>
    <div class="stat-row"><span class="label">Total time</span><span class="value">${formatNumber(powerslide?.total_duration, 1, "s")}</span></div>
    <div class="stat-row"><span class="label">Avg duration</span><span class="value">${formatNumber(averageDuration, 2, "s")}</span></div>
  `;
}

export function renderDemoStats(demo: PlayerStatsSnapshot["demo"] | undefined): string {
  return `
    <div class="stat-row"><span class="label">Inflicted</span><span class="value">${formatInteger(demo?.demos_inflicted)}</span></div>
    <div class="stat-row"><span class="label">Taken</span><span class="value">${formatInteger(demo?.demos_taken)}</span></div>
  `;
}

export function renderDodgeResetStats(
  dodgeReset: PlayerStatsSnapshot["dodge_reset"] | undefined,
): string {
  return `
    <div class="stat-row"><span class="label">Count</span><span class="value">${formatInteger(dodgeReset?.count)}</span></div>
    <div class="stat-row"><span class="label">On ball</span><span class="value">${formatInteger(dodgeReset?.on_ball_count)}</span></div>
  `;
}

export function renderMustyFlickStats(
  mustyFlick: PlayerStatsSnapshot["musty_flick"] | undefined,
): string {
  const averageConfidence = mustyFlick && mustyFlick.count > 0
    ? mustyFlick.cumulative_confidence / mustyFlick.count
    : undefined;
  return `
    <div class="stat-row"><span class="label">Attempts</span><span class="value">${formatInteger(mustyFlick?.count)}</span></div>
    <div class="stat-row"><span class="label">High conf</span><span class="value">${formatInteger(mustyFlick?.high_confidence_count)}</span></div>
    <div class="stat-row"><span class="label">Last quality</span><span class="value">${formatNumber(asNumber(mustyFlick?.last_confidence), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Avg quality</span><span class="value">${formatNumber(averageConfidence, 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Best quality</span><span class="value">${formatNumber(asNumber(mustyFlick?.best_confidence), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Since last</span><span class="value">${formatNumber(asNumber(mustyFlick?.time_since_last_musty), 2, "s")}</span></div>
  `;
}

export function renderSpeedFlipStats(
  speedFlip: PlayerStatsSnapshot["speed_flip"] | undefined,
): string {
  const averageQuality = speedFlip && speedFlip.count > 0
    ? speedFlip.cumulative_quality / speedFlip.count
    : undefined;
  return `
    <div class="stat-row"><span class="label">Attempts</span><span class="value">${formatInteger(speedFlip?.count)}</span></div>
    <div class="stat-row"><span class="label">High conf</span><span class="value">${formatInteger(speedFlip?.high_confidence_count)}</span></div>
    <div class="stat-row"><span class="label">Last quality</span><span class="value">${formatNumber(asNumber(speedFlip?.last_quality), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Avg quality</span><span class="value">${formatNumber(averageQuality, 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Best quality</span><span class="value">${formatNumber(asNumber(speedFlip?.best_quality), 0, "%")}</span></div>
    <div class="stat-row"><span class="label">Since last</span><span class="value">${formatNumber(asNumber(speedFlip?.time_since_last_speed_flip), 2, "s")}</span></div>
  `;
}

export function renderBoostStats(boost: PlayerStatsSnapshot["boost"] | undefined): string {
  const avgBoost =
    boost && boost.tracked_time > 0
      ? toBoostDisplayUnits(boost.boost_integral / boost.tracked_time).toFixed(0)
      : "?";
  const trackedTime = asNumber(boost?.tracked_time);

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
    <div class="stat-row"><span class="label">Time @ 0</span><span class="value">${formatTimeShareFromTrackedTime(asNumber(boost?.time_zero_boost), trackedTime)}</span></div>
    <div class="stat-row"><span class="label">Time 0-25</span><span class="value">${formatTimeShareFromTrackedTime(asNumber(boost?.time_boost_0_25), trackedTime)}</span></div>
    <div class="stat-row"><span class="label">Time 25-50</span><span class="value">${formatTimeShareFromTrackedTime(asNumber(boost?.time_boost_25_50), trackedTime)}</span></div>
    <div class="stat-row"><span class="label">Time 50-75</span><span class="value">${formatTimeShareFromTrackedTime(asNumber(boost?.time_boost_50_75), trackedTime)}</span></div>
    <div class="stat-row"><span class="label">Time 75-100</span><span class="value">${formatTimeShareFromTrackedTime(asNumber(boost?.time_boost_75_100), trackedTime)}</span></div>
    <div class="stat-row"><span class="label">Time @ 100</span><span class="value">${formatTimeShareFromTrackedTime(asNumber(boost?.time_hundred_boost), trackedTime)}</span></div>
  `;
}
