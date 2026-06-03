import test from "node:test";
import assert from "node:assert/strict";

import type { PositioningEvent } from "./generated/PositioningEvent.ts";
import type { StatsTimeline } from "./statsTimeline.ts";
import { buildTimeInZoneTimelineRanges } from "./timelineRanges.ts";
import { createStatsTimeline } from "./testStatsTimeline.ts";

function positioningEvent(
  overrides: Partial<PositioningEvent> &
    Pick<PositioningEvent, "frame" | "time" | "player" | "is_team_0">,
): PositioningEvent {
  return {
    frame: overrides.frame,
    time: overrides.time,
    player: overrides.player,
    is_team_0: overrides.is_team_0,
    active_game_time: 0,
    tracked_time: 0,
    sum_distance_to_teammates: 0,
    sum_distance_to_ball: 0,
    sum_distance_to_ball_has_possession: 0,
    time_has_possession: 0,
    sum_distance_to_ball_no_possession: 0,
    time_no_possession: 0,
    time_demolished: 0,
    time_no_teammates: 0,
    time_most_back: 0,
    time_most_forward: 0,
    time_mid_role: 0,
    time_other_role: 0,
    time_defensive_third: 0,
    time_neutral_third: 0,
    time_offensive_third: 0,
    time_defensive_half: 0,
    time_offensive_half: 0,
    time_closest_to_ball: 0,
    time_farthest_from_ball: 0,
    time_behind_ball: 0,
    time_level_with_ball: 0,
    time_in_front_of_ball: 0,
    times_caught_ahead_of_play_on_conceded_goals: 0,
    ...overrides,
  };
}

test("buildTimeInZoneTimelineRanges derives per-player third occupancy spans", () => {
  const timeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            positioning: {
              active_game_time: 1,
              time_defensive_zone: 1,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 1,
              time_offensive_half: 0,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
        ],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            positioning: {
              active_game_time: 2,
              time_defensive_zone: 1,
              time_neutral_zone: 1,
              time_offensive_zone: 0,
              time_defensive_half: 1,
              time_offensive_half: 1,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
        ],
      },
      {
        frame_number: 3,
        time: 3,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            positioning: {
              active_game_time: 3,
              time_defensive_zone: 1,
              time_neutral_zone: 1,
              time_offensive_zone: 1,
              time_defensive_half: 1,
              time_offensive_half: 2,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
        ],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildTimeInZoneTimelineRanges(timeline), [
    {
      id: "time-in-zone:Steam:blue-id:time_defensive_third:0.000",
      startTime: 0,
      endTime: 1,
      lane: "time-in-zone:Steam:blue-id",
      laneLabel: "Blue",
      label: "Def third",
      color: "rgba(89, 195, 255, 0.74)",
      isTeamZero: true,
    },
    {
      id: "time-in-zone:Steam:blue-id:time_neutral_third:1.000",
      startTime: 1,
      endTime: 2,
      lane: "time-in-zone:Steam:blue-id",
      laneLabel: "Blue",
      label: "Neutral third",
      color: "rgba(209, 217, 224, 0.68)",
      isTeamZero: true,
    },
    {
      id: "time-in-zone:Steam:blue-id:time_offensive_third:2.000",
      startTime: 2,
      endTime: 3,
      lane: "time-in-zone:Steam:blue-id",
      laneLabel: "Blue",
      label: "Off third",
      color: "rgba(255, 193, 92, 0.78)",
      isTeamZero: true,
    },
  ]);
});

test("buildTimeInZoneTimelineRanges derives spans from compact positioning events", () => {
  const playerId = { Steam: "blue-id" };
  const timeline = createStatsTimeline({
    events: {
      positioning: [
        positioningEvent({
          frame: 1,
          time: 1,
          player: playerId,
          is_team_0: true,
          time_defensive_third: 1,
        }),
        positioningEvent({
          frame: 2,
          time: 2,
          player: playerId,
          is_team_0: true,
          time_neutral_third: 1,
        }),
        positioningEvent({
          frame: 3,
          time: 3,
          player: playerId,
          is_team_0: true,
          time_offensive_third: 1,
        }),
      ],
    },
    frames: [
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        team_zero: {},
        team_one: {},
        players: [{ player_id: playerId, name: "Blue", is_team_0: true }],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 1,
        team_zero: {},
        team_one: {},
        players: [{ player_id: playerId, name: "Blue", is_team_0: true }],
      },
      {
        frame_number: 3,
        time: 3,
        dt: 1,
        team_zero: {},
        team_one: {},
        players: [{ player_id: playerId, name: "Blue", is_team_0: true }],
      },
    ],
  });

  assert.deepEqual(buildTimeInZoneTimelineRanges(timeline), [
    {
      id: "time-in-zone:Steam:blue-id:time_defensive_third:0.000",
      startTime: 0,
      endTime: 1,
      lane: "time-in-zone:Steam:blue-id",
      laneLabel: "Blue",
      label: "Def third",
      color: "rgba(89, 195, 255, 0.74)",
      isTeamZero: true,
    },
    {
      id: "time-in-zone:Steam:blue-id:time_neutral_third:1.000",
      startTime: 1,
      endTime: 2,
      lane: "time-in-zone:Steam:blue-id",
      laneLabel: "Blue",
      label: "Neutral third",
      color: "rgba(209, 217, 224, 0.68)",
      isTeamZero: true,
    },
    {
      id: "time-in-zone:Steam:blue-id:time_offensive_third:2.000",
      startTime: 2,
      endTime: 3,
      lane: "time-in-zone:Steam:blue-id",
      laneLabel: "Blue",
      label: "Off third",
      color: "rgba(255, 193, 92, 0.78)",
      isTeamZero: true,
    },
  ]);
});

test("buildTimeInZoneTimelineRanges merges continuous spans independently per player lane", () => {
  const timeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            positioning: {
              active_game_time: 1,
              time_defensive_zone: 1,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 1,
              time_offensive_half: 0,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
          {
            player_id: { Epic: "orange-id" },
            name: "Orange",
            is_team_0: false,
            positioning: {
              active_game_time: 1,
              time_defensive_zone: 1,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 1,
              time_offensive_half: 0,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
        ],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            positioning: {
              active_game_time: 2,
              time_defensive_zone: 2,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 2,
              time_offensive_half: 0,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
          {
            player_id: { Epic: "orange-id" },
            name: "Orange",
            is_team_0: false,
            positioning: {
              active_game_time: 2,
              time_defensive_zone: 2,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 2,
              time_offensive_half: 0,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
        ],
      },
      {
        frame_number: 3,
        time: 3,
        dt: 1,
        players: [
          {
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            positioning: {
              active_game_time: 3,
              time_defensive_zone: 3,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 3,
              time_offensive_half: 0,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
          {
            player_id: { Epic: "orange-id" },
            name: "Orange",
            is_team_0: false,
            positioning: {
              active_game_time: 3,
              time_defensive_zone: 3,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 3,
              time_offensive_half: 0,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
        ],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildTimeInZoneTimelineRanges(timeline), [
    {
      id: "time-in-zone:Steam:blue-id:time_defensive_third:0.000",
      startTime: 0,
      endTime: 3,
      lane: "time-in-zone:Steam:blue-id",
      laneLabel: "Blue",
      label: "Def third",
      color: "rgba(89, 195, 255, 0.74)",
      isTeamZero: true,
    },
    {
      id: "time-in-zone:Epic:orange-id:time_defensive_third:0.000",
      startTime: 0,
      endTime: 3,
      lane: "time-in-zone:Epic:orange-id",
      laneLabel: "Orange",
      label: "Def third",
      color: "rgba(255, 193, 92, 0.78)",
      isTeamZero: false,
    },
  ]);
});

test("buildTimeInZoneTimelineRanges uses player-relative colors for orange players", () => {
  const timeline = {
    replay_meta: {},
    timeline_events: [],
    frames: [
      {
        frame_number: 1,
        time: 1,
        dt: 1,
        players: [
          {
            player_id: { Epic: "orange-id" },
            name: "Orange",
            is_team_0: false,
            positioning: {
              active_game_time: 1,
              time_defensive_zone: 1,
              time_neutral_zone: 0,
              time_offensive_zone: 0,
              time_defensive_half: 1,
              time_offensive_half: 0,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
        ],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 1,
        players: [
          {
            player_id: { Epic: "orange-id" },
            name: "Orange",
            is_team_0: false,
            positioning: {
              active_game_time: 2,
              time_defensive_zone: 1,
              time_neutral_zone: 0,
              time_offensive_zone: 1,
              time_defensive_half: 1,
              time_offensive_half: 1,
              time_demolished: 0,
              time_no_teammates: 0,
              time_most_back: 0,
              time_most_forward: 0,
              time_mid_role: 0,
              time_other_role: 0,
            },
          },
        ],
      },
    ],
  } as StatsTimeline;

  assert.deepEqual(buildTimeInZoneTimelineRanges(timeline), [
    {
      id: "time-in-zone:Epic:orange-id:time_defensive_third:0.000",
      startTime: 0,
      endTime: 1,
      lane: "time-in-zone:Epic:orange-id",
      laneLabel: "Orange",
      label: "Def third",
      color: "rgba(255, 193, 92, 0.78)",
      isTeamZero: false,
    },
    {
      id: "time-in-zone:Epic:orange-id:time_offensive_third:1.000",
      startTime: 1,
      endTime: 2,
      lane: "time-in-zone:Epic:orange-id",
      laneLabel: "Orange",
      label: "Off third",
      color: "rgba(89, 195, 255, 0.74)",
      isTeamZero: false,
    },
  ]);
});
