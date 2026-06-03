import test from "node:test";
import assert from "node:assert/strict";

import { normalizeReplayData, normalizeReplayDataAsync } from "../src/replay-data";
import { formatReplayLoadProgress, formatReplayLoadProgressMeta } from "../src/load-ui";
import type { RawReplayFramesData } from "../src/types";

function rigidBody(x: number, y: number, z: number) {
  return {
    sleeping: false,
    location: { x, y, z },
    rotation: { x: 0, y: 0, z: 0, w: 1 },
    linear_velocity: { x: x * 10, y: y * 10, z: z * 10 },
    angular_velocity: { x: 0, y: 0, z: 0 },
  };
}

function playerFrame(playerName: string, index: number) {
  return {
    Data: {
      rigid_body: rigidBody(index, 0, 17),
      boost_amount: 128,
      boost_active: false,
      powerslide_active: false,
      jump_active: false,
      double_jump_active: false,
      dodge_active: false,
      player_name: playerName,
      team: playerName.startsWith("Blue") ? 0 : 1,
      is_team_0: playerName.startsWith("Blue"),
    },
  };
}

function ballFrame(index: number) {
  return {
    Data: {
      rigid_body: rigidBody(index, 50, 93),
    },
  };
}

function buildReplayData(frameCount: number, playerCount = 4): RawReplayFramesData {
  const teamZero = Array.from({ length: Math.ceil(playerCount / 2) }, (_, index) => ({
    remote_id: { Steam: `blue-player-${index}` },
    stats: null,
    name: `Blue ${index + 1}`,
  }));
  const teamOne = Array.from({ length: Math.floor(playerCount / 2) }, (_, index) => ({
    remote_id: { Steam: `orange-player-${index}` },
    stats: null,
    name: `Orange ${index + 1}`,
  }));
  const players: RawReplayFramesData["frame_data"]["players"] = [...teamZero, ...teamOne].map(
    (player): RawReplayFramesData["frame_data"]["players"][number] => [
      player.remote_id,
      {
        frames: Array.from({ length: frameCount }, (_, index) => playerFrame(player.name, index)),
      },
    ],
  );

  return {
    meta: {
      team_zero: teamZero,
      team_one: teamOne,
      all_headers: [],
    },
    frame_data: {
      metadata_frames: Array.from({ length: frameCount }, (_, index) => ({
        time: 10 + index * 0.1,
        seconds_remaining: 300 - index * 0.1,
        replicated_game_state_name: 0,
        replicated_game_state_time_remaining: 0,
      })),
      ball_data: {
        frames: Array.from({ length: frameCount }, (_, index) => ballFrame(index)),
      },
      players,
    },
    demolish_infos: [],
    boost_pad_events: [],
    boost_pads: [],
    replay_tick_marks: [],
    touch_events: [],
    dodge_refreshed_events: [],
    player_stat_events: [],
    goal_events: [],
  };
}

function assertMonotonicProgress(progressValues: number[]): void {
  assert.equal(progressValues[0], 0);
  assert.equal(progressValues.at(-1), 1);

  for (let index = 1; index < progressValues.length; index += 1) {
    assert.ok(progressValues[index]! >= progressValues[index - 1]!, "expected monotonic progress");
  }
}

test("normalization progress is reported headlessly at a bounded cadence", () => {
  const progressValues: number[] = [];
  const frameCounts: number[] = [];

  normalizeReplayData(buildReplayData(240), {
    onProgress(progress, details) {
      progressValues.push(progress);
      frameCounts.push(details.processedFrames);
    },
  });

  assertMonotonicProgress(progressValues);
  assert.equal(frameCounts[0], 0);
  assert.equal(frameCounts.at(-1), 1440);
  assert.ok(progressValues.length > 10, "expected incremental progress reports");
  assert.ok(progressValues.length <= 210, "expected throttled progress reports");

  for (let index = 1; index < progressValues.length; index += 1) {
    const previous = progressValues[index - 1]!;
    const current = progressValues[index]!;
    if (current === 1) {
      continue;
    }
    assert.ok(
      current - previous >= 0.005 - Number.EPSILON,
      "expected reports to respect the default progress delta",
    );
  }
});

test("normalization progress can be reported by processed frame records", () => {
  const progressValues: number[] = [];
  const frameCounts: number[] = [];

  normalizeReplayData(buildReplayData(240), {
    progressReportMinDelta: 1,
    progressReportFrameInterval: 300,
    onProgress(progress, details) {
      progressValues.push(progress);
      frameCounts.push(details.processedFrames);
      assert.equal(details.totalFrames, 1440);
    },
  });

  assert.deepEqual(frameCounts, [0, 300, 600, 900, 1200, 1440]);
  assert.equal(progressValues[0], 0);
  assert.equal(progressValues.at(-1), 1);
});

test("normalization progress report cadence is configurable", () => {
  const progressValues: number[] = [];

  normalizeReplayData(buildReplayData(240), {
    progressReportMinDelta: 0.25,
    onProgress(progress) {
      progressValues.push(progress);
    },
  });

  assertMonotonicProgress(progressValues);
  assert.ok(progressValues.length >= 4, "expected phase-sized reports");
  assert.ok(progressValues.length <= 6, "expected sparse progress reports");

  for (let index = 1; index < progressValues.length; index += 1) {
    const previous = progressValues[index - 1]!;
    const current = progressValues[index]!;
    if (current === 1) {
      continue;
    }
    assert.ok(
      current - previous >= 0.25 - Number.EPSILON,
      "expected reports to respect the configured progress delta",
    );
  }
});

test("normalization carries inferred player hitbox metadata", () => {
  const raw = buildReplayData(2, 2);
  raw.meta.team_zero[0]!.stats = {
    Body: { Str: "Dominus" },
  };
  raw.meta.team_one[0]!.stats = {
    LoadoutBody: { Str: "Merc" },
  };

  const replay = normalizeReplayData(raw);

  assert.equal(replay.players[0]!.hitbox.kind, "dominus");
  assert.equal(replay.players[1]!.hitbox.kind, "merc");
});

test("async normalization can yield without a progress callback", async () => {
  let yieldCount = 0;
  const raw = buildReplayData(12);

  const replay = await normalizeReplayDataAsync(raw, {
    yieldEveryMs: 0,
    yieldToMainThread: async () => {
      yieldCount += 1;
    },
  });

  assert.deepEqual(replay, normalizeReplayData(raw));
  assert.ok(yieldCount > 1, "expected async normalization to yield headlessly");
});

test("normalization exposes replay tick marks and timeline bookmark events", () => {
  const raw = buildReplayData(4, 0);
  raw.replay_tick_marks = [
    {
      description: "Team0Goal",
      frame: 2,
      time: 12,
    },
  ];

  const replay = normalizeReplayData(raw);

  assert.deepEqual(replay.tickMarks, [
    {
      id: "bookmark:2:Team0Goal:0",
      description: "Team0Goal",
      frame: 2,
      time: 2,
    },
  ]);
  assert.deepEqual(
    replay.timelineEvents.filter((event) => event.kind === "bookmark"),
    [
      {
        id: "bookmark:2:Team0Goal:0",
        time: 2,
        seekTime: 2,
        frame: 2,
        kind: "bookmark",
        label: "Team0Goal",
        shortLabel: "BM",
        iconName: "bookmark",
      },
    ],
  );
});

test("normalization keeps PlayStation players with duplicate online ids distinct", () => {
  const firstId = {
    PlayStation: {
      online_id: "1",
      name: "Raptor_Attacks_",
      unknown1: [98, 51, 117, 115, 112, 115, 52, 0],
    },
  };
  const secondId = {
    PlayStation: {
      online_id: "1",
      name: "remrocker29",
      unknown1: [97, 51, 117, 115, 112, 115, 52, 0],
    },
  };
  const raw = buildReplayData(2, 0);
  raw.meta.team_zero = [{ remote_id: firstId, stats: null, name: "Raptor_Attacks_" }];
  raw.meta.team_one = [{ remote_id: secondId, stats: null, name: "remrocker29" }];
  raw.frame_data.players = [
    [
      firstId,
      {
        frames: [playerFrame("Raptor_Attacks_", 0), playerFrame("Raptor_Attacks_", 1)],
      },
    ],
    [
      secondId,
      {
        frames: [playerFrame("remrocker29", 0), playerFrame("remrocker29", 1)],
      },
    ],
  ];

  const replay = normalizeReplayData(raw);

  assert.equal(replay.players.length, 2);
  assert.notEqual(replay.players[0]!.id, replay.players[1]!.id);
  assert.deepEqual(
    replay.players.map((player) => player.name),
    ["Raptor_Attacks_", "remrocker29"],
  );
});

test("normalization marks carried player frame gaps as not present", () => {
  const raw = buildReplayData(3, 2);
  raw.frame_data.players[0]![1].frames = [
    playerFrame("Blue 1", 0),
    "Empty",
    playerFrame("Blue 1", 2),
  ];

  const replay = normalizeReplayData(raw);
  const frames = replay.players[0]!.frames;

  assert.equal(frames[0]!.isPresent, true);
  assert.equal(frames[1]!.isPresent, false);
  assert.deepEqual(frames[1]!.position, frames[0]!.position);
  assert.equal(frames[2]!.isPresent, true);
});

test("normalization includes victim location on demo timeline events", () => {
  const raw = buildReplayData(3, 2);
  raw.demolish_infos = [
    {
      time: 10.1,
      seconds_remaining: 299.9,
      frame: 1,
      attacker: { Steam: "blue-player-0" },
      victim: { Steam: "orange-player-0" },
      attacker_velocity: { x: 2300, y: 0, z: 0 },
      victim_velocity: { x: 0, y: 0, z: 0 },
      victim_location: { x: 120, y: -300, z: 17 },
    },
  ];

  const replay = normalizeReplayData(raw);
  const demoEvent = replay.timelineEvents.find((event) => event.kind === "demo");

  assert.equal(demoEvent?.secondaryPlayerId, "Steam:orange-player-0");
  assert.deepEqual(demoEvent?.location, { x: 120, y: -300, z: 17 });
});

test("normalization carries shot metadata onto shot timeline events", () => {
  const raw = buildReplayData(3, 2);
  const shot = {
    ball_position: { x: 10, y: 1200, z: 93 },
    ball_velocity: { x: 0, y: 1800, z: 0 },
    ball_speed: 1800,
    player_position: { x: 10, y: 1000, z: 17 },
    player_velocity: { x: 0, y: 1200, z: 0 },
    player_speed: 1200,
    player_distance_to_ball: 214,
    target_goal_position: { x: 0, y: 5120, z: 93 },
    distance_to_goal_center: 3920,
    distance_to_goal_line: 3920,
    ball_goal_alignment: 1,
    ball_speed_toward_goal: 1800,
  };
  raw.player_stat_events = [
    {
      time: 10.1,
      frame: 1,
      player: { Steam: "blue-player-0" },
      is_team_0: true,
      kind: "Shot",
      shot,
    },
  ];

  const replay = normalizeReplayData(raw);
  const shotEvent = replay.timelineEvents.find((event) => event.kind === "shot");

  assert.deepEqual(shotEvent?.location, shot.ball_position);
  assert.deepEqual(shotEvent?.shot, shot);
});

test("async normalization yields at configured frame progress intervals", async () => {
  let yieldCount = 0;
  const raw = buildReplayData(24);

  const replay = await normalizeReplayDataAsync(raw, {
    progressReportMinDelta: 1,
    progressReportFrameInterval: 24,
    yieldEveryMs: Number.POSITIVE_INFINITY,
    onProgress() {},
    yieldToMainThread: async () => {
      yieldCount += 1;
    },
  });

  assert.deepEqual(replay, normalizeReplayData(raw));
  assert.ok(
    yieldCount >= 5,
    "expected frame progress reports to give the browser paint opportunities",
  );
});

test("async normalization yields when percent progress reports are emitted", async () => {
  let yieldCount = 0;
  const raw = buildReplayData(240);

  const replay = await normalizeReplayDataAsync(raw, {
    progressReportMinDelta: 0.25,
    yieldEveryMs: Number.POSITIVE_INFINITY,
    onProgress() {},
    yieldToMainThread: async () => {
      yieldCount += 1;
    },
  });

  assert.deepEqual(replay, normalizeReplayData(raw));
  assert.ok(yieldCount >= 3, "expected progress reports to give the browser paint opportunities");
});

test("replay loading text shows incremental normalization substeps", () => {
  assert.equal(
    formatReplayLoadProgress({ stage: "normalizing", progress: 0.72 }),
    "Normalizing replay data... 72%",
  );
  assert.equal(
    formatReplayLoadProgressMeta({ stage: "normalizing", progress: 0.1 }),
    "Decoding structured replay data",
  );
  assert.equal(
    formatReplayLoadProgressMeta({ stage: "normalizing", progress: 0.5 }),
    "Parsing frame data",
  );
  assert.equal(
    formatReplayLoadProgressMeta({ stage: "normalizing", progress: 0.72 }),
    "Building playback model",
  );
  assert.equal(
    formatReplayLoadProgressMeta({ stage: "normalizing", progress: 1 }),
    "Playback model ready",
  );
});
