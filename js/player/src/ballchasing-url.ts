export const BALLCHASING_API_BASE_URL = "https://ballchasing.com/api";
export const BALLCHASING_BASE_URL = "https://ballchasing.com";

const BALLCHASING_REPLAY_ID_PATTERN =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function buildBallchasingApiUrl(path: string, apiBaseUrl: string | URL): URL {
  const baseHref = apiBaseUrl instanceof URL ? apiBaseUrl.href : apiBaseUrl;
  const normalizedBase = baseHref.replace(/\/+$/, "");
  return new URL(`${normalizedBase}/${path.replace(/^\/+/, "")}`);
}

export function isBallchasingReplayId(value: string): boolean {
  return BALLCHASING_REPLAY_ID_PATTERN.test(value.trim());
}

export function normalizeBallchasingReplayId(value: string): string {
  const trimmed = value.trim();
  if (isBallchasingReplayId(trimmed)) {
    return trimmed.toLowerCase();
  }

  let url: URL;
  try {
    url = new URL(trimmed);
  } catch {
    throw new Error(`Invalid Ballchasing replay id: ${value}`);
  }

  if (!/(^|\.)ballchasing\.com$/i.test(url.hostname)) {
    throw new Error(`Invalid Ballchasing replay URL: ${value}`);
  }

  const pathParts = url.pathname.split("/").filter(Boolean);
  const replayIndex = pathParts.findIndex((part) => part === "replay");
  const apiReplayIndex = pathParts.findIndex((part) => part === "replays");
  const id =
    replayIndex >= 0
      ? pathParts[replayIndex + 1]
      : apiReplayIndex >= 0
        ? pathParts[apiReplayIndex + 1]
        : undefined;

  if (!id || !isBallchasingReplayId(id)) {
    throw new Error(`Invalid Ballchasing replay URL: ${value}`);
  }

  return id.toLowerCase();
}

export function getBallchasingReplayFileName(idOrUrl: string): string {
  return `ballchasing-${normalizeBallchasingReplayId(idOrUrl)}.replay`;
}

export function getBallchasingReplayFileUrl(
  idOrUrl: string,
  baseUrl: string | URL = BALLCHASING_BASE_URL,
): URL {
  const id = normalizeBallchasingReplayId(idOrUrl);
  return buildBallchasingApiUrl(`dl/replay/${encodeURIComponent(id)}`, baseUrl);
}

export function getBallchasingReplayApiFileUrl(
  idOrUrl: string,
  apiBaseUrl: string | URL = BALLCHASING_API_BASE_URL,
): URL {
  const id = normalizeBallchasingReplayId(idOrUrl);
  return buildBallchasingApiUrl(`replays/${encodeURIComponent(id)}/file`, apiBaseUrl);
}
