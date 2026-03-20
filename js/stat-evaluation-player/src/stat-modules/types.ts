import { ReplayPlayer } from "subtr-actor-player";
import type {
  FrameRenderInfo,
  ReplayModel,
  ReplayTimelineEvent,
  ReplayTimelineRange,
} from "subtr-actor-player";
import { getStatsFrameForReplayFrame } from "../statsTimeline.ts";
import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsTimeline,
} from "../statsTimeline.ts";
import { playerIdToString } from "../touchOverlay.ts";

export interface StatModuleContext {
  player: ReplayPlayer;
  replay: ReplayModel;
  statsTimeline: StatsTimeline;
  statsFrameLookup: Map<number, StatsFrame>;
  fieldScale: number;
}

export interface StatModule {
  readonly id: string;
  readonly label: string;
  setup(ctx: StatModuleContext): void;
  teardown(): void;
  onBeforeRender(info: FrameRenderInfo): void;
  getTimelineEvents?(ctx: StatModuleContext): ReplayTimelineEvent[];
  getTimelineRanges?(ctx: StatModuleContext): ReplayTimelineRange[];
  renderStats(frameIndex: number, ctx: StatModuleContext): string;
  renderSettings?(ctx: StatModuleContext | null): HTMLElement | null;
  renderFocusedPlayerStats(
    playerId: string,
    frameIndex: number,
    ctx: StatModuleContext,
  ): string;
}

export interface StatModuleRuntime {
  rerenderCurrentState(): void;
}

const MOST_BACK_FORWARD_THRESHOLD_Y = 236.0;

export const RELATIVE_POSITIONING_MODULE_ID = "relative-positioning";

export type Role = "back" | "forward" | "other" | "mid";

export const ROLE_LABELS: Record<Role, string> = {
  back: "Back",
  forward: "Fwd",
  other: "Other",
  mid: "Mid",
};

export function getTeamClass(isTeamZero: boolean): string {
  return isTeamZero ? "team-blue" : "team-orange";
}

type StatCardTone = "blue" | "orange" | "shared";

function renderStatCard(
  name: string,
  bodyHtml: string,
  options: {
    metaHtml?: string;
    tone: StatCardTone;
  },
): string {
  const toneClass = options.tone === "shared"
    ? "shared"
    : options.tone === "blue"
      ? "team-blue"
      : "team-orange";

  return `<div class="player-card ${toneClass}">
    <div class="player-card-header">
      <span class="player-name">${name}</span>
      ${options.metaHtml ?? ""}
    </div>
    ${bodyHtml}
  </div>`;
}

export function renderPlayerCard(
  name: string,
  isTeamZero: boolean,
  bodyHtml: string,
  metaHtml = "",
): string {
  return renderStatCard(name, bodyHtml, {
    metaHtml,
    tone: isTeamZero ? "blue" : "orange",
  });
}

export function renderSharedCard(
  name: string,
  bodyHtml: string,
  metaHtml = "",
): string {
  return renderStatCard(name, bodyHtml, {
    metaHtml,
    tone: "shared",
  });
}

export function getStatsPlayerSnapshot(
  ctx: StatModuleContext,
  frameIndex: number,
  playerId: string,
): PlayerStatsSnapshot | null {
  const statsFrame = getStatsFrameForReplayFrame(ctx.statsFrameLookup, frameIndex);
  if (!statsFrame) return null;

  return statsFrame.players.find(
    (player) => playerIdToString(player.player_id) === playerId,
  ) ?? null;
}

export function getCurrentRole(
  replay: ReplayModel,
  playerId: string,
  frameIndex: number,
): Role {
  const player = replay.players.find((candidate) => candidate.id === playerId);
  if (!player) return "mid";

  const frame = player.frames[frameIndex];
  if (!frame?.position) return "mid";

  const isTeamZero = player.isTeamZero;
  const teamRosterCount = replay.players.filter(
    (candidate) => candidate.isTeamZero === isTeamZero,
  ).length;
  const allYs: number[] = [];
  let normalizedY = 0;

  for (const candidate of replay.players) {
    if (candidate.isTeamZero !== isTeamZero) continue;
    const candidateFrame = candidate.frames[frameIndex];
    if (!candidateFrame?.position) continue;

    const value = isTeamZero
      ? candidateFrame.position.y
      : -candidateFrame.position.y;
    allYs.push(value);
    if (candidate.id === playerId) {
      normalizedY = value;
    }
  }

  if (teamRosterCount < 2 || allYs.length !== teamRosterCount) return "mid";

  const minY = Math.min(...allYs);
  const maxY = Math.max(...allYs);
  const spread = maxY - minY;

  if (spread <= MOST_BACK_FORWARD_THRESHOLD_Y) return "other";

  const nearBack = normalizedY - minY <= MOST_BACK_FORWARD_THRESHOLD_Y;
  const nearFront = maxY - normalizedY <= MOST_BACK_FORWARD_THRESHOLD_Y;

  if (nearBack && !nearFront) return "back";
  if (nearFront && !nearBack) return "forward";
  return "mid";
}
