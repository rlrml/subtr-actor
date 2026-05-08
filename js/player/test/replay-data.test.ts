import test from "node:test";
import assert from "node:assert/strict";

import {
  normalizeReplayData,
  normalizeReplayDataAsync,
} from "../src/replay-data";
import {
  formatReplayLoadProgress,
  formatReplayLoadProgressMeta,
} from "../src/load-ui";
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
  const players: RawReplayFramesData["frame_data"]["players"] = [
    ...teamZero,
    ...teamOne,
  ].map((player): RawReplayFramesData["frame_data"]["players"][number] => [
    player.remote_id,
    {
      frames: Array.from({ length: frameCount }, (_, index) =>
        playerFrame(player.name, index),
      ),
    },
  ]);

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
    touch_events: [],
    dodge_refreshed_events: [],
    player_stat_events: [],
    goal_events: [],
    heuristic_data: {
      flip_reset_events: [],
      post_wall_dodge_events: [],
      flip_reset_followup_dodge_events: [],
    },
  };
}

function assertMonotonicProgress(progressValues: number[]): void {
  assert.equal(progressValues[0], 0);
  assert.equal(progressValues.at(-1), 1);

  for (let index = 1; index < progressValues.length; index += 1) {
    assert.ok(
      progressValues[index]! >= progressValues[index - 1]!,
      "expected monotonic progress",
    );
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
