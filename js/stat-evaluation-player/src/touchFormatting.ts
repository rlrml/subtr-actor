import type { PlayerStatsSnapshot } from "./statsTimeline.ts";

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

export function renderTouchStats(
  touch: PlayerStatsSnapshot["touch"],
): string {
  return `
    <div class="stat-row"><span class="label">Touches</span><span class="value">${formatInteger(touch?.touch_count)}</span></div>
    <div class="stat-row"><span class="label">Dribbles</span><span class="value">${formatInteger(touch?.dribble_touch_count)}</span></div>
    <div class="stat-row"><span class="label">Control</span><span class="value">${formatInteger(touch?.control_touch_count)}</span></div>
    <div class="stat-row"><span class="label">Medium</span><span class="value">${formatInteger(touch?.medium_hit_count)}</span></div>
    <div class="stat-row"><span class="label">Hard</span><span class="value">${formatInteger(touch?.hard_hit_count)}</span></div>
    <div class="stat-row"><span class="label">Aerials</span><span class="value">${formatInteger(touch?.aerial_touch_count)}</span></div>
    <div class="stat-row"><span class="label">High aerials</span><span class="value">${formatInteger(touch?.high_aerial_touch_count)}</span></div>
    <div class="stat-row"><span class="label">Current</span><span class="value">${touch?.is_last_touch ? "Yes" : "No"}</span></div>
    <div class="stat-row"><span class="label">Touch time</span><span class="value">${formatNumber(touch?.last_touch_time, 2, "s")}</span></div>
    <div class="stat-row"><span class="label">Touch frame</span><span class="value">${formatInteger(touch?.last_touch_frame)}</span></div>
    <div class="stat-row"><span class="label">Since touch</span><span class="value">${formatNumber(touch?.time_since_last_touch, 2, "s")}</span></div>
    <div class="stat-row"><span class="label">Frames since</span><span class="value">${formatInteger(touch?.frames_since_last_touch)}</span></div>
    <div class="stat-row"><span class="label">Last change</span><span class="value">${formatNumber(touch?.last_ball_speed_change, 1)}</span></div>
    <div class="stat-row"><span class="label">Avg change</span><span class="value">${formatNumber(touch?.average_ball_speed_change, 1)}</span></div>
    <div class="stat-row"><span class="label">Max change</span><span class="value">${formatNumber(touch?.max_ball_speed_change, 1)}</span></div>
  `;
}
