import type {
  RawBoostPadEvent,
  RawReplayFramesData,
  ReplayBoostPad,
  ReplayBoostPadEvent,
  ReplayBoostPadSize,
  ReplayPlayerTrack,
} from "./types";
import { buildPlayerLookup, normalizeReplayTime, playerIdToString } from "./replay-data-helpers";

interface NormalizeReplayProgressTracker {
  advance(units?: number): boolean;
}

interface AsyncNormalizeReplayProgressTracker extends NormalizeReplayProgressTracker {
  yieldToMainThread(): Promise<void>;
}

const BOOST_PAD_SMALL_Z = 70;
const BOOST_PAD_BIG_Z = 73;
const BOOST_PAD_BACK_CORNER_X = 3072;
const BOOST_PAD_BACK_CORNER_Y = 4096;
const BOOST_PAD_BACK_LANE_X = 1792;
const BOOST_PAD_BACK_LANE_Y = 4184;
const BOOST_PAD_BACK_MID_X = 940;
const BOOST_PAD_BACK_MID_Y = 3308;
const BOOST_PAD_CENTER_BACK_Y = 2816;
const BOOST_PAD_SIDE_WALL_X = 3584;
const BOOST_PAD_SIDE_WALL_Y = 2484;
const BOOST_PAD_SIDE_LANE_X = 1788;
const BOOST_PAD_SIDE_LANE_Y = 2300;
const BOOST_PAD_FRONT_LANE_X = 2048;
const BOOST_PAD_FRONT_LANE_Y = 1036;
const BOOST_PAD_CENTER_X = 1024;
const BOOST_PAD_CENTER_MID_Y = 1024;
const BOOST_PAD_GOAL_LINE_Y = 4240;
export const STANDARD_SOCCAR_BOOST_PAD_COUNT = 34;

function pushPad(
  pads: ReplayBoostPad[],
  x: number,
  y: number,
  z: number,
  size: ReplayBoostPadSize,
): void {
  pads.push({
    index: pads.length,
    padId: null,
    size,
    position: { x, y, z },
    events: [],
  });
}

function pushMirrorX(
  pads: ReplayBoostPad[],
  x: number,
  y: number,
  z: number,
  size: ReplayBoostPadSize,
): void {
  pushPad(pads, -x, y, z, size);
  pushPad(pads, x, y, z, size);
}

function pushMirrorY(
  pads: ReplayBoostPad[],
  x: number,
  y: number,
  z: number,
  size: ReplayBoostPadSize,
): void {
  pushPad(pads, x, -y, z, size);
  pushPad(pads, x, y, z, size);
}

function pushMirrorXY(
  pads: ReplayBoostPad[],
  x: number,
  y: number,
  z: number,
  size: ReplayBoostPadSize,
): void {
  pushMirrorX(pads, x, -y, z, size);
  pushMirrorX(pads, x, y, z, size);
}

function buildStandardSoccarBoostPads(): ReplayBoostPad[] {
  const pads: ReplayBoostPad[] = [];

  pushMirrorY(pads, 0, BOOST_PAD_GOAL_LINE_Y, BOOST_PAD_SMALL_Z, "small");
  pushMirrorXY(pads, BOOST_PAD_BACK_LANE_X, BOOST_PAD_BACK_LANE_Y, BOOST_PAD_SMALL_Z, "small");
  pushMirrorXY(pads, BOOST_PAD_BACK_CORNER_X, BOOST_PAD_BACK_CORNER_Y, BOOST_PAD_BIG_Z, "big");
  pushMirrorXY(pads, BOOST_PAD_BACK_MID_X, BOOST_PAD_BACK_MID_Y, BOOST_PAD_SMALL_Z, "small");
  pushMirrorY(pads, 0, BOOST_PAD_CENTER_BACK_Y, BOOST_PAD_SMALL_Z, "small");
  pushMirrorXY(pads, BOOST_PAD_SIDE_WALL_X, BOOST_PAD_SIDE_WALL_Y, BOOST_PAD_SMALL_Z, "small");
  pushMirrorXY(pads, BOOST_PAD_SIDE_LANE_X, BOOST_PAD_SIDE_LANE_Y, BOOST_PAD_SMALL_Z, "small");
  pushMirrorXY(pads, BOOST_PAD_FRONT_LANE_X, BOOST_PAD_FRONT_LANE_Y, BOOST_PAD_SMALL_Z, "small");
  pushMirrorY(pads, 0, BOOST_PAD_CENTER_MID_Y, BOOST_PAD_SMALL_Z, "small");
  pushMirrorX(pads, BOOST_PAD_SIDE_WALL_X, 0, BOOST_PAD_BIG_Z, "big");
  pushMirrorX(pads, BOOST_PAD_CENTER_X, 0, BOOST_PAD_SMALL_Z, "small");

  return pads;
}

function parseBoostPadAvailability(kind: unknown): boolean | null {
  if (kind === "Available") {
    return true;
  }
  if (kind && typeof kind === "object") {
    if ("Available" in kind) {
      return true;
    }
    if ("PickedUp" in kind) {
      return false;
    }
    const taggedKind = (kind as { kind?: unknown }).kind;
    if (taggedKind === "Available") {
      return true;
    }
    if (taggedKind === "PickedUp") {
      return false;
    }
  }
  return null;
}

function parseBoostPadSize(size: unknown): ReplayBoostPadSize | null {
  if (size === "big" || size === "Big") {
    return "big";
  }
  if (size === "small" || size === "Small") {
    return "small";
  }
  return null;
}

function inferBoostPadSize(events: RawBoostPadEvent[]): ReplayBoostPadSize | null {
  let lastPickupTime: number | null = null;
  for (const event of events) {
    const available = parseBoostPadAvailability(event.kind);
    if (available === false) {
      lastPickupTime = event.time;
      continue;
    }
    if (available === true && lastPickupTime !== null) {
      return event.time - lastPickupTime >= 7 ? "big" : "small";
    }
  }

  return null;
}

export function buildBoostPads(
  raw: RawReplayFramesData,
  players: ReplayPlayerTrack[],
  startTime: number,
  progressTracker?: NormalizeReplayProgressTracker,
): ReplayBoostPad[] {
  const playersById = buildPlayerLookup(players);
  const eventsByPadId = new Map<string, RawBoostPadEvent[]>();

  for (const event of raw.boost_pad_events ?? []) {
    const availability = parseBoostPadAvailability(event.kind);
    if (availability === null) {
      progressTracker?.advance();
      continue;
    }
    const bucket = eventsByPadId.get(event.pad_id);
    if (bucket) {
      bucket.push(event);
    } else {
      eventsByPadId.set(event.pad_id, [event]);
    }
    progressTracker?.advance();
  }

  const rawPads = raw.boost_pads;
  if (!rawPads || rawPads.length === 0) {
    progressTracker?.advance(STANDARD_SOCCAR_BOOST_PAD_COUNT);
    return buildStandardSoccarBoostPads();
  }

  const sortedPads = [...rawPads].sort((left, right) => left.index - right.index);
  const pads = new Array<ReplayBoostPad>(sortedPads.length);

  for (let index = 0; index < sortedPads.length; index += 1) {
    const pad = sortedPads[index]!;
    const padId = typeof pad.pad_id === "string" ? pad.pad_id : null;
    const rawEvents = padId ? [...(eventsByPadId.get(padId) ?? [])] : [];
    const size =
      parseBoostPadSize(pad.size) ??
      inferBoostPadSize(rawEvents) ??
      (pad.position.z >= 72 ? "big" : "small");

    const sortedEvents = rawEvents.sort((left, right) => left.time - right.time);
    const events = new Array<ReplayBoostPadEvent>(sortedEvents.length);

    for (let eventIndex = 0; eventIndex < sortedEvents.length; eventIndex += 1) {
      const event = sortedEvents[eventIndex]!;
      const playerId = event.player ? playerIdToString(event.player) : null;
      events[eventIndex] = {
        time: normalizeReplayTime(event.time, startTime),
        frame: event.frame,
        available: parseBoostPadAvailability(event.kind) ?? true,
        playerId,
        playerName: playerId ? (playersById.get(playerId)?.name ?? playerId) : null,
      };
    }

    pads[index] = {
      index: pad.index,
      padId,
      size,
      position: pad.position,
      events,
    };
    progressTracker?.advance();
  }

  return pads;
}

export async function buildBoostPadsAsync(
  raw: RawReplayFramesData,
  players: ReplayPlayerTrack[],
  startTime: number,
  progressTracker: AsyncNormalizeReplayProgressTracker,
): Promise<ReplayBoostPad[]> {
  const playersById = buildPlayerLookup(players);
  const eventsByPadId = new Map<string, RawBoostPadEvent[]>();

  for (const event of raw.boost_pad_events ?? []) {
    const availability = parseBoostPadAvailability(event.kind);
    if (availability === null) {
      if (progressTracker.advance()) {
        await progressTracker.yieldToMainThread();
      }
      continue;
    }
    const bucket = eventsByPadId.get(event.pad_id);
    if (bucket) {
      bucket.push(event);
    } else {
      eventsByPadId.set(event.pad_id, [event]);
    }
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
  }

  const rawPads = raw.boost_pads;
  if (!rawPads || rawPads.length === 0) {
    if (progressTracker.advance(STANDARD_SOCCAR_BOOST_PAD_COUNT)) {
      await progressTracker.yieldToMainThread();
    }
    return buildStandardSoccarBoostPads();
  }

  const sortedPads = [...rawPads].sort((left, right) => left.index - right.index);
  const pads = new Array<ReplayBoostPad>(sortedPads.length);

  for (let index = 0; index < sortedPads.length; index += 1) {
    const pad = sortedPads[index]!;
    const padId = typeof pad.pad_id === "string" ? pad.pad_id : null;
    const rawEvents = padId ? [...(eventsByPadId.get(padId) ?? [])] : [];
    const size =
      parseBoostPadSize(pad.size) ??
      inferBoostPadSize(rawEvents) ??
      (pad.position.z >= 72 ? "big" : "small");

    const sortedEvents = rawEvents.sort((left, right) => left.time - right.time);
    const events = new Array<ReplayBoostPadEvent>(sortedEvents.length);

    for (let eventIndex = 0; eventIndex < sortedEvents.length; eventIndex += 1) {
      const event = sortedEvents[eventIndex]!;
      const playerId = event.player ? playerIdToString(event.player) : null;
      events[eventIndex] = {
        time: normalizeReplayTime(event.time, startTime),
        frame: event.frame,
        available: parseBoostPadAvailability(event.kind) ?? true,
        playerId,
        playerName: playerId ? (playersById.get(playerId)?.name ?? playerId) : null,
      };
    }

    pads[index] = {
      index: pad.index,
      padId,
      size,
      position: pad.position,
      events,
    };
    if (progressTracker.advance()) {
      await progressTracker.yieldToMainThread();
    }
  }

  return pads;
}
