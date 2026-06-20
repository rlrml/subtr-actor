import assert from "node:assert/strict";
import { test } from "node:test";
import {
  decodeStatsPlayerConfig,
  encodeStatsPlayerConfig,
  getStatsPlayerConfigParamSnapshot,
  getStatsPlayerConfigFromLocation,
  isStatsPlayerConfigDebugEnabled,
  mapWindowPlacementToViewport,
  setStatsPlayerConfigOnUrl,
  type StatsPlayerConfig,
} from "./playerConfig.ts";

const CONFIG: StatsPlayerConfig = {
  version: 1,
  playback: {
    currentTime: 42.5,
    playing: true,
    rate: 1.5,
    skipPostGoalTransitions: true,
    skipKickoffs: false,
  },
  camera: {
    mode: "follow",
    attachedPlayerId: "player-one",
    ballCam: true,
    usePlayerCameraSettings: false,
    customSettings: {
      fov: 108,
      height: 120,
      pitch: -5,
      distance: 280,
      stiffness: 0.4,
      swivelSpeed: 3.2,
      transitionSpeed: 1.1,
    },
  },
  overlays: {
    timelineEvents: ["touch", "demo"],
    timelineRanges: ["boost"],
    mechanics: ["air_dribble", "double_tap"],
    renderEffects: ["touch"],
    followedPlayerHud: true,
    boostPads: false,
    boostPickupAnimation: true,
    hitboxWireframes: true,
    hitboxOnlyMode: true,
  },
  recording: {
    fps: 60,
    playbackRate: 0.5,
  },
  singletonWindows: [
    {
      id: "camera",
      placement: {
        x: 16,
        y: 64,
        viewport: { width: 1920, height: 1080 },
        zIndex: 31,
        visible: true,
      },
    },
    {
      id: "event-playlist",
      placement: {
        x: 1280,
        y: 180,
        viewport: { width: 1920, height: 1080 },
        zIndex: 32,
        visible: false,
      },
    },
    {
      id: "replay-loading",
      placement: {
        x: 1240,
        y: 64,
        viewport: { width: 1920, height: 1080 },
        zIndex: 34,
        visible: true,
      },
    },
    {
      id: "scoreboard",
      placement: {
        x: 820,
        y: 48,
        viewport: { width: 1920, height: 1080 },
        zIndex: 33,
        visible: true,
      },
    },
  ],
  statsWindows: [
    {
      id: "stats-1",
      kind: "ad-hoc",
      placement: {
        x: 420,
        y: 120,
        viewport: { width: 1920, height: 1080 },
        zIndex: 40,
        visible: true,
      },
      playerId: null,
      team: null,
      entries: [
        { statId: "player.core.score", targetId: "player-one" },
        { statId: "team.core.goals", targetId: "blue" },
      ],
    },
    {
      id: "stats-2",
      kind: "kickoff-overview",
      placement: {
        x: 760,
        y: 120,
        viewport: { width: 1920, height: 1080 },
        zIndex: 41,
        visible: true,
      },
      playerId: null,
      team: null,
      entries: [],
    },
  ],
  moduleConfigs: {
    boost: {
      padTypes: ["big"],
      comparisons: ["both", "missed"],
      activities: ["active"],
      fieldHalves: ["opponent"],
      playerIds: ["player-one"],
    },
    touch: {
      decaySeconds: 7,
      breakdownClasses: ["kind"],
    },
  },
};

test("stats player config round-trips through compressed url-safe encoding", () => {
  const encoded = encodeStatsPlayerConfig(CONFIG);
  assert.match(encoded, /^[A-Za-z0-9_-]+$/);
  assert.deepEqual(decodeStatsPlayerConfig(encoded), CONFIG);
});

test("stats player config also accepts uncompressed JSON cfg payloads", () => {
  assert.deepEqual(decodeStatsPlayerConfig(JSON.stringify(CONFIG)), CONFIG);
});

test("stats player config preserves use-player-camera setting", () => {
  const config = decodeStatsPlayerConfig(
    JSON.stringify({
      ...CONFIG,
      camera: {
        ...CONFIG.camera,
        usePlayerCameraSettings: true,
        customSettings: null,
      },
    }),
  );

  assert.equal(config.camera.usePlayerCameraSettings, true);
  assert.equal(config.camera.customSettings, null);
});

test("stats player config preserves plugin-only overlay extension fields", () => {
  const config: StatsPlayerConfig = {
    ...CONFIG,
    overlays: {
      ...CONFIG.overlays,
      pluginRenderEffects: ["mechanics", "team", "goal_context"],
      pluginHudOverlay: false,
    },
  };

  assert.deepEqual(decodeStatsPlayerConfig(JSON.stringify(config)), config);
});

test("stats player config can live in the URL hash without disturbing query params", () => {
  const url = setStatsPlayerConfigOnUrl(
    new URL("https://viewer.example/app/?r=abc#tab=stats"),
    CONFIG,
  );
  assert.equal(url.search, "?r=abc");

  const decoded = getStatsPlayerConfigFromLocation({
    search: url.search,
    hash: url.hash,
  } as Location);
  assert.deepEqual(decoded, CONFIG);
});

test("stats player config accepts raw JSON from URL hash cfg values", () => {
  const decoded = getStatsPlayerConfigFromLocation({
    search: "",
    hash: `#cfg=${encodeURIComponent(JSON.stringify(CONFIG))}`,
  } as Location);

  assert.deepEqual(decoded, CONFIG);
});

test("stats player config also accepts cfg in search params", () => {
  const encoded = encodeStatsPlayerConfig(CONFIG);
  assert.deepEqual(
    getStatsPlayerConfigFromLocation({
      search: `?cfg=${encoded}`,
      hash: "",
    } as Location),
    CONFIG,
  );
});

test("stats player config param snapshot exposes raw parsed cfg sources", () => {
  const hashEncoded = encodeStatsPlayerConfig(CONFIG);
  const searchEncoded = encodeStatsPlayerConfig({
    ...CONFIG,
    playback: { ...CONFIG.playback, currentTime: 12 },
  });
  const snapshot = getStatsPlayerConfigParamSnapshot({
    search: `?cfg=${searchEncoded}&cfgDebug=1`,
    hash: `#cfg=${hashEncoded}&tab=stats`,
  } as Location);

  assert.equal(snapshot.selectedSource, "hash");
  assert.equal(snapshot.selectedValue, hashEncoded);
  assert.deepEqual(snapshot.searchValues, [searchEncoded]);
  assert.deepEqual(snapshot.hashValues, [hashEncoded]);
  assert.deepEqual(snapshot.searchParams, [
    ["cfg", searchEncoded],
    ["cfgDebug", "1"],
  ]);
  assert.deepEqual(snapshot.hashParams, [
    ["cfg", hashEncoded],
    ["tab", "stats"],
  ]);
});

test("stats player config debug flag can come from search or hash params", () => {
  assert.equal(
    isStatsPlayerConfigDebugEnabled({ search: "?cfgDebug=1", hash: "" } as Location),
    true,
  );
  assert.equal(
    isStatsPlayerConfigDebugEnabled({ search: "", hash: "#cfgDebug=true" } as Location),
    true,
  );
  assert.equal(
    isStatsPlayerConfigDebugEnabled({ search: "?cfgDebug=0", hash: "" } as Location),
    false,
  );
});

test("window placement scales from the saved viewport to the current viewport", () => {
  assert.deepEqual(
    mapWindowPlacementToViewport(
      {
        x: 960,
        y: 540,
        viewport: { width: 1920, height: 1080 },
        visible: true,
      },
      { width: 1280, height: 720 },
    ),
    { x: 640, y: 360 },
  );
});

test("window placement is clamped to leave controls reachable", () => {
  assert.deepEqual(
    mapWindowPlacementToViewport(
      {
        x: 1800,
        y: 1000,
        viewport: { width: 1920, height: 1080 },
        visible: true,
      },
      { width: 640, height: 360 },
      200,
      160,
    ),
    { x: 440, y: 200 },
  );
});
