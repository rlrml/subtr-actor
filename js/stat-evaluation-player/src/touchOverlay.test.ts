import test from "node:test";
import assert from "node:assert/strict";

import type { StatsFrame } from "./statsTimeline.ts";
import { getLastTouchPlayer } from "./touchOverlay.ts";

test("getLastTouchPlayer returns the player marked as the current last touch", () => {
  const statsFrame: StatsFrame = {
    frame_number: 42,
    time: 12.3,
    dt: 0.1,
    players: [
      {
        player_id: { Steam: "blue" },
        name: "Blue",
        is_team_0: true,
        touch: {
          touch_count: 2,
          is_last_touch: false,
        },
      },
      {
        player_id: { Steam: "orange" },
        name: "Orange",
        is_team_0: false,
        touch: {
          touch_count: 3,
          is_last_touch: true,
          time_since_last_touch: 0.4,
        },
      },
    ],
  };

  assert.equal(getLastTouchPlayer(statsFrame)?.name, "Orange");
});

test("getLastTouchPlayer returns null when no player is the current last touch", () => {
  const statsFrame: StatsFrame = {
    frame_number: 42,
    time: 12.3,
    dt: 0.1,
    players: [
      {
        player_id: { Steam: "blue" },
        name: "Blue",
        is_team_0: true,
        touch: {
          touch_count: 0,
          is_last_touch: false,
        },
      },
    ],
  };

  assert.equal(getLastTouchPlayer(statsFrame), null);
});
