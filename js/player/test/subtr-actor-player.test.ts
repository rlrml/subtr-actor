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
