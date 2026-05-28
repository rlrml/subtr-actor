import {
  formatCollectedWithRespawnBound,
  formatBoostDisplayAmount,
  toBoostDisplayUnits,
} from "../boostFormatting.ts";
import type { PlayerStatsSnapshot } from "../statsTimeline.ts";
import {
  asNumber,
  formatTimeShareFromTrackedTime,
} from "./rendererFormatting.ts";

export function renderBoostStats(boost: PlayerStatsSnapshot["boost"] | undefined): string {
  const avgBoost =
    boost && boost.tracked_time > 0
      ? toBoostDisplayUnits(boost.boost_integral / boost.tracked_time).toFixed(0)
      : "?";
  const trackedTime = asNumber(boost?.tracked_time);

  return `
    <div class="stat-row"><span class="label">Collected</span><span class="value">${formatCollectedWithRespawnBound(boost?.amount_collected, boost?.amount_respawned)}</span></div>
    <div class="stat-row"><span class="label">Inactive collected</span><span class="value">${formatBoostDisplayAmount(boost?.amount_collected_inactive)}</span></div>
    <div class="stat-row"><span class="label">Big pads amt</span><span class="value">${formatBoostDisplayAmount(boost?.amount_collected_big)}</span></div>
    <div class="stat-row"><span class="label">Small pads amt</span><span class="value">${formatBoostDisplayAmount(boost?.amount_collected_small)}</span></div>
    <div class="stat-row"><span class="label">Respawns</span><span class="value">${formatBoostDisplayAmount(boost?.amount_respawned)}</span></div>
    <div class="stat-row"><span class="label">Overfill</span><span class="value">${formatBoostDisplayAmount(boost?.overfill_total)}</span></div>
    <div class="stat-row"><span class="label">Used</span><span class="value">${formatBoostDisplayAmount(boost?.amount_used)}</span></div>
    <div class="stat-row"><span class="label">Used ground</span><span class="value">${formatBoostDisplayAmount(boost?.amount_used_while_grounded)}</span></div>
    <div class="stat-row"><span class="label">Used air</span><span class="value">${formatBoostDisplayAmount(boost?.amount_used_while_airborne)}</span></div>
    <div class="stat-row"><span class="label">Big pads</span><span class="value">${boost?.big_pads_collected ?? "?"}</span></div>
    <div class="stat-row"><span class="label">Small pads</span><span class="value">${boost?.small_pads_collected ?? "?"}</span></div>
    <div class="stat-row"><span class="label">Inactive big pads</span><span class="value">${boost?.big_pads_collected_inactive ?? "?"}</span></div>
    <div class="stat-row"><span class="label">Inactive small pads</span><span class="value">${boost?.small_pads_collected_inactive ?? "?"}</span></div>
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
