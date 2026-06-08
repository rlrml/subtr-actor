import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "@rlrml/player";
import {
  STATS_EVENT_STREAM_COUNT_TYPES,
  STATS_MECHANIC_EVENT_COUNT_TYPES,
} from "./eventCountDerivation.ts";
import type { StatModuleContext } from "./statModules.ts";
import {
  buildEventPlaylistItems,
  getEventPlaylistSelectedSourceIds,
  getEventPlaylistSources,
  getEventTimelineSources,
} from "./eventTimelineSources.ts";
import { createStatsTimeline } from "./testStatsTimeline.ts";

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

  assert.ok(sourceIds.has("stats-stream:positioning_activity"));
  assert.ok(sourceIds.has("stats-stream:backboard"));
  assert.ok(sourceIds.has("stats-stream:rotation_role_span"));
  assert.ok(sourceIds.has("stats-stream:flip_impulse"));
  assert.ok(sourceIds.has("mechanic:speed_flip"));

  const streamSources = timelineSources.filter((source) => source.id.startsWith("stats-stream:"));
  for (const streamId of STATS_EVENT_STREAM_COUNT_TYPES) {
    assert.ok(sourceIds.has(`stats-stream:${streamId}`), `missing stats stream ${streamId}`);
  }

  for (const mechanicKind of STATS_MECHANIC_EVENT_COUNT_TYPES.filter(
    (kind) => kind !== "wavedash",
  )) {
    assert.ok(sourceIds.has(`mechanic:${mechanicKind}`), `missing mechanic ${mechanicKind}`);
  }

  assert.equal(
    streamSources.every((source) => source.count === 0),
    true,
  );
});

test("event playlist filters include empty canonical stats streams", () => {
  const { ctx } = createSourceTestContext();
  const playlistSources = getEventPlaylistSources(ctx, getTestTimelineSources(ctx));
  const playlistSourceIds = new Set(playlistSources.map((source) => source.id));

  assert.ok(playlistSourceIds.has("stats-stream:positioning_activity"));
  assert.ok(playlistSourceIds.has("stats-stream:backboard"));
  assert.ok(playlistSourceIds.has("mechanic:speed_flip"));
  assert.equal(
    playlistSources.find((source) => source.id === "stats-stream:positioning_activity")?.events
      .length,
    0,
  );
});

test("event playlist sources include generic stats event streams such as positioning activity", () => {
  const replay = {
    frames: Array.from({ length: 13 }, (_, frame) => ({ time: frame === 12 ? 1.25 : frame })),
    players: [{ id: "Steam:blue-id", name: "Blue" }],
    timelineEvents: [],
  } as ReplayModel;
  const statsTimeline = createStatsTimeline({
    events: {
      positioning_activity: [
        {
          time: 1.2,
          frame: 11,
          end_time: 1.25,
          end_frame: 12,
          duration: 0.05,
          player: { Steam: "blue-id" },
          is_team_0: true,
          active: true,
          tracked: true,
          demolished: false,
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

  const positioningActivitySource = timelineSources.find(
    (source) => source.id === "stats-stream:positioning_activity",
  );
  assert.ok(positioningActivitySource);
  assert.equal(positioningActivitySource.group, "Event streams");
  assert.equal(positioningActivitySource.label, "Positioning Activity");
  assert.equal(positioningActivitySource.count, 1);
  assert.equal(positioningActivitySource.active, false);

  positioningActivitySource.setActive(true);
  assert.deepEqual(toggled, [{ id: "stats-stream:positioning_activity", enabled: true }]);

  const playlistSources = getEventPlaylistSources(ctx, timelineSources);
  const playlistSource = playlistSources.find(
    (source) => source.id === "stats-stream:positioning_activity",
  );
  assert.ok(playlistSource);
  assert.equal(
    getEventPlaylistSelectedSourceIds(playlistSources, null).has(
      "stats-stream:positioning_activity",
    ),
    false,
  );

  const items = buildEventPlaylistItems({
    sources: playlistSources,
    activeSourceIds: new Set(["stats-stream:positioning_activity"]),
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
        sourceId: "stats-stream:positioning_activity",
        sourceLabel: "Positioning Activity",
        time: 1.25,
        frame: 12,
        label: "Blue positioning activity",
        shortLabel: "PA",
        playerId: "Steam:blue-id",
        playerName: "Blue",
        isTeamZero: true,
      },
    ],
  );
});
