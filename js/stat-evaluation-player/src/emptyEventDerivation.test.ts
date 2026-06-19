import test from "node:test";
import assert from "node:assert/strict";

import { applyStatsTimelineEventDerivedStats } from "./statsTimelineDerivation.ts";
import { createStatsFrame, createStatsTimeline } from "./testStatsTimeline.ts";

const playerId = { Steam: "empty-event-player" } as Record<string, unknown>;

function deleteField(object: object, field: string): void {
  delete (object as Record<string, unknown>)[field];
}

test("empty event derivations restore default fields for compacted zero-event timelines", () => {
  const timeline = createStatsTimeline({
    frames: [
      createStatsFrame({
        frame_number: 10,
        time: 1,
        players: [{ player_id: playerId, is_team_0: true, name: "Blue" }],
      }),
    ],
  });
  const frame = timeline.frames[0]!;
  const player = frame.players[0]!;

  deleteField(frame.team_zero.core, "goals");
  deleteField(frame.team_zero.possession, "tracked_time");
  deleteField(frame.team_zero.ball_half, "tracked_time");
  deleteField(frame.team_zero.rotation, "rotation_count");
  deleteField(frame.team_zero.rush, "count");
  deleteField(frame.team_zero.backboard, "count");
  deleteField(frame.team_zero.double_tap, "count");
  deleteField(frame.team_zero.one_timer, "count");
  deleteField(frame.team_zero.pass, "completed_pass_count");
  deleteField(frame.team_zero.ball_carry, "carry_count");
  deleteField(frame.team_zero.air_dribble, "count");
  deleteField(frame.team_zero.boost, "amount_used");
  deleteField(frame.team_zero.bump, "bumps_inflicted");
  deleteField(frame.team_zero.half_volley, "count");
  deleteField(frame.team_zero.movement, "tracked_time");
  deleteField(frame.team_zero.powerslide, "total_duration");
  deleteField(frame.team_zero.demo, "demos_inflicted");
  deleteField(frame.team_zero.fifty_fifty, "count");

  deleteField(player.core, "goals");
  deleteField(player.backboard, "count");
  deleteField(player.ceiling_shot, "count");
  deleteField(player.wall_aerial, "count");
  deleteField(player.wall_aerial_shot, "count");
  deleteField(player.double_tap, "count");
  deleteField(player.one_timer, "count");
  deleteField(player.pass, "completed_pass_count");
  deleteField(player.fifty_fifty, "count");
  deleteField(player.speed_flip, "count");
  deleteField(player.half_flip, "count");
  deleteField(player.half_volley, "count");
  deleteField(player.wavedash, "count");
  deleteField(player.touch, "touch_count");
  deleteField(player.whiff, "whiff_count");
  deleteField(player.flick, "count");
  deleteField(player.dodge_reset, "count");
  deleteField(player.ball_carry, "carry_count");
  deleteField(player.air_dribble, "count");
  deleteField(player.boost, "amount_used");
  deleteField(player.bump, "bumps_inflicted");
  deleteField(player.movement, "tracked_time");
  deleteField(player.positioning, "tracked_time");
  deleteField(player.rotation, "tracked_time");
  deleteField(player.powerslide, "total_duration");
  deleteField(player.demo, "demos_inflicted");

  const derivedFrame = applyStatsTimelineEventDerivedStats(timeline).frames[0]!;
  const derivedPlayer = derivedFrame.players[0]!;

  assert.equal(derivedFrame.team_zero.core.goals, 0);
  assert.equal(derivedFrame.team_zero.possession.tracked_time, 0);
  assert.equal(derivedFrame.team_zero.ball_half.tracked_time, 0);
  assert.equal(derivedFrame.team_zero.rotation.rotation_count, 0);
  assert.equal(derivedFrame.team_zero.rush.count, 0);
  assert.equal(derivedFrame.team_zero.backboard.count, 0);
  assert.equal(derivedFrame.team_zero.double_tap.count, 0);
  assert.equal(derivedFrame.team_zero.one_timer.count, 0);
  assert.equal(derivedFrame.team_zero.pass.completed_pass_count, 0);
  assert.equal(derivedFrame.team_zero.ball_carry.carry_count, 0);
  assert.equal(derivedFrame.team_zero.air_dribble.count, 0);
  assert.equal(derivedFrame.team_zero.boost.amount_used, 0);
  assert.equal(derivedFrame.team_zero.bump.bumps_inflicted, 0);
  assert.equal(derivedFrame.team_zero.half_volley.count, 0);
  assert.equal(derivedFrame.team_zero.movement.tracked_time, 0);
  assert.equal(derivedFrame.team_zero.powerslide.total_duration, 0);
  assert.equal(derivedFrame.team_zero.demo.demos_inflicted, 0);
  assert.equal(derivedFrame.team_zero.fifty_fifty.count, 0);

  assert.equal(derivedPlayer.core.goals, 0);
  assert.equal(derivedPlayer.backboard.count, 0);
  assert.equal(derivedPlayer.ceiling_shot.count, 0);
  assert.equal(derivedPlayer.wall_aerial.count, 0);
  assert.equal(derivedPlayer.wall_aerial_shot.count, 0);
  assert.equal(derivedPlayer.double_tap.count, 0);
  assert.equal(derivedPlayer.one_timer.count, 0);
  assert.equal(derivedPlayer.pass.completed_pass_count, 0);
  assert.equal(derivedPlayer.fifty_fifty.count, 0);
  assert.equal(derivedPlayer.speed_flip.count, 0);
  assert.equal(derivedPlayer.half_flip.count, 0);
  assert.equal(derivedPlayer.half_volley.count, 0);
  assert.equal(derivedPlayer.wavedash.count, 0);
  assert.equal(derivedPlayer.touch.touch_count, 0);
  assert.equal(derivedPlayer.whiff.whiff_count, 0);
  assert.equal(derivedPlayer.flick.count, 0);
  assert.equal(derivedPlayer.dodge_reset.count, 0);
  assert.equal(derivedPlayer.ball_carry.carry_count, 0);
  assert.equal(derivedPlayer.air_dribble.count, 0);
  assert.equal(derivedPlayer.boost.amount_used, 0);
  assert.equal(derivedPlayer.bump.bumps_inflicted, 0);
  assert.equal(derivedPlayer.movement.tracked_time, 0);
  assert.equal(derivedPlayer.positioning.tracked_time, 0);
  assert.equal(derivedPlayer.rotation.active_game_time, 0);
  assert.equal(derivedPlayer.powerslide.total_duration, 0);
  assert.equal(derivedPlayer.demo.demos_inflicted, 0);
});
