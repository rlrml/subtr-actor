import assert from "node:assert/strict";
import test from "node:test";
import {
  applyConfigAdapterSnapshot,
  getConfigAdapterSnapshot,
  type StatsPlayerConfigAdapter,
} from "./configAdapters.ts";

test("config adapter snapshots use canonical ids", () => {
  const adapters: StatsPlayerConfigAdapter[] = [
    {
      id: "touch",
      getConfig() {
        return { decaySeconds: 7 };
      },
    },
    {
      id: "boost",
      aliases: ["boost-pickup-animation"],
      getConfig() {
        return { padTypes: ["big"] };
      },
    },
  ];

  assert.deepEqual(getConfigAdapterSnapshot(adapters), {
    touch: { decaySeconds: 7 },
    boost: { padTypes: ["big"] },
  });
});

test("config adapter apply accepts aliases for migrated config keys", () => {
  let applied: unknown = null;
  applyConfigAdapterSnapshot(
    [
      {
        id: "boost",
        aliases: ["boost-pickup-animation"],
        applyConfig(config) {
          applied = config;
        },
      },
    ],
    {
      "boost-pickup-animation": { comparisons: ["missed"] },
    },
  );

  assert.deepEqual(applied, { comparisons: ["missed"] });
});

test("config adapter canonical config wins over aliases", () => {
  let applied: unknown = null;
  applyConfigAdapterSnapshot(
    [
      {
        id: "boost",
        aliases: ["boost-pickup-animation"],
        applyConfig(config) {
          applied = config;
        },
      },
    ],
    {
      boost: { comparisons: ["both"] },
      "boost-pickup-animation": { comparisons: ["missed"] },
    },
  );

  assert.deepEqual(applied, { comparisons: ["both"] });
});

test("config adapter snapshots reject duplicate canonical ids", () => {
  assert.throws(
    () =>
      getConfigAdapterSnapshot([
        { id: "touch", getConfig: () => ({}) },
        { id: "touch", getConfig: () => ({}) },
      ]),
    /Duplicate stats player config adapter id: touch/,
  );
});
