import assert from "node:assert/strict";
import test from "node:test";
import {
  AutoPossessionCameraController,
  buildAutoPossessionCameraSpans,
  selectAutoPossessionCameraPlayer,
} from "./autoPossessionCamera.ts";
import type { StatsEventPayload } from "./statsTimeline.ts";
import { createLegacyStatsTimeline } from "./testStatsTimeline.ts";

const blue = { Steam: "blue" };
const orange = { Steam: "orange" };

function playerPossession(
  player_id: StatsEventPayload<"player_possession">["player_id"],
  start_time: number,
  end_time: number,
  overrides: Partial<StatsEventPayload<"player_possession">> = {},
): StatsEventPayload<"player_possession"> {
  return {
    player_id,
    is_team_0: player_id === blue,
    start_frame: Math.round(start_time * 30),
    end_frame: Math.round(end_time * 30),
    start_time,
    end_time,
    duration: end_time - start_time,
    touch_count: 1,
    aerial_touch_count: 0,
    wall_touch_count: 0,
    advance_distance: 0,
    retreat_distance: 0,
    carry_time: 0,
    air_dribble_time: 0,
    carry_count: 0,
    air_dribble_count: 0,
    close_time: end_time - start_time,
    sustained_control: false,
    start_field_third: null,
    end_field_third: null,
    ...overrides,
  };
}

test("auto follow starts before the first possession span", () => {
  const timeline = createLegacyStatsTimeline({
    player_possession: [playerPossession(blue, 2, 4)],
  });

  const spans = buildAutoPossessionCameraSpans(timeline, { preRollSeconds: 0.75 });

  assert.equal(spans.length, 1);
  assert.equal(spans[0]?.playerId, "Steam:blue");
  assert.equal(spans[0]?.startFrame, 37);
  assert.equal(selectAutoPossessionCameraPlayer(spans, 45), "Steam:blue");
});

test("auto follow pre-roll does not cut away from active possession", () => {
  const timeline = createLegacyStatsTimeline({
    player_possession: [playerPossession(blue, 1, 4), playerPossession(orange, 4.2, 6)],
  });

  const spans = buildAutoPossessionCameraSpans(timeline, { preRollSeconds: 1 });

  assert.equal(spans[0]?.playerId, "Steam:blue");
  assert.equal(spans[0]?.endFrame, 120);
  assert.equal(spans[1]?.playerId, "Steam:orange");
  assert.equal(spans[1]?.startFrame, 120);
});

test("auto follow ignores tiny possession noise unless it has sustained control", () => {
  const timeline = createLegacyStatsTimeline({
    player_possession: [
      playerPossession(blue, 1, 1.2),
      playerPossession(orange, 2, 2.2, { sustained_control: true }),
    ],
  });

  const spans = buildAutoPossessionCameraSpans(timeline, { minPossessionSeconds: 0.45 });

  assert.deepEqual(
    spans.map((span) => span.playerId),
    ["Steam:orange"],
  );
});

test("auto follow bridges same-player possession fragments", () => {
  const timeline = createLegacyStatsTimeline({
    player_possession: [
      playerPossession(blue, 1, 2),
      playerPossession(blue, 2.3, 3),
      playerPossession(orange, 5, 6),
    ],
  });

  const spans = buildAutoPossessionCameraSpans(timeline, {
    preRollSeconds: 0.2,
    samePlayerBridgeSeconds: 0.5,
  });

  assert.equal(spans.length, 2);
  assert.equal(spans[0]?.playerId, "Steam:blue");
  assert.equal(spans[0]?.possessionEndFrame, 90);
  assert.equal(spans[1]?.playerId, "Steam:orange");
});

test("auto follow uses frame index rather than normalized playback time", () => {
  const timeline = createLegacyStatsTimeline({
    player_possession: [playerPossession(blue, 102, 104)],
  });

  const spans = buildAutoPossessionCameraSpans(timeline, { preRollSeconds: 0.5 });

  assert.equal(selectAutoPossessionCameraPlayer(spans, 3050), "Steam:blue");
  assert.equal(selectAutoPossessionCameraPlayer(spans, 45), null);
});

test("auto follow applies the selected player while preserving the enabled toggle", () => {
  const replay = {
    players: [{ id: "Steam:blue" }, { id: "Steam:orange" }],
  };
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
  (controller as unknown as { spans: ReturnType<typeof buildAutoPossessionCameraSpans> }).spans = [
    {
      playerId: "Steam:orange",
      startFrame: 10,
      endFrame: Number.POSITIVE_INFINITY,
      possessionStartFrame: 20,
      possessionEndFrame: 40,
    },
  ];

  controller.update({ frameIndex: 10, nextFrameIndex: 10, alpha: 0, currentTime: 0 });

  assert.deepEqual(followedPlayers, ["Steam:orange"]);
  assert.deepEqual(
    followOptions.map((options) =>
      typeof options === "object" && options !== null
        ? (options as { preserveAutoPossession?: boolean }).preserveAutoPossession
        : null,
    ),
    [true],
  );
});
