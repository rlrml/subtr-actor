import type { PlayerStatsSnapshot } from "../statsTimeline.ts";
import {
  asNumber,
  formatInteger,
  formatNumber,
  formatTimeShareFromTrackedTime,
} from "./rendererFormatting.ts";

function formatRotationTimeShare(
  rotation: PlayerStatsSnapshot["rotation"] | undefined,
  timeFieldName: keyof NonNullable<PlayerStatsSnapshot["rotation"]>,
): string {
  return formatTimeShareFromTrackedTime(
    asNumber(rotation?.[timeFieldName]),
    asNumber(rotation?.tracked_time),
  );
}

function formatStateLabel(value: string | undefined): string {
  if (!value) {
    return "?";
  }
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

export function renderRotationStats(rotation: PlayerStatsSnapshot["rotation"] | undefined): string {
  const averageFirstManStint =
    rotation && rotation.first_man_stint_count > 0
      ? rotation.time_first_man / rotation.first_man_stint_count
      : undefined;
  return `
    <div class="stat-row"><span class="label">Current role</span><span class="value">${formatStateLabel(rotation?.current_role_state)}</span></div>
    <div class="stat-row"><span class="label">Current depth</span><span class="value">${formatStateLabel(rotation?.current_depth_state)}</span></div>
    <div class="stat-row"><span class="label">First man</span><span class="value">${formatRotationTimeShare(rotation, "time_first_man")}</span></div>
    <div class="stat-row"><span class="label">First stints</span><span class="value">${formatInteger(rotation?.first_man_stint_count)}</span></div>
    <div class="stat-row"><span class="label">Avg first stint</span><span class="value">${formatNumber(averageFirstManStint, 2, "s")}</span></div>
    <div class="stat-row"><span class="label">Longest first stint</span><span class="value">${formatNumber(rotation?.longest_first_man_stint_time, 2, "s")}</span></div>
    <div class="stat-row"><span class="label">Second man</span><span class="value">${formatRotationTimeShare(rotation, "time_second_man")}</span></div>
    <div class="stat-row"><span class="label">Third man</span><span class="value">${formatRotationTimeShare(rotation, "time_third_man")}</span></div>
    <div class="stat-row"><span class="label">Ambiguous</span><span class="value">${formatRotationTimeShare(rotation, "time_ambiguous_role")}</span></div>
    <div class="stat-row"><span class="label">Behind play</span><span class="value">${formatRotationTimeShare(rotation, "time_behind_play")}</span></div>
    <div class="stat-row"><span class="label">Level with play</span><span class="value">${formatRotationTimeShare(rotation, "time_level_with_play")}</span></div>
    <div class="stat-row"><span class="label">Ahead of play</span><span class="value">${formatRotationTimeShare(rotation, "time_ahead_of_play")}</span></div>
    <div class="stat-row"><span class="label">Became first</span><span class="value">${formatInteger(rotation?.became_first_man_count)}</span></div>
    <div class="stat-row"><span class="label">Lost first</span><span class="value">${formatInteger(rotation?.lost_first_man_count)}</span></div>
  `;
}
