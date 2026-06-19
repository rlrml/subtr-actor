import assert from "node:assert/strict";
import { test } from "node:test";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import {
  buildMissedEventReviewPayload,
  captureMissedEventFromPlayer,
  resolveCaptureReplayId,
  type MissedEventCaptureRecord,
} from "./missedEventCapture.ts";

const REPLAY_UUID = "0196f449-e997-7413-af77-28082e6478f0";

function record(overrides: Partial<MissedEventCaptureRecord> = {}): MissedEventCaptureRecord {
  return {
    localId: "missed-1",
    mechanic: "flick",
    frame: 1187,
    time: 52.89,
    subjectKind: "player",
    subjectId: "steam:76561198298819443",
    playerName: "strangy",
    startFrame: null,
    endFrame: null,
    notes: null,
    confidence: 1,
    replayId: REPLAY_UUID,
    context: {},
    ...overrides,
  };
}

test("resolveCaptureReplayId prefers the replayId query param", () => {
  assert.equal(resolveCaptureReplayId(`?replayId=${REPLAY_UUID}`, "fallback"), REPLAY_UUID);
  assert.equal(resolveCaptureReplayId(`?replay-id=${REPLAY_UUID}`), REPLAY_UUID);
});

test("resolveCaptureReplayId falls back then yields null", () => {
  assert.equal(resolveCaptureReplayId("", "fallback-id"), "fallback-id");
  assert.equal(resolveCaptureReplayId(""), null);
  assert.equal(resolveCaptureReplayId("?other=1", "  "), null);
});

test("buildMissedEventReviewPayload returns null without a replay id", () => {
  assert.equal(buildMissedEventReviewPayload(record({ replayId: null })), null);
});

test("buildMissedEventReviewPayload includes subject, span, and confirmed status", () => {
  const payload = buildMissedEventReviewPayload(
    record({ startFrame: 1182, endFrame: 1190, notes: " missed " }),
  );
  assert.ok(payload);
  assert.equal(payload.replay_id, REPLAY_UUID);
  assert.equal(payload.reviewed_mechanic, "flick");
  assert.equal(payload.reviewed_event_frame, 1187);
  assert.equal(payload.reviewed_subject_kind, "player");
  assert.equal(payload.reviewed_subject_id, "steam:76561198298819443");
  assert.equal(payload.reviewed_start_frame, 1182);
  assert.equal(payload.reviewed_end_frame, 1190);
  assert.equal(payload.reviewed_event_time, 52.89);
  assert.equal(payload.status, "confirmed");
  assert.equal(payload.notes, "missed");
});

test("buildMissedEventReviewPayload omits subject when there is no attached player", () => {
  const payload = buildMissedEventReviewPayload(
    record({ subjectKind: null, subjectId: null, playerName: null }),
  );
  assert.ok(payload);
  assert.equal(payload.reviewed_subject_kind, undefined);
  assert.equal(payload.reviewed_subject_id, undefined);
});

test("captureMissedEventFromPlayer snapshots frame, time, and attached subject", () => {
  const player = {
    getState: () => ({
      frameIndex: 1187.4,
      currentTime: 52.89,
      attachedPlayerId: "steam:76561198298819443",
    }),
    replay: {
      duration: 412.7,
      players: [{ id: "steam:76561198298819443", name: "strangy" }],
    },
  } as unknown as StatsReplayPlayer;

  const captured = captureMissedEventFromPlayer(player, {
    mechanic: "flick",
    replayId: REPLAY_UUID,
    localId: "missed-7",
  });

  assert.equal(captured.frame, 1187);
  assert.equal(captured.time, 52.89);
  assert.equal(captured.subjectKind, "player");
  assert.equal(captured.subjectId, "steam:76561198298819443");
  assert.equal(captured.playerName, "strangy");
  assert.equal(captured.confidence, 1);
  assert.equal(captured.replayId, REPLAY_UUID);
  assert.equal(captured.context.durationSeconds, 412.7);
});

test("captureMissedEventFromPlayer tolerates no attached player", () => {
  const player = {
    getState: () => ({ frameIndex: 10, currentTime: 1.2, attachedPlayerId: null }),
    replay: { duration: 100, players: [] },
  } as unknown as StatsReplayPlayer;

  const captured = captureMissedEventFromPlayer(player, {
    mechanic: "whiff",
    replayId: null,
    localId: "missed-8",
  });

  assert.equal(captured.subjectKind, null);
  assert.equal(captured.subjectId, null);
  assert.equal(captured.playerName, null);
  assert.equal(captured.replayId, null);
});
