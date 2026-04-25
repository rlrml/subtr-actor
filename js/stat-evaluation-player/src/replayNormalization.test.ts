import test from "node:test";
import assert from "node:assert/strict";

import {
  normalizeReplayData,
  normalizeReplayDataAsync,
} from "subtr-actor-player";
import type { RawReplayFramesData } from "subtr-actor-player";

function rigidBody(x: number, y: number, z: number) {
  return {
    sleeping: false,
    location: { x, y, z },
    rotation: { x: 0, y: 0, z: 0, w: 1 },
    linear_velocity: { x: x * 10, y: y * 10, z: z * 10 },
    angular_velocity: { x: 0, y: 0, z: 0 },
  };
}

function playerFrame(playerName: string, x: number) {
  return {
    Data: {
      rigid_body: rigidBody(x, 0, 17),
      boost_amount: 128,
      boost_active: false,
      powerslide_active: false,
      jump_active: false,
      double_jump_active: false,
      dodge_active: false,
      player_name: playerName,
      team: playerName === "Blue" ? 0 : 1,
      is_team_0: playerName === "Blue",
    },
  };
}

function ballFrame(x: number) {
  return {
    Data: {
      rigid_body: rigidBody(x, 50, 93),
    },
  };
}

const replayData: RawReplayFramesData = {
  meta: {
    team_zero: [
      {
        remote_id: { Steam: "blue-player" },
        stats: null,
        name: "Blue",
      },
    ],
    team_one: [
      {
        remote_id: { Steam: "orange-player" },
        stats: null,
        name: "Orange",
      },
    ],
    all_headers: [],
  },
  frame_data: {
    metadata_frames: [
      {
        time: 10,
        seconds_remaining: 300,
        replicated_game_state_name: 0,
        replicated_game_state_time_remaining: 0,
      },
      {
        time: 10.1,
        seconds_remaining: 299.9,
        replicated_game_state_name: 0,
        replicated_game_state_time_remaining: 0,
      },
      {
        time: 10.2,
        seconds_remaining: 299.8,
        replicated_game_state_name: 0,
        replicated_game_state_time_remaining: 0,
      },
    ],
    ball_data: {
      frames: [ballFrame(0), ballFrame(100), ballFrame(200)],
    },
    players: [
      [
        { Steam: "blue-player" },
        {
          frames: [
            playerFrame("Blue", 0),
            playerFrame("Blue", 10),
            playerFrame("Blue", 20),
          ],
        },
      ],
      [
        { Steam: "orange-player" },
        {
          frames: [
            playerFrame("Orange", 0),
            playerFrame("Orange", -10),
            playerFrame("Orange", -20),
          ],
        },
      ],
    ],
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

test("async replay normalization matches sync output and can yield progress", async () => {
  const progressValues: number[] = [];
  let yieldCount = 0;

  const asyncReplay = await normalizeReplayDataAsync(replayData, {
    yieldEveryMs: 0,
    yieldToMainThread: async () => {
      yieldCount += 1;
    },
    onProgress(progress) {
      progressValues.push(progress);
    },
  });

  assert.deepEqual(asyncReplay, normalizeReplayData(replayData));
  assert.ok(yieldCount > 1, "expected yielded normalization work");
  assert.ok(progressValues.length > 1, "expected incremental progress reports");
  assert.equal(progressValues.at(-1), 1);

  for (let index = 1; index < progressValues.length; index += 1) {
    assert.ok(
      progressValues[index]! >= progressValues[index - 1]!,
      "expected monotonic progress",
    );
  }
});
