import assert from "node:assert/strict";
import { test } from "node:test";
import {
  fetchBallchasingReplayBytes,
  getBallchasingReplayApiFileUrl,
  getBallchasingReplayFileName,
  getBallchasingReplayFileUrl,
  isBallchasingReplayId,
  normalizeBallchasingReplayId,
} from "../src/ballchasing";

const REPLAY_ID = "56889c3e-c420-45db-92fd-47ce2a3604b0";

test("recognizes and normalizes Ballchasing replay ids", () => {
  assert.equal(isBallchasingReplayId(REPLAY_ID), true);
  assert.equal(normalizeBallchasingReplayId(REPLAY_ID.toUpperCase()), REPLAY_ID);
  assert.equal(isBallchasingReplayId("not-a-replay-id"), false);
});

test("extracts Ballchasing replay ids from web and API URLs", () => {
  assert.equal(
    normalizeBallchasingReplayId(`https://ballchasing.com/replay/${REPLAY_ID}`),
    REPLAY_ID,
  );
  assert.equal(
    normalizeBallchasingReplayId(`https://ballchasing.com/api/replays/${REPLAY_ID}/file`),
    REPLAY_ID,
  );
});

test("rejects non-Ballchasing replay URLs", () => {
  assert.throws(
    () => normalizeBallchasingReplayId(`https://example.com/replay/${REPLAY_ID}`),
    /Invalid Ballchasing replay URL/,
  );
});

test("builds Ballchasing replay file URLs and names", () => {
  assert.equal(
    getBallchasingReplayFileUrl(REPLAY_ID).href,
    `https://ballchasing.com/dl/replay/${REPLAY_ID}`,
  );
  assert.equal(
    getBallchasingReplayFileUrl(REPLAY_ID, "https://proxy.example/ballchasing").href,
    `https://proxy.example/ballchasing/dl/replay/${REPLAY_ID}`,
  );
  assert.equal(
    getBallchasingReplayApiFileUrl(REPLAY_ID).href,
    `https://ballchasing.com/api/replays/${REPLAY_ID}/file`,
  );
  assert.equal(getBallchasingReplayFileName(REPLAY_ID), `ballchasing-${REPLAY_ID}.replay`);
});

test("fetchBallchasingReplayBytes posts to the public download endpoint", async () => {
  const calls: Array<{
    url: string;
    authorization: string | null;
    method: string | null;
  }> = [];
  const bytes = new Uint8Array([1, 2, 3]);
  const fakeFetch: typeof fetch = async (input, init) => {
    calls.push({
      url: input instanceof URL ? input.href : String(input),
      authorization: new Headers(init?.headers).get("Authorization"),
      method: init?.method ?? null,
    });
    return new Response(bytes, { status: 200 });
  };

  const result = await fetchBallchasingReplayBytes(REPLAY_ID, {
    fetch: fakeFetch,
  });

  assert.deepEqual([...result], [...bytes]);
  assert.deepEqual(calls, [
    {
      url: `https://ballchasing.com/dl/replay/${REPLAY_ID}`,
      authorization: null,
      method: "POST",
    },
  ]);
});
