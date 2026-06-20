import test from "node:test";
import assert from "node:assert/strict";

import {
  computeTimelineSegments,
  getReplayPlaybackEndTime,
  projectReplayTimeToTimeline,
  projectTimelineTimeToReplay,
} from "../src/player-internals/timeline";
import { getPostGoalTransitionSkipTargetTime } from "../src/player-helpers";
import type { ReplayModel } from "../src/types";

function replayWithGameStates(
  gameStates: number[],
  timelineEvents: ReplayModel["timelineEvents"] = [],
): ReplayModel {
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
    timelineEvents,
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

test("timeline projection keeps replay times canonical after skipped ranges", () => {
  const segments = [{ startTime: 2, endTime: 4 }];

  assert.deepEqual(projectReplayTimeToTimeline(10, segments, 7), {
    replayTime: 7,
    timelineTime: 7,
    seekTime: 7,
    hiddenBySkip: false,
  });
  assert.equal(projectTimelineTimeToReplay(10, 10, segments, 7), 7);
});

test("timeline projection identifies skipped ranges without compacting them", () => {
  assert.deepEqual(projectReplayTimeToTimeline(10, [{ startTime: 2, endTime: 4 }], 3), {
    replayTime: 3,
    timelineTime: 3,
    seekTime: 4,
    hiddenBySkip: true,
  });
});

test("post-goal skip preserves the goal explosion window", () => {
  const replay = replayWithGameStates(
    [0, 0, 1, 1, 1, 1, 0, 0],
    [
      {
        kind: "goal",
        time: 2,
        frame: 2,
        playerId: null,
        playerName: null,
        isTeamZero: true,
      },
    ],
  );

  assert.deepEqual(computeTimelineSegments(replay, true, false, 0, null), [
    { startTime: 4, endTime: 6 },
  ]);
  assert.equal(getPostGoalTransitionSkipTargetTime(replay, 2, 0, null), null);
  assert.equal(getPostGoalTransitionSkipTargetTime(replay, 3, 0, null), null);
  assert.equal(getPostGoalTransitionSkipTargetTime(replay, 4, 0, null), 6);
});
