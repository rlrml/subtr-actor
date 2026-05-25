import assert from "node:assert/strict";
import { test } from "node:test";
import {
  decodeCompressedReplayUrl,
  encodeCompressedReplayUrl,
  getReplayFileNameFromUrl,
  getReplayFetchRequestFromSearch,
  getReplayUrlFromSearch,
} from "./replayUrl.ts";

const BASE_URL = "https://viewer.example/app/";
const GITHUB_REPLAY_URL =
  "https://raw.githubusercontent.com/rlrml/subtr-actor/fix-legacy-rigidbody-normalization/assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay";
const BALLCHASING_ID = "56889c3e-c420-45db-92fd-47ce2a3604b0";

test("getReplayUrlFromSearch accepts replayUrl parameter", () => {
  const replayUrl = getReplayUrlFromSearch(`?replayUrl=${GITHUB_REPLAY_URL}`, BASE_URL);

  assert.equal(replayUrl?.href, GITHUB_REPLAY_URL);
});

test("getReplayUrlFromSearch accepts compressed replay URL parameter", () => {
  const encoded = encodeCompressedReplayUrl(GITHUB_REPLAY_URL);
  const replayUrl = getReplayUrlFromSearch(`?r=${encoded}`, BASE_URL);

  assert.equal(decodeCompressedReplayUrl(encoded), GITHUB_REPLAY_URL);
  assert.equal(replayUrl?.href, GITHUB_REPLAY_URL);
  assert.ok(`?r=${encoded}`.length < `?replayUrl=${encodeURIComponent(GITHUB_REPLAY_URL)}`.length);
});

test("getReplayUrlFromSearch accepts legacy aliases and relative URLs", () => {
  assert.equal(
    getReplayUrlFromSearch("?replay_url=/replays/a.replay", BASE_URL)?.href,
    "https://viewer.example/replays/a.replay",
  );
  assert.equal(
    getReplayUrlFromSearch("?replay=replays/b.replay", BASE_URL)?.href,
    "https://viewer.example/app/replays/b.replay",
  );
});

test("getReplayUrlFromSearch returns null when no replay URL is present", () => {
  assert.equal(getReplayUrlFromSearch("?module=boost", BASE_URL), null);
});

test("getReplayUrlFromSearch rejects non-fetchable URL schemes", () => {
  assert.throws(
    () => getReplayUrlFromSearch("?replayUrl=javascript:alert(1)", BASE_URL),
    /Unsupported replay URL protocol/,
  );
});

test("getReplayUrlFromSearch rejects invalid compressed replay URL parameters", () => {
  assert.throws(
    () => getReplayUrlFromSearch("?r=not-valid-deflate", BASE_URL),
    /Invalid compressed replay URL/,
  );
});

test("getReplayFetchRequestFromSearch accepts a Ballchasing replay id", () => {
  const request = getReplayFetchRequestFromSearch(`?ballchasing=${BALLCHASING_ID}`, BASE_URL);

  assert.equal(request?.kind, "ballchasing");
  assert.equal(request?.url.href, `https://ballchasing.com/dl/replay/${BALLCHASING_ID}`);
  assert.equal(request?.name, `ballchasing-${BALLCHASING_ID}.replay`);
  assert.deepEqual(request?.fetchInit, { method: "POST" });
});

test("getReplayFetchRequestFromSearch accepts Ballchasing replay URLs", () => {
  const request = getReplayFetchRequestFromSearch(
    `?ballchasingReplay=${encodeURIComponent(`https://ballchasing.com/replay/${BALLCHASING_ID}`)}`,
    BASE_URL,
  );

  assert.equal(request?.kind, "ballchasing");
  assert.equal(request?.url.href, `https://ballchasing.com/dl/replay/${BALLCHASING_ID}`);
});

test("getReplayFetchRequestFromSearch falls back to replay URL parameters", () => {
  const request = getReplayFetchRequestFromSearch(`?replayUrl=${GITHUB_REPLAY_URL}`, BASE_URL);

  assert.equal(request?.kind, "url");
  assert.equal(request?.url.href, GITHUB_REPLAY_URL);
  assert.equal(
    request?.name,
    "replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay",
  );
});

test("getReplayFileNameFromUrl derives a readable name", () => {
  assert.equal(
    getReplayFileNameFromUrl(new URL("https://cdn.example/replays/test%20one.replay")),
    "test one.replay",
  );
  assert.equal(getReplayFileNameFromUrl(new URL("https://cdn.example/replays/")), "replays");
});
