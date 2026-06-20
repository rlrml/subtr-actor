import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "@rlrml/player";
import {
  AutoPossessionCameraController,
  buildAutoPossessionCameraSpans,
  selectAutoPossessionCameraPlayer,
  type AutoPossessionSpan,
} from "./autoPossessionCamera.ts";
import type { Event } from "./statsTimeline.ts";
import { createStatsTimeline } from "./testStatsTimeline.ts";

test("auto possession camera spans come from player possession events", () => {
  const timeline = createStatsTimeline({
    events: {
      events: [
        {
          meta: {
            id: "player_possession:10:20:0",
            stream: "player_possession",
            label: "Player Possession",
            scope: "player",
            timing: {
              type: "span",
              start_time: 1,
              start_frame: 10,
              end_time: 2,
              end_frame: 20,
            },
            primary_player: { Steam: "blue-id" },
            properties: [],
          },
          payload: {
            kind: "player_possession",
            payload: {
              player_id: { Steam: "blue-id" },
              is_team_0: true,
              start_frame: 10,
              end_frame: 20,
              start_time: 1,
              end_time: 2,
              duration: 1,
              touch_count: 2,
              aerial_touch_count: 0,
              wall_touch_count: 0,
              advance_distance: 200,
              retreat_distance: 0,
              carry_time: 0,
              air_dribble_time: 0,
              carry_count: 0,
              air_dribble_count: 0,
              close_time: 0.8,
              sustained_control: true,
              start_field_third: "neutral_third",
              end_field_third: "team_one_third",
            },
          },
        } satisfies Event,
        {
          meta: {
            id: "possession:11:21:0",
            stream: "possession",
            label: "Possession",
            scope: "team",
            timing: {
              type: "span",
              start_time: 1.1,
              start_frame: 11,
              end_time: 2.1,
              end_frame: 21,
            },
            properties: [],
          },
          payload: {
            kind: "possession",
            payload: {
              time: 1.1,
              frame: 11,
              end_time: 2.1,
              end_frame: 21,
              active: true,
              duration: 1,
              possession_state: "team_zero",
              player_id: { Steam: "blue-id" },
            },
          },
        } satisfies Event,
      ],
    },
  });

  assert.deepEqual(buildAutoPossessionCameraSpans(timeline), [
    {
      playerId: "Steam:blue-id",
      startFrame: 10,
      endFrame: 20,
      startTime: 1,
      endTime: 2,
    },
  ]);
});

test("auto possession camera selects possessed player before closest player fallback", () => {
  const replay = {
    ballFrames: [{ position: { x: 100, y: 0, z: 0 } }],
    players: [
      {
        id: "Steam:blue-id",
        frames: [{ position: { x: 1000, y: 0, z: 0 } }],
      },
      {
        id: "Steam:orange-id",
        frames: [{ position: { x: 120, y: 0, z: 0 } }],
      },
    ],
  } as ReplayModel;
  const spans: AutoPossessionSpan[] = [
    {
      playerId: "Steam:blue-id",
      startFrame: 0,
      endFrame: 5,
      startTime: 0,
      endTime: 0.5,
    },
  ];

  assert.equal(selectAutoPossessionCameraPlayer(replay, spans, 0, 0.25), "Steam:blue-id");
  assert.equal(selectAutoPossessionCameraPlayer(replay, spans, 0, 1), "Steam:orange-id");
});

function createFallbackReplay(): ReplayModel {
  return {
    ballFrames: Array.from({ length: 5 }, () => ({ position: { x: 0, y: 0, z: 0 } })),
    players: [
      {
        id: "Steam:blue-id",
        frames: [
          { position: { x: 100, y: 0, z: 0 } },
          { position: { x: 500, y: 0, z: 0 } },
          { position: { x: 500, y: 0, z: 0 } },
          { position: { x: 500, y: 0, z: 0 } },
          { position: { x: 500, y: 0, z: 0 } },
        ],
      },
      {
        id: "Steam:orange-id",
        frames: [
          { position: { x: 500, y: 0, z: 0 } },
          { position: { x: 100, y: 0, z: 0 } },
          { position: { x: 100, y: 0, z: 0 } },
          { position: { x: 100, y: 0, z: 0 } },
          { position: { x: 100, y: 0, z: 0 } },
        ],
      },
    ],
  } as ReplayModel;
}

test("auto possession camera keeps nearest-player fallback fixed after initial attach", () => {
  const replay = createFallbackReplay();
  const followedPlayers: string[] = [];
  let attachedPlayerId: string | null = null;
  const replayPlayer = {
    replay,
    getState() {
      return {
        frameIndex: 0,
        currentTime: 0,
        cameraViewMode: attachedPlayerId === null ? "free" : "follow",
        attachedPlayerId,
      };
    },
  };
  const cameraControls = {
    autoPossessionEnabled: true,
    followPlayerWithReplayCamera(playerId: string) {
      attachedPlayerId = playerId;
      followedPlayers.push(playerId);
    },
  };
  const controller = new AutoPossessionCameraController({
    getReplayPlayer: () => replayPlayer as never,
    getStatsTimeline: () => null,
    getCameraControlsController: () => cameraControls as never,
  });

  controller.update({ frameIndex: 0, nextFrameIndex: 0, alpha: 0, currentTime: 0 });
  controller.update({ frameIndex: 1, nextFrameIndex: 1, alpha: 0, currentTime: 0.7 });
  controller.update({ frameIndex: 2, nextFrameIndex: 2, alpha: 0, currentTime: 1.1 });
  controller.update({ frameIndex: 3, nextFrameIndex: 3, alpha: 0, currentTime: 1.3 });
  controller.update({ frameIndex: 4, nextFrameIndex: 4, alpha: 0, currentTime: 1.6 });

  assert.deepEqual(followedPlayers, ["Steam:blue-id"]);
});

test("auto possession camera switches after stable possession", () => {
  const replay = createFallbackReplay();
  const possessionSpans: AutoPossessionSpan[] = [
    {
      playerId: "Steam:orange-id",
      startFrame: 1,
      endFrame: 4,
      startTime: 0.7,
      endTime: 2,
    },
  ];
  const followedPlayers: string[] = [];
  const followOptions: unknown[] = [];
  let attachedPlayerId: string | null = null;
  const replayPlayer = {
    replay,
    getState() {
      return {
        frameIndex: 0,
        currentTime: 0,
        cameraViewMode: attachedPlayerId === null ? "free" : "follow",
        attachedPlayerId,
      };
    },
  };
  const cameraControls = {
    autoPossessionEnabled: true,
    followPlayerWithReplayCamera(playerId: string, options: unknown) {
      attachedPlayerId = playerId;
      followedPlayers.push(playerId);
      followOptions.push(options);
    },
  };
  const controller = new AutoPossessionCameraController({
    getReplayPlayer: () => replayPlayer as never,
    getStatsTimeline: () => null,
    getCameraControlsController: () => cameraControls as never,
  });
  (controller as unknown as { spans: AutoPossessionSpan[] }).spans = possessionSpans;

  controller.update({ frameIndex: 0, nextFrameIndex: 0, alpha: 0, currentTime: 0 });
  controller.update({ frameIndex: 1, nextFrameIndex: 1, alpha: 0, currentTime: 0.7 });
  controller.update({ frameIndex: 2, nextFrameIndex: 2, alpha: 0, currentTime: 1.1 });
  controller.update({ frameIndex: 3, nextFrameIndex: 3, alpha: 0, currentTime: 1.3 });
  controller.update({ frameIndex: 4, nextFrameIndex: 4, alpha: 0, currentTime: 1.6 });

  assert.deepEqual(followedPlayers, ["Steam:blue-id", "Steam:orange-id"]);
  assert.deepEqual(
    followOptions.map((options) =>
      typeof options === "object" && options !== null
        ? (options as { usePlayerCameraSettings?: boolean }).usePlayerCameraSettings
        : null,
    ),
    [false, false],
  );
});

test("auto possession camera uses current followed player as fixed fallback", () => {
  const replay = {
    ballFrames: [{ position: { x: 0, y: 0, z: 0 } }],
    players: [
      {
        id: "Steam:blue-id",
        frames: [{ position: { x: 500, y: 0, z: 0 } }],
      },
      {
        id: "Steam:orange-id",
        frames: [{ position: { x: 100, y: 0, z: 0 } }],
      },
    ],
  } as ReplayModel;
  const followedPlayers: string[] = [];
  let attachedPlayerId: string | null = "Steam:blue-id";
  const replayPlayer = {
    replay,
    getState() {
      return {
        frameIndex: 0,
        currentTime: 0,
        cameraViewMode: "follow",
        attachedPlayerId,
      };
    },
  };
  const cameraControls = {
    autoPossessionEnabled: true,
    followPlayerWithReplayCamera(playerId: string) {
      attachedPlayerId = playerId;
      followedPlayers.push(playerId);
    },
  };
  const controller = new AutoPossessionCameraController({
    getReplayPlayer: () => replayPlayer as never,
    getStatsTimeline: () => null,
    getCameraControlsController: () => cameraControls as never,
  });

  controller.update({ frameIndex: 0, nextFrameIndex: 0, alpha: 0, currentTime: 0 });

  assert.deepEqual(followedPlayers, []);
  assert.equal(attachedPlayerId, "Steam:blue-id");
});
