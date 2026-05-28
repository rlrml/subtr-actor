import type { ReplayModel, ReplayTimelineRange } from "@rlrml/player";
import type { StatsTimeline } from "./statsTimeline.ts";
export {
  buildBoostPickupTimelineRanges,
  type BoostPickupTimelineRangeOptions,
} from "./timelineRangeBoostPickups.ts";
export {
  buildPossessionTimelineRanges,
  buildPressureTimelineRanges,
} from "./timelineRangesControl.ts";
export { buildMechanicTimelineRanges } from "./timelineRangesMechanics.ts";
export { buildTimeInZoneTimelineRanges } from "./timelineRangesTimeInZone.ts";

export function buildRushTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
): ReplayTimelineRange[] {
  return timeline.events.rush.map((event, index) => {
    const startTime = replay?.frames[event.start_frame]?.time ?? event.start_time;
    const endTime = replay?.frames[event.end_frame]?.time ?? event.end_time;
    const matchupLabel = `${event.attackers}v${event.defenders}`;
    const isTeamZero = event.is_team_0;

    return {
      id: `rush-range:${event.start_frame}:${event.end_frame}:${index}`,
      startTime,
      endTime: Math.max(startTime, endTime),
      lane: "rush",
      laneLabel: "Rush",
      label: `${isTeamZero ? "Blue" : "Orange"} rush ${matchupLabel}`,
      color: isTeamZero ? "rgba(59, 130, 246, 0.4)" : "rgba(245, 158, 11, 0.4)",
      isTeamZero,
    };
  });
}
