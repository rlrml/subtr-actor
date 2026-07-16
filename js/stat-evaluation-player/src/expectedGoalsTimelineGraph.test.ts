import test from "node:test";
import assert from "node:assert/strict";

import { buildExpectedGoalsTimelineGraphs } from "./expectedGoalsTimelineGraph.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

test("instantaneous xG graph explains incident thresholds, selection, and exclusion", () => {
  const timeline = {
    frames: [
      { frame_number: 0, time: 0 },
      { frame_number: 10, time: 1 },
      { frame_number: 20, time: 2 },
      { frame_number: 30, time: 3 },
    ],
    expected_goals_tracks: {
      config: {
        episode_threshold: 0.15,
        episode_end_threshold: 0.05,
        goal_touch_exclusion_seconds: 0.5,
        incident_xg_calibration_factor: 0.583503,
      },
      teams: [
        {
          is_team_0: true,
          points: [
            {
              frame: 10,
              stats: {
                current_threat: 0.2,
                incident_xg: 0,
                xg: 0.02,
                episode_count: 0,
                goal_episode_count: 0,
              },
            },
            {
              frame: 20,
              stats: {
                current_threat: 0.7,
                incident_xg: 0,
                xg: 0.1,
                episode_count: 0,
                goal_episode_count: 0,
              },
            },
          ],
        },
        { is_team_0: false, points: [] },
      ],
      players: [],
      episodes: [
        {
          start_time: 1,
          start_frame: 10,
          end_time: 3,
          end_frame: 30,
          team_is_team_0: true,
          xg: 0.12,
          peak_value: 0.7,
          peak_frame: 20,
          peak_time: 2,
          incident_peak_value: 0.2,
          incident_xg: 0.125895,
          incident_xg_frame: 10,
          incident_xg_time: 1,
          goal_exclusion_start_time: 1.5,
          credited_player: null,
          ended_in_goal: true,
          end_reason: "goal",
        },
      ],
    },
  } as unknown as StatsTimeline;

  const [graph] = buildExpectedGoalsTimelineGraphs(timeline);
  assert.ok(graph);
  assert.deepEqual(
    graph.references?.map((reference) => reference.value),
    [0.15, 0.05],
  );
  assert.equal(graph.highlights?.length, 2);
  assert.equal(graph.highlights?.[1]?.className, "excluded");
  assert.equal(graph.highlights?.[1]?.startTime, 1.5);
  assert.equal(graph.markers?.length, 2);
  assert.equal(graph.markers?.[0]?.className, "selected");
  assert.equal(graph.markers?.[0]?.time, 1);
  assert.match(graph.markers?.[0]?.label ?? "", /contributes 0\.126 xG/);
  assert.equal(graph.markers?.[1]?.className, "excluded-peak");
  assert.equal(graph.markers?.[1]?.time, 2);
});
