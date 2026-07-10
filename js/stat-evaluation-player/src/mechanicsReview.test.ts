import test from "node:test";
import assert from "node:assert/strict";

import {
  getMechanicsReviewDecisionForKey,
  isReviewLabelsEndpoint,
  parseMechanicsReviewPlaylist,
  resolveMechanicsReviewPerspectivePlayerTrack,
  resolveMechanicsReviewBoundTime,
  resolveMechanicsReviewTargetTime,
  type MechanicsReviewItem,
  type MechanicsReviewTimingReplay,
} from "./mechanicsReview.ts";
import type { ReplayPlayerTrack } from "@rlrml/player";

function replayWithFrame(frameIndex: number, time: number): MechanicsReviewTimingReplay {
  const frames = Array.from({ length: frameIndex + 1 }, (_, index) => ({ time: index / 30 }));
  frames[frameIndex] = { time };
  return {
    duration: 120,
    rawStartTime: 37,
    frames,
  };
}

test("raw replay review playlist times use the replay normalization offset", () => {
  const replay = replayWithFrame(3000, 63);
  const item: MechanicsReviewItem = {
    replay: "replay-id",
    start: { kind: "time", value: 96 },
    end: { kind: "time", value: 104 },
    meta: {
      target: {
        eventTime: 100,
      },
    },
  };

  assert.equal(resolveMechanicsReviewBoundTime(item, item.start, replay, "rawReplay"), 59);
  assert.equal(resolveMechanicsReviewBoundTime(item, item.end, replay, "rawReplay"), 67);
  assert.equal(resolveMechanicsReviewTargetTime(item, replay, "rawReplay"), 63);
});

test("legacy review playlist times are shifted from raw replay time into player playback time", () => {
  const replay = replayWithFrame(3000, 63);
  const item: MechanicsReviewItem = {
    replay: "replay-id",
    start: { kind: "time", value: 96 },
    end: { kind: "time", value: 104 },
    meta: {
      target: {
        eventTime: 100,
        eventFrame: 3000,
      },
    },
  };

  assert.equal(resolveMechanicsReviewBoundTime(item, item.start, replay), 59);
  assert.equal(resolveMechanicsReviewBoundTime(item, item.end, replay), 67);
  assert.equal(resolveMechanicsReviewTargetTime(item, replay), 63);
});

test("review playlist times are unchanged when they already match player playback time", () => {
  const replay = replayWithFrame(3000, 100);
  const item: MechanicsReviewItem = {
    replay: "replay-id",
    start: { kind: "time", value: 96 },
    end: { kind: "time", value: 104 },
    meta: {
      target: {
        eventTime: 100,
        eventFrame: 3000,
      },
    },
  };

  assert.equal(resolveMechanicsReviewBoundTime(item, item.start, replay, "playback"), 96);
  assert.equal(resolveMechanicsReviewBoundTime(item, item.end, replay, "playback"), 104);
  assert.equal(resolveMechanicsReviewTargetTime(item, replay, "playback"), 100);
});

test("review playlist frame bounds use replay frame playback time directly", () => {
  const replay = replayWithFrame(3000, 63);
  const item: MechanicsReviewItem = {
    replay: "replay-id",
    start: { kind: "frame", value: 3000 },
    end: { kind: "frame", value: 3000 },
    meta: {
      target: {
        eventTime: 100,
        eventFrame: 3000,
      },
    },
  };

  assert.equal(resolveMechanicsReviewBoundTime(item, item.start, replay), 63);
});

test("review playlists preserve optional clip perspective", () => {
  const playlist = parseMechanicsReviewPlaylist({
    items: [
      {
        replay: "replay-id",
        start: { kind: "time", value: 1 },
        end: { kind: "time", value: 2 },
        perspective: {
          kind: "player",
          playerId: "Steam:76561198000000000",
          playerName: "Scorer",
          ballCam: "player",
          usePlayerCameraSettings: true,
        },
      },
    ],
  });

  assert.deepEqual(playlist.items[0]?.perspective, {
    kind: "player",
    playerId: "Steam:76561198000000000",
    playerName: "Scorer",
    ballCam: "player",
    usePlayerCameraSettings: true,
  });
});

test("review playlist perspectives reject invalid ball cam modes", () => {
  assert.throws(
    () =>
      parseMechanicsReviewPlaylist({
        items: [
          {
            replay: "replay-id",
            start: { kind: "time", value: 1 },
            end: { kind: "time", value: 2 },
            perspective: {
              kind: "player",
              playerName: "Scorer",
              ballCam: "sometimes",
            },
          },
        ],
      }),
    /perspective ballCam/,
  );
});

test("review clip perspectives match exact replay track ids", () => {
  const track = playerTrack("Steam:76561198000000000", "Scorer");
  assert.equal(
    resolveMechanicsReviewPerspectivePlayerTrack(
      { kind: "player", playerId: "Steam:76561198000000000" },
      [track],
    )?.id,
    track.id,
  );
});

test("review clip perspectives fall back to player name when ids differ", () => {
  const track = playerTrack("replay-track-id", "Known Alias");
  assert.equal(
    resolveMechanicsReviewPerspectivePlayerTrack(
      {
        kind: "player",
        playerId: "source-event-id",
        playerName: "Known Alias",
      },
      [track],
    )?.id,
    track.id,
  );
});

test("single-keystroke labels map to review decisions", () => {
  assert.equal(getMechanicsReviewDecisionForKey("y"), "confirmed");
  assert.equal(getMechanicsReviewDecisionForKey("1"), "confirmed");
  assert.equal(getMechanicsReviewDecisionForKey("n"), "rejected");
  assert.equal(getMechanicsReviewDecisionForKey("2"), "rejected");
  assert.equal(getMechanicsReviewDecisionForKey("u"), "uncertain");
  assert.equal(getMechanicsReviewDecisionForKey("3"), "uncertain");
  assert.equal(getMechanicsReviewDecisionForKey("b"), "bad_candidate");
  assert.equal(getMechanicsReviewDecisionForKey("4"), "bad_candidate");
  assert.equal(getMechanicsReviewDecisionForKey("Y"), "confirmed");
  assert.equal(getMechanicsReviewDecisionForKey(" "), null);
  assert.equal(getMechanicsReviewDecisionForKey("r"), null);
  assert.equal(getMechanicsReviewDecisionForKey("ArrowRight"), null);
});

test("review-labels endpoints are detected by pathname", () => {
  assert.equal(isReviewLabelsEndpoint("/review-labels/flicks"), true);
  assert.equal(isReviewLabelsEndpoint("/review-labels/flicks?candidate=3&replay=abc"), true);
  assert.equal(isReviewLabelsEndpoint("http://localhost:5173/review-labels/flicks"), true);
  assert.equal(isReviewLabelsEndpoint("/api/v1/events/abc/reviews"), false);
  assert.equal(isReviewLabelsEndpoint("https://rocket-sense.example/api/reviews"), false);
});

function playerTrack(id: string, name: string): ReplayPlayerTrack {
  return {
    id,
    name,
    isTeamZero: true,
    cameraSettings: {},
    frames: [],
  } as unknown as ReplayPlayerTrack;
}
