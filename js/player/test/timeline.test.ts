import test from "node:test";
import assert from "node:assert/strict";

import {
  computeTimelineSegments,
  getReplayPlaybackEndTime,
  projectTimelineTimeToReplay,
} from "../src/player-internals/timeline";
import type { ReplayModel } from "../src/types";

function replayWithGameStates(gameStates: number[]): ReplayModel {
  return {
    frameCount: gameStates.length,
    duration: Math.max(0, gameStates.length - 1),
    frames: gameStates.map((gameState, index) => ({
      time: index,
      secondsRemaining: 300 - index,
      gameState,
      kickoffCountdown: 0,
    })),
    ballFrames: gameStates.map((_, index) => ({
      position: { x: index, y: 0, z: 92 },
      linearVelocity: null,
      angularVelocity: null,
      rotation: null,
    })),
    boostPads: [],
    players: [],
    tickMarks: [],
    timelineEvents: [],
    teamZeroNames: [],
    teamOneNames: [],
  };
}

test("playback end truncates a final skipped post-goal segment", () => {
  const replay = replayWithGameStates([0, 0, 0, 1, 1, 1]);
  const segments = computeTimelineSegments(replay, true, false, 0, null);

  assert.deepEqual(segments, [{ startTime: 3, endTime: 5 }]);
  assert.equal(getReplayPlaybackEndTime(replay.duration, segments), 3);
  assert.equal(projectTimelineTimeToReplay(replay.duration, 3, segments, 3), 3);
});

test("playback end keeps raw duration when final frames are visible", () => {
  const replay = replayWithGameStates([0, 0, 1, 1, 0, 0]);
  const segments = computeTimelineSegments(replay, true, false, 0, null);

  assert.deepEqual(segments, [{ startTime: 2, endTime: 4 }]);
  assert.equal(getReplayPlaybackEndTime(replay.duration, segments), 5);
});
