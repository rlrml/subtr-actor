import type { PlayerStatsSnapshot } from "../statsTimeline.ts";
import { asNumber, formatNumber, formatTimeShare } from "./rendererFormatting.ts";

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
  const timeValue = asNumber(positioning?.[timeFieldName as keyof NonNullable<typeof positioning>]);
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
  const sumValue = asNumber(positioning?.[sumFieldName as keyof NonNullable<typeof positioning>]);
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
