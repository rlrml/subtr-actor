import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "@rlrml/player";
import type { FlickEvent } from "./generated/FlickEvent.ts";
import { STATS_EVENT_STREAM_COUNT_TYPES } from "./eventCountDerivation.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import type { Event } from "./statsTimeline.ts";
import {
  buildEventPlaylistItems,
  getEventPlaylistSelectedSourceIds,
  getEventPlaylistSources,
  getEventTimelineSources,
} from "./eventTimelineSources.ts";
import { EVENT_TYPE_TIMELINE_KINDS } from "./timelineMarkers.ts";
import { createLegacyStatsTimeline, createStatsTimeline } from "./testStatsTimeline.ts";

function createSourceTestContext(): {
  ctx: StatModuleContext;
  replay: ReplayModel;
} {
  const replay = {
    frames: Array.from({ length: 13 }, (_, frame) => ({ time: frame === 12 ? 1.25 : frame })),
    players: [{ id: "Steam:blue-id", name: "Blue" }],
    timelineEvents: [],
  } as ReplayModel;
  return {
    replay,
    ctx: {
      replay,
      statsTimeline: createStatsTimeline(),
    } as StatModuleContext,
  };
}

function getTestTimelineSources(ctx: StatModuleContext) {
  return getEventTimelineSources({
    ctx,
    modules: [],
    activeTimelineEventSourceIds: new Set(),
    activeMechanicTimelineKinds: new Set(),
    toggleEventSource() {},
    setMechanicTimelineKind() {},
  });
}

test("event timeline sources include empty canonical stats streams", () => {
  const { ctx } = createSourceTestContext();
  const timelineSources = getTestTimelineSources(ctx);
  const sourceIds = new Set(timelineSources.map((source) => source.id));

  assert.ok(sourceIds.has("stats-stream:player_activity"));
  assert.ok(sourceIds.has("stats-stream:backboard"));
  assert.ok(sourceIds.has("stats-stream:rotation_role"));
  assert.ok(sourceIds.has("stats-stream:dodge"));
  assert.ok(sourceIds.has("mechanic:speed_flip"));
  assert.ok(sourceIds.has("stats-stream:territorial_pressure"));

  const streamSources = timelineSources.filter((source) => source.id.startsWith("stats-stream:"));
  for (const streamId of STATS_EVENT_STREAM_COUNT_TYPES) {
    assert.ok(sourceIds.has(`stats-stream:${streamId}`), `missing stats stream ${streamId}`);
  }

  for (const mechanicKind of EVENT_TYPE_TIMELINE_KINDS) {
    assert.ok(sourceIds.has(`mechanic:${mechanicKind}`), `missing mechanic ${mechanicKind}`);
  }

  // air_dribble and flip_reset exist only as goal tags (no standalone events),
  // and wavedash is owned by its stat module, so none get an Event-types lane.
  assert.ok(!sourceIds.has("mechanic:air_dribble"));
  assert.ok(!sourceIds.has("mechanic:flip_reset"));
  assert.ok(!sourceIds.has("mechanic:wavedash"));

  assert.equal(
    streamSources.every((source) => source.count === 0),
    true,
  );
});

test("event playlist filters include empty canonical stats streams", () => {
  const { ctx } = createSourceTestContext();
  const dodgeModule = {
    id: "dodge",
    label: "Dodge",
    getTimelineEvents: () => [],
  } satisfies StatModule;
  const playlistSources = getEventPlaylistSources(
    ctx,
    getEventTimelineSources({
      ctx,
      modules: [dodgeModule],
      activeTimelineEventSourceIds: new Set(),
      activeMechanicTimelineKinds: new Set(),
      toggleEventSource() {},
      setMechanicTimelineKind() {},
    }),
  );
  const playlistSourceIds = new Set(playlistSources.map((source) => source.id));
  const selectedSourceIds = getEventPlaylistSelectedSourceIds(playlistSources, null);

  assert.ok(playlistSourceIds.has("stats-stream:player_activity"));
  assert.ok(playlistSourceIds.has("stats-stream:backboard"));
  assert.ok(playlistSourceIds.has("stats-stream:territorial_pressure"));
  assert.ok(playlistSourceIds.has("mechanic:speed_flip"));
  assert.ok(playlistSourceIds.has("module:dodge"));
  assert.equal(selectedSourceIds.has("module:dodge"), false);
  assert.equal(
    playlistSources.find((source) => source.id === "stats-stream:player_activity")?.events.length,
    0,
  );
});

test("typed mechanic payloads populate their Event-types lane", () => {
  const replay = {
    frames: Array.from({ length: 4 }, (_, frame) => ({ time: frame })),
    players: [{ id: "Steam:blue-id", name: "Blue" }],
    timelineEvents: [],
  } as ReplayModel;
  const statsTimeline = createLegacyStatsTimeline({
    flick_events: [
      {
        time: 2,
        frame: 2,
        player: { Steam: "blue-id" },
        is_team_0: true,
        kind: "other",
      } as unknown as FlickEvent,
    ],
  });
  const ctx = { replay, statsTimeline } as StatModuleContext;

  const flickSource = getTestTimelineSources(ctx).find((source) => source.id === "mechanic:flick");
  assert.ok(flickSource, "mechanic:flick lane should exist");
  assert.equal(flickSource.count, 1);
  assert.equal(flickSource.buildTimelineEvents().length, 1);
  assert.equal(flickSource.buildTimelineEvents()[0]?.frame, 2);
});

test("per-player span streams fan out into one lane per player", () => {
  const replay = {
    frames: Array.from({ length: 8 }, (_, frame) => ({ time: frame })),
    players: [
      { id: "Steam:blue-id", name: "Blue" },
      { id: "Steam:orange-id", name: "Orange" },
    ],
    timelineEvents: [],
  } as ReplayModel;
  const statsTimeline = createStatsTimeline({
    events: {
      player_activity: [
        {
          time: 1,
          frame: 1,
          end_time: 3,
          end_frame: 3,
          duration: 2,
          player: { Steam: "blue-id" },
          is_team_0: true,
          state: "tracked",
        },
        {
          time: 2,
          frame: 2,
          end_time: 5,
          end_frame: 5,
          duration: 3,
          player: { Steam: "orange-id" },
          is_team_0: false,
          state: "tracked",
        },
      ],
    },
  });
  const ctx = { replay, statsTimeline } as StatModuleContext;

  const source = getTestTimelineSources(ctx).find(
    (candidate) => candidate.id === "stats-stream:player_activity",
  );
  assert.ok(source, "player_activity stream source should exist");
  const lanes = new Set((source.buildTimelineRanges?.() ?? []).map((range) => range.lane));
  assert.deepEqual([...lanes].sort(), [
    "stats-stream:player_activity:player:Steam:blue-id",
    "stats-stream:player_activity:player:Steam:orange-id",
  ]);
});

test("event playlist sources include generic stats event streams such as positioning activity", () => {
  const replay = {
    frames: Array.from({ length: 13 }, (_, frame) => ({ time: frame === 12 ? 1.25 : frame })),
    players: [{ id: "Steam:blue-id", name: "Blue" }],
    timelineEvents: [],
  } as ReplayModel;
  const statsTimeline = createStatsTimeline({
    events: {
      player_activity: [
        {
          time: 1.2,
          frame: 11,
          end_time: 1.25,
          end_frame: 12,
          duration: 0.05,
          player: { Steam: "blue-id" },
          is_team_0: true,
          state: "tracked",
        },
      ],
    },
  });
  const toggled: Array<{ id: string; enabled: boolean }> = [];
  const ctx = {
    replay,
    statsTimeline,
  } as StatModuleContext;

  const timelineSources = getEventTimelineSources({
    ctx,
    modules: [],
    activeTimelineEventSourceIds: new Set(),
    activeMechanicTimelineKinds: new Set(),
    toggleEventSource(id, enabled) {
      toggled.push({ id, enabled });
    },
    setMechanicTimelineKind() {
      throw new Error("unexpected mechanic toggle");
    },
  });

  const playerActivitySource = timelineSources.find(
    (source) => source.id === "stats-stream:player_activity",
  );
  assert.ok(playerActivitySource);
  assert.equal(playerActivitySource.group, "Event streams");
  assert.equal(playerActivitySource.label, "Player Activity");
  assert.equal(playerActivitySource.count, 1);
  assert.equal(playerActivitySource.active, false);

  playerActivitySource.setActive(true);
  assert.deepEqual(toggled, [{ id: "stats-stream:player_activity", enabled: true }]);

  const playlistSources = getEventPlaylistSources(ctx, timelineSources);
  const playlistSource = playlistSources.find(
    (source) => source.id === "stats-stream:player_activity",
  );
  assert.ok(playlistSource);
  assert.equal(
    getEventPlaylistSelectedSourceIds(playlistSources, null).has("stats-stream:player_activity"),
    false,
  );

  const items = buildEventPlaylistItems({
    sources: playlistSources,
    activeSourceIds: new Set(["stats-stream:player_activity"]),
    replayPlayers: replay.players,
  });

  assert.deepEqual(
    items.map((item) => ({
      sourceId: item.sourceId,
      sourceLabel: item.sourceLabel,
      time: item.event.time,
      frame: item.event.frame,
      label: item.event.label,
      shortLabel: item.event.shortLabel,
      playerId: item.event.playerId,
      playerName: item.event.playerName,
      isTeamZero: item.event.isTeamZero,
    })),
    [
      {
        sourceId: "stats-stream:player_activity",
        sourceLabel: "Player Activity",
        time: 1.25,
        frame: 12,
        label: "Blue positioning tracked | 0.05s",
        shortLabel: "PA",
        playerId: "Steam:blue-id",
        playerName: "Blue",
        isTeamZero: true,
      },
    ],
  );
});

test("controlled play timeline source renders spans as timeline ranges", () => {
  const replay = {
    frames: Array.from({ length: 13 }, (_, frame) => ({
      time: frame === 5 ? 0.55 : frame === 12 ? 1.25 : frame,
    })),
    players: [{ id: "Steam:blue-id", name: "Blue" }],
    timelineEvents: [],
  } as ReplayModel;
  const statsTimeline = createStatsTimeline({
    events: {
      controlled_play: [
        {
          player_id: { Steam: "blue-id" },
          is_team_0: true,
          start_frame: 5,
          end_frame: 12,
          start_time: 0.5,
          end_time: 1.2,
          duration: 0.7,
          first_touch_frame: 5,
          last_touch_frame: 12,
          first_touch_time: 0.5,
          last_touch_time: 1.2,
          touch_count: 2,
          close_duration: 0.6,
          total_advance_distance: 400,
        },
      ],
    },
  });
  const ctx = {
    replay,
    statsTimeline,
  } as StatModuleContext;

  const timelineSources = getTestTimelineSources(ctx);
  const controlledPlaySource = timelineSources.find(
    (source) => source.id === "stats-stream:controlled_play",
  );

  assert.ok(controlledPlaySource);
  assert.equal(controlledPlaySource.count, 1);
  assert.deepEqual(controlledPlaySource.buildTimelineEvents(), []);
  assert.deepEqual(controlledPlaySource.buildTimelineRanges?.(), [
    {
      id: "stats-stream:controlled_play:5:12:0",
      startTime: 0.55,
      endTime: 1.25,
      lane: "stats-stream:controlled_play:team:0",
      laneLabel: "Blue controlled play",
      label: "Blue controlled play",
      shortLabel: "CP",
      isTeamZero: true,
      color: "#3b82f6",
    },
  ]);
});

test("generic ball half playlist events include field half and duration", () => {
  const replay = {
    frames: Array.from({ length: 8 }, (_, frame) => ({ time: frame * 0.5 })),
    players: [],
    timelineEvents: [],
  } as ReplayModel;
  const statsTimeline = createStatsTimeline({
    events: {
      ball_half: [
        {
          time: 1,
          frame: 2,
          end_time: 2.5,
          end_frame: 5,
          active: true,
          duration: 1.5,
          field_half: "team_zero_side",
        },
        {
          time: 2.5,
          frame: 5,
          end_time: 3,
          end_frame: 6,
          active: false,
          duration: 0.5,
          field_half: "neutral",
        },
      ],
    },
  });
  const ctx = {
    replay,
    statsTimeline,
  } as StatModuleContext;
  const timelineSources = getTestTimelineSources(ctx);
  const ballHalfSource = timelineSources.find((source) => source.id === "stats-stream:ball_half");
  assert.ok(ballHalfSource);
  assert.deepEqual(ballHalfSource.buildTimelineRanges?.(), [
    {
      id: "stats-stream:ball_half:2:5:0",
      startTime: 1,
      endTime: 2.5,
      lane: "stats-stream:ball_half",
      laneLabel: "Ball Half",
      label: "Ball Half",
      shortLabel: "BH",
      isTeamZero: null,
      color: "#3b82f6",
    },
    {
      id: "stats-stream:ball_half:5:6:1",
      startTime: 2.5,
      endTime: 3,
      lane: "stats-stream:ball_half",
      laneLabel: "Ball Half",
      label: "Ball Half",
      shortLabel: "BH",
      isTeamZero: null,
      color: "#d1d9e0",
    },
  ]);

  const playlistSources = getEventPlaylistSources(ctx, timelineSources);

  const items = buildEventPlaylistItems({
    sources: playlistSources,
    activeSourceIds: new Set(["stats-stream:ball_half"]),
    replayPlayers: replay.players,
  });

  assert.deepEqual(
    items.map((item) => ({
      sourceId: item.sourceId,
      time: item.event.time,
      frame: item.event.frame,
      label: item.event.label,
      shortLabel: item.event.shortLabel,
      color: item.color,
    })),
    [
      {
        sourceId: "stats-stream:ball_half",
        time: 2.5,
        frame: 5,
        label: "Ball on blue side | 1.5s",
        shortLabel: "BH",
        color: "#3b82f6",
      },
      {
        sourceId: "stats-stream:ball_half",
        time: 3,
        frame: 6,
        label: "Ball half inactive | 0.5s",
        shortLabel: "BH",
        color: "#d1d9e0",
      },
    ],
  );
});

test("generic ball third playlist events include field third and duration", () => {
  const replay = {
    frames: Array.from({ length: 8 }, (_, frame) => ({ time: frame * 0.5 })),
    players: [],
    timelineEvents: [],
  } as ReplayModel;
  const statsTimeline = createStatsTimeline({
    events: {
      ball_third: [
        {
          time: 1,
          frame: 2,
          end_time: 2.5,
          end_frame: 5,
          active: true,
          duration: 1.5,
          field_third: "team_zero_third",
        },
        {
          time: 2.5,
          frame: 5,
          end_time: 3,
          end_frame: 6,
          active: false,
          duration: 0.5,
          field_third: "neutral_third",
        },
      ],
    },
  });
  const ctx = {
    replay,
    statsTimeline,
  } as StatModuleContext;
  const playlistSources = getEventPlaylistSources(ctx, getTestTimelineSources(ctx));

  const items = buildEventPlaylistItems({
    sources: playlistSources,
    activeSourceIds: new Set(["stats-stream:ball_third"]),
    replayPlayers: replay.players,
  });

  assert.deepEqual(
    items.map((item) => ({
      sourceId: item.sourceId,
      time: item.event.time,
      frame: item.event.frame,
      label: item.event.label,
      shortLabel: item.event.shortLabel,
      color: item.color,
    })),
    [
      {
        sourceId: "stats-stream:ball_third",
        time: 2.5,
        frame: 5,
        label: "Ball in blue third | 1.5s",
        shortLabel: "BT",
        color: "#d1d9e0",
      },
      {
        sourceId: "stats-stream:ball_third",
        time: 3,
        frame: 6,
        label: "Ball third inactive | 0.5s",
        shortLabel: "BT",
        color: "#d1d9e0",
      },
    ],
  );
});

test("generic possession streams derive colors from possession payloads", () => {
  const replay = {
    frames: Array.from({ length: 8 }, (_, frame) => ({ time: frame * 0.5 })),
    players: [
      { id: "Steam:blue-id", name: "Blue" },
      { id: "Steam:orange-id", name: "Orange" },
    ],
    timelineEvents: [],
  } as ReplayModel;
  const statsTimeline = createStatsTimeline({
    events: {
      possession: [
        {
          time: 1,
          frame: 2,
          end_time: 2.5,
          end_frame: 5,
          active: true,
          duration: 1.5,
          possession_state: "team_zero",
          player_id: null,
        },
        {
          time: 2.5,
          frame: 5,
          end_time: 3,
          end_frame: 6,
          active: true,
          duration: 0.5,
          possession_state: "team_one",
          player_id: null,
        },
        {
          time: 3,
          frame: 6,
          end_time: 3.5,
          end_frame: 7,
          active: true,
          duration: 0.5,
          possession_state: "neutral",
          player_id: null,
        },
      ],
      events: [
        {
          meta: {
            id: "player_possession:2:5:0",
            stream: "player_possession",
            label: "Player Possession",
            scope: "player",
            timing: {
              type: "span",
              start_time: 1,
              start_frame: 2,
              end_time: 2.5,
              end_frame: 5,
            },
            primary_player: { Steam: "orange-id" },
            properties: [],
          },
          payload: {
            kind: "player_possession",
            payload: {
              player_id: { Steam: "orange-id" },
              is_team_0: false,
              start_frame: 2,
              end_frame: 5,
              start_time: 1,
              end_time: 2.5,
              duration: 1.5,
              touch_count: 2,
              aerial_touch_count: 0,
              wall_touch_count: 0,
              advance_distance: 300,
              retreat_distance: 0,
              carry_time: 0,
              air_dribble_time: 0,
              carry_count: 0,
              air_dribble_count: 0,
              close_time: 1.2,
              sustained_control: true,
              start_field_third: "neutral_third",
              end_field_third: "team_zero_third",
            },
          },
        } satisfies Event,
      ],
    },
  });
  const ctx = {
    replay,
    statsTimeline,
  } as StatModuleContext;
  const timelineSources = getTestTimelineSources(ctx);
  const possessionSource = timelineSources.find(
    (source) => source.id === "stats-stream:possession",
  );
  const playerPossessionSource = timelineSources.find(
    (source) => source.id === "stats-stream:player_possession",
  );

  assert.ok(possessionSource);
  assert.deepEqual(possessionSource.buildTimelineRanges?.(), [
    {
      id: "stats-stream:possession:2:5:0",
      startTime: 1,
      endTime: 2.5,
      lane: "stats-stream:possession",
      laneLabel: "Possession",
      label: "Possession",
      shortLabel: "P",
      isTeamZero: null,
      color: "#3b82f6",
    },
    {
      id: "stats-stream:possession:5:6:1",
      startTime: 2.5,
      endTime: 3,
      lane: "stats-stream:possession",
      laneLabel: "Possession",
      label: "Possession",
      shortLabel: "P",
      isTeamZero: null,
      color: "#f97316",
    },
    {
      id: "stats-stream:possession:6:7:2",
      startTime: 3,
      endTime: 3.5,
      lane: "stats-stream:possession",
      laneLabel: "Possession",
      label: "Possession",
      shortLabel: "P",
      isTeamZero: null,
      color: "#d1d9e0",
    },
  ]);

  assert.ok(playerPossessionSource);
  assert.deepEqual(playerPossessionSource.buildTimelineRanges?.(), [
    {
      id: "stats-stream:player_possession:2:5:0",
      startTime: 1,
      endTime: 2.5,
      lane: "stats-stream:player_possession:player:Steam:orange-id",
      laneLabel: "Orange player possession",
      label: "Orange player possession",
      shortLabel: "PP",
      isTeamZero: null,
      color: "#f97316",
    },
  ]);

  const playlistSources = getEventPlaylistSources(ctx, timelineSources);
  const items = buildEventPlaylistItems({
    sources: playlistSources,
    activeSourceIds: new Set(["stats-stream:possession"]),
    replayPlayers: replay.players,
  });

  assert.deepEqual(
    items.map((item) => ({
      sourceId: item.sourceId,
      time: item.event.time,
      frame: item.event.frame,
      label: item.event.label,
      shortLabel: item.event.shortLabel,
      color: item.color,
    })),
    [
      {
        sourceId: "stats-stream:possession",
        time: 2.5,
        frame: 5,
        label: "Blue possession | 1.5s",
        shortLabel: "P",
        color: "#3b82f6",
      },
      {
        sourceId: "stats-stream:possession",
        time: 3,
        frame: 6,
        label: "Orange possession | 0.5s",
        shortLabel: "P",
        color: "#f97316",
      },
      {
        sourceId: "stats-stream:possession",
        time: 3.5,
        frame: 7,
        label: "Neutral possession | 0.5s",
        shortLabel: "P",
        color: "#d1d9e0",
      },
    ],
  );
});

test("span-based stats event streams build ranges instead of timeline markers", () => {
  const replay = {
    frames: Array.from({ length: 7 }, (_, frame) => ({ time: frame })),
    players: [],
    timelineEvents: [],
  } as ReplayModel;
  const statsTimeline = createStatsTimeline({
    events: {
      territorial_pressure: [
        {
          start_time: 1,
          start_frame: 1,
          end_time: 5,
          end_frame: 5,
          team_is_team_0: false,
          duration: 4,
          offensive_half_time: 4,
          offensive_third_time: 2,
          end_reason: "relieved",
        },
      ],
    },
  });
  const ctx = {
    replay,
    statsTimeline,
  } as StatModuleContext;

  const source = getEventTimelineSources({
    ctx,
    modules: [],
    activeTimelineEventSourceIds: new Set(["stats-stream:territorial_pressure"]),
    activeMechanicTimelineKinds: new Set(),
    toggleEventSource() {},
    setMechanicTimelineKind() {},
  }).find((candidate) => candidate.id === "stats-stream:territorial_pressure");

  assert.ok(source);
  assert.equal(source.count, 1);
  assert.deepEqual(source.buildTimelineEvents(), []);
  assert.deepEqual(source.buildTimelineRanges?.(), [
    {
      id: "stats-stream:territorial_pressure:1:5:0",
      startTime: 1,
      endTime: 5,
      lane: "stats-stream:territorial_pressure:team:1",
      laneLabel: "Orange territorial pressure",
      label: "Orange territorial pressure",
      shortLabel: "TP",
      isTeamZero: false,
      color: "#f97316",
    },
  ]);
});
