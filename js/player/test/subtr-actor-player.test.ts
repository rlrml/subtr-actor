import test from "node:test";
import assert from "node:assert/strict";

import { SubtrActorPlayer } from "../src/player/adapter/SubtrActorPlayer.ts";

test("players sharing a display name keep separate entities", () => {
  const raw = {
    meta: {
      team_zero: [{ remote_id: { Steam: "1" }, name: "Same" }],
      team_one: [{ remote_id: { Steam: "2" }, name: "Same" }],
    },
    frame_data: {
      metadata_frames: [{ time: 0 }, { time: 1 }],
      ball_data: { frames: [] },
      players: [
        [{ Steam: "1" }, { frames: [] }],
        [{ Steam: "2" }, { frames: [] }],
      ],
    },
  };

  const adapter = new SubtrActorPlayer(raw as never);

  assert.deepEqual(
    adapter.playerList.map(({ id, name, team }) => ({ id, name, team })),
    [
      { id: "Steam:1", name: "Same", team: 0 },
      { id: "Steam:2", name: "Same (2)", team: 1 },
    ],
  );
  assert.equal(adapter.players.size, 2);
  assert.equal(adapter.players.get("Same (2)")?.id, "Steam:2");
});

test("car visibility follows missing bodies across demolition and respawn seeks", () => {
  const live = {
    Data: {
      rigid_body: {
        location: { x: 100, y: 200, z: 17 },
        rotation: { x: 0, y: 0, z: 0, w: 1 },
      },
      boost_amount: 100,
      boost_active: true,
      player_name: "Driver",
      team: 0,
      is_team_0: true,
    },
  };
  const raw = {
    meta: { team_zero: [], team_one: [] },
    frame_data: {
      metadata_frames: Array.from({ length: 6 }, (_, time) => ({
        time: time + 10,
        seconds_remaining: 300 - time,
      })),
      ball_data: { frames: [] },
      players: [["driver", { frames: ["Empty", live, "Empty", "Empty", live, "Empty"] }]],
    },
  } satisfies ConstructorParameters<typeof SubtrActorPlayer>[0];
  const adapter = new SubtrActorPlayer(raw, { timelineCompaction: false });
  const car = adapter.getPlayer("Driver")!;
  for (const [time, visible] of [
    [0, false],
    [1, true],
    [1.99, true],
    [2, false],
    [3.99, false],
    [4, true],
    [5, false],
    [3, false],
    [1, true],
    [4, true],
    [0, false],
  ] as const) {
    adapter.seek(time);
    assert.equal(car.isVisible, visible, `visibility at ${time}`);
    if (!visible) assert.equal(car.isBoosting, false);
  }
});
