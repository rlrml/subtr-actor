import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";

/**
 * Capture of a mechanic a human observed during playback that the detector
 * never emitted — a "missed event" / false negative. The capture is anchored to
 * the engine-authoritative frame plus the wall-clock time and (optionally) the
 * attached player as subject, so it can later be matched against detector output
 * to measure recall.
 *
 * The capture is intentionally self-contained: it carries everything needed to
 * round-trip to a JSON export OR to the rocket-sense missed-event endpoint
 * (`POST /api/v1/events/reviews`, which accepts a review with a null detected
 * `event_id`). See `crates/rocket-sense-server/src/api/mechanics.rs`
 * (`create_missed_event_review`).
 */
export interface MissedEventCaptureRecord {
  /** Stable client id for list management (not sent to the server). */
  readonly localId: string;
  /** Canonical-ish mechanic key (e.g. `flick`); rocket-sense re-normalizes it. */
  readonly mechanic: string;
  /** Engine frame index the mechanic occurs on. */
  readonly frame: number;
  /** Wall-clock playback time in seconds. */
  readonly time: number;
  readonly subjectKind: "player" | null;
  readonly subjectId: string | null;
  readonly playerName: string | null;
  /** Optional span around the event, in frames. */
  readonly startFrame: number | null;
  readonly endFrame: number | null;
  readonly notes: string | null;
  /** Reviewer-asserted confidence (defaults to 1.0 — a confirmed positive). */
  readonly confidence: number;
  /** Resolved rocket-sense replay UUID, when known; null disables upload. */
  readonly replayId: string | null;
  /** Free-form context preserved verbatim in the export and the snapshot. */
  readonly context: Record<string, unknown>;
}

/** Request body for `POST /api/v1/events/reviews` (missed-event review). */
export interface MissedEventReviewPayload {
  replay_id: string;
  reviewed_mechanic: string;
  reviewed_subject_kind?: string;
  reviewed_subject_id?: string;
  reviewed_event_frame: number;
  reviewed_start_frame?: number;
  reviewed_end_frame?: number;
  reviewed_event_time?: number;
  confidence?: number;
  notes?: string;
  status?: string;
  context?: Record<string, unknown>;
}

/**
 * Resolve the rocket-sense replay UUID for the current viewer session.
 *
 * Capture works without it (JSON export is always available), but uploading to
 * rocket-sense needs the replay's DB id. Priority: an explicit `replayId` /
 * `replay-id` query param (how rocket-sense should launch the viewer for
 * capture), then a caller-supplied fallback (e.g. the active review item).
 */
export function resolveCaptureReplayId(search: string, fallback?: string | null): string | null {
  const params = new URLSearchParams(search);
  const fromQuery = params.get("replayId") ?? params.get("replay-id");
  const candidate = (fromQuery ?? fallback ?? "").trim();
  return candidate.length > 0 ? candidate : null;
}

/**
 * Auth headers for review submission. Mirrors `mechanicsReviewAuthHeaders` in
 * `mechanicsReviewWindow.ts`: a `reviewToken`/`token` query param or the
 * `rocket_sense_access_token` localStorage value becomes a bearer token.
 */
export function reviewAuthHeaders(): Record<string, string> {
  const params = new URLSearchParams(window.location.search);
  const token =
    params.get("reviewToken") ??
    params.get("token") ??
    window.localStorage.getItem("rocket_sense_access_token");
  return token ? { Authorization: `Bearer ${token}` } : {};
}

/** Build the endpoint request body from a capture, omitting absent fields. */
export function buildMissedEventReviewPayload(
  record: MissedEventCaptureRecord,
): MissedEventReviewPayload | null {
  if (!record.replayId) {
    return null;
  }
  const payload: MissedEventReviewPayload = {
    replay_id: record.replayId,
    reviewed_mechanic: record.mechanic,
    reviewed_event_frame: record.frame,
    reviewed_event_time: record.time,
    confidence: record.confidence,
    status: "confirmed",
  };
  if (record.subjectKind && record.subjectId) {
    payload.reviewed_subject_kind = record.subjectKind;
    payload.reviewed_subject_id = record.subjectId;
  }
  if (record.startFrame !== null) {
    payload.reviewed_start_frame = record.startFrame;
  }
  if (record.endFrame !== null) {
    payload.reviewed_end_frame = record.endFrame;
  }
  if (record.notes && record.notes.trim()) {
    payload.notes = record.notes.trim();
  }
  if (Object.keys(record.context).length > 0) {
    payload.context = record.context;
  }
  return payload;
}

export interface CaptureFromPlayerOptions {
  readonly mechanic: string;
  readonly replayId: string | null;
  readonly notes?: string | null;
  readonly localId: string;
}

/**
 * Snapshot a missed-event capture from the live player state: current frame /
 * time as the anchor, the attached (camera-followed) player as subject.
 */
export function captureMissedEventFromPlayer(
  player: StatsReplayPlayer,
  options: CaptureFromPlayerOptions,
): MissedEventCaptureRecord {
  const state = player.getState();
  const frame = Math.max(0, Math.round(state.frameIndex ?? 0));
  const time = state.currentTime ?? 0;
  const subjectId = state.attachedPlayerId ?? null;
  const playerName = subjectId
    ? (player.replay.players.find((entry) => entry.id === subjectId)?.name ?? null)
    : null;

  return {
    localId: options.localId,
    mechanic: options.mechanic,
    frame,
    time,
    subjectKind: subjectId ? "player" : null,
    subjectId,
    playerName,
    startFrame: null,
    endFrame: null,
    notes: options.notes?.trim() ? options.notes.trim() : null,
    confidence: 1,
    replayId: options.replayId,
    context: {
      capturedFrom: "stat-evaluation-player",
      attachedPlayerId: subjectId,
      playerName,
      durationSeconds: player.replay.duration ?? null,
    },
  };
}

export interface MissedEventUploadResult {
  readonly record: MissedEventCaptureRecord;
  readonly ok: boolean;
  readonly message: string;
}

/**
 * POST a single capture to the rocket-sense missed-event endpoint. Returns a
 * structured result rather than throwing so the controller can report per-row
 * outcomes.
 */
export async function uploadMissedEvent(
  record: MissedEventCaptureRecord,
  endpoint = "/api/v1/events/reviews",
): Promise<MissedEventUploadResult> {
  const payload = buildMissedEventReviewPayload(record);
  if (!payload) {
    return {
      record,
      ok: false,
      message: "No replay id — cannot upload (export JSON instead).",
    };
  }
  try {
    const response = await fetch(endpoint, {
      method: "POST",
      headers: { "content-type": "application/json", ...reviewAuthHeaders() },
      credentials: "same-origin",
      body: JSON.stringify(payload),
    });
    if (!response.ok) {
      let message = `${response.status}${response.statusText ? ` ${response.statusText}` : ""}`;
      try {
        const body = (await response.json()) as { error?: unknown };
        if (typeof body.error === "string") {
          message = body.error;
        }
      } catch {
        // Keep the HTTP status fallback.
      }
      return { record, ok: false, message };
    }
    return { record, ok: true, message: "uploaded" };
  } catch (error) {
    return {
      record,
      ok: false,
      message: error instanceof Error ? error.message : String(error),
    };
  }
}

/** Mechanic keys offered in the capture picker. rocket-sense normalizes these. */
export const MISSED_EVENT_MECHANIC_OPTIONS = [
  "flick",
  "musty_flick",
  "whiff",
  "double_tap",
  "ceiling_shot",
  "wall_aerial",
  "flip_reset",
  "one_timer",
  "speed_flip",
  "half_flip",
  "wavedash",
  "demo",
] as const;
