import test from "node:test";
import assert from "node:assert/strict";

import { buildExpectedGoalsTimelineGraphs } from "./expectedGoalsTimelineGraph.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

test("chance xG and continuous threat have separate graph semantics", () => {
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
      },
      teams: [
        {
          is_team_0: true,
          points: [
            {
              frame: 10,
              stats: {
                current_threat: 0.2,
                evaluated_touch_count: 0,
                unavailable_touch_count: 0,
                threat_integral: 0,
                xg: 0.02,
                episode_count: 0,
                goal_episode_count: 0,
              },
            },
            {
              frame: 20,
              stats: {
                current_threat: 0.7,
                evaluated_touch_count: 0,
                unavailable_touch_count: 0,
                threat_integral: 0,
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
          peak_value: 0.7,
          peak_frame: 20,
          peak_time: 2,
          threat_integral: 0.125895,
          credited_player: null,
          ended_in_goal: true,
          end_reason: "goal",
        },
      ],
    },
  } as unknown as StatsTimeline;

  const [xg, graph] = buildExpectedGoalsTimelineGraphs(timeline);
  assert.ok(xg);
  assert.ok(graph);
  assert.equal(xg.label, "Expected goals (pre-touch)");
  assert.deepEqual(
    xg.series[0]?.points.map((point) => point.value),
    [0.02, 0.1],
  );
  assert.equal(graph.label, "Scoring threat (5s)");
  assert.deepEqual(
    graph.references?.map((reference) => reference.value),
    [0.15, 0.05],
  );
  assert.equal(graph.highlights?.length, 1);
  assert.equal(graph.markers?.length, 1);
  assert.equal(graph.markers?.[0]?.time, 2);
  assert.match(graph.markers?.[0]?.label ?? "", /peak threat 70.0%/);
});
