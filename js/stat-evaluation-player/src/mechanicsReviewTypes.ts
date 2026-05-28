import type { PlaylistManifestPage } from "@rlrml/player";
import type { ReplayLoadBundle, ReplayLoadProgress } from "./replayLoader.ts";

export type MechanicsReviewPlaybackBound =
  | { kind: "time"; value: number }
  | { kind: "frame"; value: number };

export interface MechanicsReviewReplay {
  id: string;
  path?: string;
  label?: string;
  locator?: Record<string, unknown>;
  meta?: Record<string, unknown>;
}

export interface MechanicsReviewItemMeta {
  confidence?: number | null;
  eventId?: string;
  mechanic?: string;
  mechanicLabel?: string;
  playerId?: string;
  playerName?: string | null;
  reason?: string;
  reviewEndpoint?: string;
  reviewStatus?: string | null;
  target?: Record<string, unknown>;
  followupGoal?: unknown;
  [key: string]: unknown;
}

export interface MechanicsReviewItem {
  id?: string;
  replay: string;
  start: MechanicsReviewPlaybackBound;
  end: MechanicsReviewPlaybackBound;
  label?: string;
  meta?: MechanicsReviewItemMeta;
}

export interface MechanicsReviewPlaylist {
  label?: string;
  replays?: MechanicsReviewReplay[];
  items: MechanicsReviewItem[];
  page?: PlaylistManifestPage;
  playback?: unknown;
  meta?: unknown;
}

export type MechanicsReviewReplayLoadStatus = "idle" | "loading" | "loaded" | "error";

export interface MechanicsReviewReplayLoadState {
  replayId: string;
  label: string;
  path: string;
  clipCount: number;
  status: MechanicsReviewReplayLoadStatus;
  progress: ReplayLoadProgress | null;
  error: string | null;
}

export interface ActiveMechanicsReview {
  manifest: MechanicsReviewPlaylist;
  sourceUrl: string | null;
  replaysById: Map<string, MechanicsReviewReplay>;
  replayLoadStates: Map<string, MechanicsReviewReplayLoadState>;
  replayLoadCache: Map<string, Promise<ReplayLoadBundle>>;
  currentIndex: number;
  loading: boolean;
  preloading: boolean;
  currentReplayId: string | null;
  currentClip: { startTime: number; endTime: number } | null;
}
