import test from "node:test";
import assert from "node:assert/strict";

import type { ReplayModel } from "@rlrml/player";
import type { StatsFrame } from "./statsTimeline.ts";
import {
  buildTouchMarkers,
  getLastTouchPlayer,
  getVisibleTouchMarkers,
  touchMarkerColor,
  touchMarkerRingColors,
  type TouchMarker,
} from "./touchOverlay.ts";
import {
  createLegacyStatsTimeline,
  createPlayerStatsSnapshot,
  createStatsFrame,
} from "./testStatsTimeline.ts";

test("getLastTouchPlayer returns the player marked as the current last touch", () => {
  const statsFrame: StatsFrame = createStatsFrame({
    frame_number: 42,
    time: 12.3,
    dt: 0.1,
    players: [
      createPlayerStatsSnapshot({
        player_id: { Steam: "blue" },
        name: "Blue",
        is_team_0: true,
        touch: {
          touch_count: 2,
          is_last_touch: false,
        },
      }),
      createPlayerStatsSnapshot({
        player_id: { Steam: "orange" },
        name: "Orange",
        is_team_0: false,
        touch: {
          touch_count: 3,
          is_last_touch: true,
          time_since_last_touch: 0.4,
        },
      }),
    ],
  });

  assert.equal(getLastTouchPlayer(statsFrame)?.name, "Orange");
});

test("buildTouchMarkers derives markers from touch stats and ball frames", () => {
  const replay = {
    ballFrames: [{ position: { x: 0, y: 0, z: 0 } }, { position: { x: 100, y: -250, z: 320 } }],
    frames: [{ time: 0 }, { time: 1.25 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
      {
        id: "Steam:orange-id",
        name: "Orange",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    events: {
      touch: [
        {
          time: 1.25,
          frame: 1,
          sample_time: 1.25,
          sample_frame: 1,
          player: { Steam: "blue-id" },
          is_team_0: true,
          tags: [
            { group: "kind", value: "control" },
            { group: "height_band", value: "ground" },
            { group: "surface", value: "ground" },
            { group: "dodge_state", value: "no_dodge" },
            { group: "possession", value: "control" },
            { group: "reception", value: "first_touch" },
          ],
          ball_speed_change: 0,
        },
      ],
    },
    frames: [
      {
        frame_number: 0,
        time: 0,
        dt: 0.1,
        players: [
          createPlayerStatsSnapshot({
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            touch: {
              touch_count: 0,
              is_last_touch: false,
            },
          }),
        ],
      },
      {
        frame_number: 1,
        time: 1.25,
        dt: 0.1,
        players: [
          createPlayerStatsSnapshot({
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            touch: {
              touch_count: 1,
              is_last_touch: true,
              last_touch_time: 1.25,
              last_touch_frame: 1,
            },
          }),
        ],
      },
    ],
  });

  assert.deepEqual(buildTouchMarkers(statsTimeline, replay), [
    {
      id: "touch-stat:1:Steam:blue-id:1",
      time: 1.25,
      frame: 1,
      isTeamZero: true,
      playerId: "Steam:blue-id",
      playerName: "Blue",
      kind: "control",
      intention: "control",
      heightBand: "ground",
      surface: "ground",
      dodgeState: "no_dodge",
      firstTouch: true,
      contested: false,
      classifications: [
        { key: "intention", value: "control", label: "Control", color: 0x000000 },
        { key: "kind", value: "control", label: "Control", color: 0x000000 },
        { key: "height_band", value: "ground", label: "Ground", color: 0xa3e635 },
        { key: "surface", value: "ground", label: "Ground", color: 0x84cc16 },
        { key: "dodge_state", value: "no_dodge", label: "No Dodge", color: 0x94a3b8 },
        { key: "first_touch", value: "true", label: "First touch", color: 0xffffff },
      ],
      position: { x: 100, y: -250, z: 320 },
      endPosition: { x: 100, y: -250, z: 320 },
      totalBallAdvanceDistance: 0,
      totalBallRetreatDistance: 0,
      totalBallTravelDistance: 0,
    },
  ]);
});

test("buildTouchMarkers assigns credited ball movement to the active touch marker", () => {
  const replay = {
    ballFrames: [
      { position: { x: 0, y: 0, z: 92 } },
      { position: { x: 0, y: 100, z: 92 } },
      { position: { x: 40, y: 170, z: 92 } },
    ],
    frames: [{ time: 0 }, { time: 1 }, { time: 2 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    events: {
      touch: [
        {
          time: 0,
          frame: 0,
          sample_time: 0,
          sample_frame: 0,
          player: { Steam: "blue-id" },
          is_team_0: true,
          tags: [
            { group: "kind", value: "control" },
            { group: "height_band", value: "ground" },
            { group: "surface", value: "ground" },
            { group: "dodge_state", value: "no_dodge" },
          ],
          ball_speed_change: 0,
          ball_movement: {
            start_time: 1,
            start_frame: 1,
            end_time: 2,
            end_frame: 2,
            duration: 0.2,
            travel_distance: 180,
            advance_distance: 170,
            retreat_distance: 0,
            finalized: true,
          },
        },
      ],
    },
    frames: [
      {
        frame_number: 0,
        time: 0,
        dt: 0.1,
        players: [
          createPlayerStatsSnapshot({
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            touch: {
              touch_count: 1,
              is_last_touch: true,
              last_touch_time: 0,
              last_touch_frame: 0,
            },
          }),
        ],
      },
      {
        frame_number: 1,
        time: 1,
        dt: 0.1,
        players: [
          createPlayerStatsSnapshot({
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            touch: {
              touch_count: 1,
              is_last_touch: true,
              last_touch_time: 0,
              last_touch_frame: 0,
              total_ball_travel_distance: 100,
              total_ball_advance_distance: 100,
            },
          }),
        ],
      },
      {
        frame_number: 2,
        time: 2,
        dt: 0.1,
        players: [
          createPlayerStatsSnapshot({
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            touch: {
              touch_count: 1,
              is_last_touch: true,
              last_touch_time: 0,
              last_touch_frame: 0,
              total_ball_travel_distance: 180,
              total_ball_advance_distance: 170,
              total_ball_retreat_distance: 0,
            },
          }),
        ],
      },
    ],
  });

  const [marker] = buildTouchMarkers(statsTimeline, replay);
  assert.equal(marker?.totalBallTravelDistance, 180);
  assert.equal(marker?.totalBallAdvanceDistance, 170);
  assert.equal(marker?.totalBallRetreatDistance, 0);
  assert.deepEqual(marker?.endPosition, { x: 40, y: 170, z: 92 });
});

test("buildTouchMarkers uses normalized replay frame time instead of raw stats time", () => {
  const replay = {
    ballFrames: [{ position: { x: 0, y: 0, z: 0 } }, { position: { x: 100, y: -250, z: 320 } }],
    frames: [{ time: 0 }, { time: 1.25 }],
    players: [
      {
        id: "Steam:blue-id",
        name: "Blue",
      },
    ],
  } as ReplayModel;

  const statsTimeline = createLegacyStatsTimeline({
    events: {
      touch: [
        {
          time: 4.75,
          frame: 1,
          sample_time: 4.75,
          sample_frame: 1,
          player: { Steam: "blue-id" },
          is_team_0: true,
          tags: [
            { group: "kind", value: "control" },
            { group: "height_band", value: "ground" },
            { group: "surface", value: "ground" },
            { group: "dodge_state", value: "no_dodge" },
          ],
          ball_speed_change: 0,
        },
      ],
    },
    frames: [
      {
        frame_number: 1,
        time: 4.75,
        dt: 0.1,
        players: [
          createPlayerStatsSnapshot({
            player_id: { Steam: "blue-id" },
            name: "Blue",
            is_team_0: true,
            touch: {
              touch_count: 1,
              is_last_touch: true,
              last_touch_time: 4.75,
              last_touch_frame: 1,
            },
          }),
        ],
      },
    ],
  });

  assert.equal(buildTouchMarkers(statsTimeline, replay)[0]?.time, 1.25);
});

test("getVisibleTouchMarkers returns markers inside the decay window", () => {
  const markers = [
    {
      id: "touch:1",
      time: 2,
      frame: 20,
      isTeamZero: true,
      playerId: "Steam:blue-id",
      playerName: "Blue",
      kind: null,
      intention: null,
      heightBand: null,
      surface: null,
      dodgeState: null,
      firstTouch: false,
      contested: false,
      classifications: [],
      position: { x: 0, y: 0, z: 0 },
      endPosition: { x: 0, y: 0, z: 0 },
      totalBallAdvanceDistance: 0,
      totalBallRetreatDistance: 0,
      totalBallTravelDistance: 0,
    },
    {
      id: "touch:2",
      time: 8,
      frame: 80,
      isTeamZero: false,
      playerId: "Steam:orange-id",
      playerName: "Orange",
      kind: null,
      intention: null,
      heightBand: null,
      surface: null,
      dodgeState: null,
      firstTouch: false,
      contested: false,
      classifications: [],
      position: { x: 0, y: 0, z: 0 },
      endPosition: { x: 0, y: 0, z: 0 },
      totalBallAdvanceDistance: 0,
      totalBallRetreatDistance: 0,
      totalBallTravelDistance: 0,
    },
  ];

  assert.deepEqual(
    getVisibleTouchMarkers(markers, 10, 5).map((marker) => marker.id),
    ["touch:2"],
  );
});

test("touchMarkerColor selects palettes by color mode with fallbacks", () => {
  const marker: TouchMarker = {
    id: "touch:1",
    time: 0,
    frame: 0,
    isTeamZero: false,
    playerId: "Steam:orange-id",
    playerName: "Orange",
    kind: "hard_hit",
    intention: "shot",
    heightBand: "high_air",
    surface: "wall",
    dodgeState: "dodge",
    firstTouch: false,
    contested: true,
    classifications: [
      { key: "intention", value: "shot", label: "Shot", color: 0xff00c8 },
      { key: "kind", value: "hard_hit", label: "Hard Hit", color: 0xff5d6c },
      { key: "height_band", value: "high_air", label: "High Air", color: 0x818cf8 },
      { key: "surface", value: "wall", label: "Wall", color: 0xf97316 },
      { key: "dodge_state", value: "dodge", label: "Dodge", color: 0xe879f9 },
      { key: "contested", value: "true", label: "Contested", color: 0xef4444 },
    ],
    position: { x: 0, y: 0, z: 0 },
    endPosition: { x: 0, y: 0, z: 0 },
    totalBallAdvanceDistance: 0,
    totalBallRetreatDistance: 0,
    totalBallTravelDistance: 0,
  };

  assert.equal(touchMarkerColor(marker, "team"), 0xffc15c);
  assert.equal(touchMarkerColor({ ...marker, isTeamZero: true }, "team"), 0x59c3ff);
  assert.equal(touchMarkerColor(marker, "intention"), 0xff00c8);
  assert.equal(touchMarkerColor({ ...marker, intention: "boom" }, "intention"), 0xf472b6);
  assert.equal(touchMarkerColor(marker, "kind"), 0xff5d6c);
  assert.equal(touchMarkerColor(marker, "height_band"), 0x818cf8);
  assert.equal(touchMarkerColor(marker, "surface"), 0xf97316);
  assert.equal(touchMarkerColor(marker, "dodge_state"), 0xe879f9);
  assert.equal(touchMarkerColor(marker, "flag"), 0xef4444);
  assert.deepEqual(touchMarkerRingColors(marker, ["height_band"]), [0x818cf8]);
  assert.deepEqual(
    touchMarkerRingColors({ ...marker, isTeamZero: true }, ["team", "kind"]),
    [0x59c3ff, 0xff5d6c],
  );
  assert.deepEqual(touchMarkerRingColors(marker, ["team"]), [0xffc15c]);
  assert.deepEqual(touchMarkerRingColors(marker, []), [0xffc15c]);
  assert.equal(touchMarkerColor({ ...marker, kind: "control" }, "kind"), 0x000000);
  assert.equal(touchMarkerColor({ ...marker, intention: "control" }, "intention"), 0x000000);
  assert.equal(touchMarkerColor({ ...marker, intention: null }, "intention"), 0x9aa5b1);
  assert.equal(touchMarkerColor({ ...marker, kind: "mystery" }, "kind"), 0x9aa5b1);
  assert.equal(
    touchMarkerColor({ ...marker, contested: false, firstTouch: true }, "flag"),
    0xffffff,
  );
  assert.equal(
    touchMarkerColor({ ...marker, contested: false, firstTouch: false }, "flag"),
    0x9aa5b1,
  );
});
