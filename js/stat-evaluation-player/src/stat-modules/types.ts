import type { StatsReplayPlayer } from "../statsReplayPlayer.ts";
import type {
  FrameRenderInfo,
  BoostPickupAnimationPickup,
  ReplayModel,
  ReplayTimelineEvent,
  ReplayTimelineGraph,
  ReplayTimelineRange,
} from "@rlrml/player";
import { getStatsFrameForReplayFrame } from "../statsTimeline.ts";
import type { PlayerStatsSnapshot, StatsFrameLookup, StatsTimeline } from "../statsTimeline.ts";
import { playerIdToString } from "../touchOverlay.ts";

export interface StatModuleContext {
  player: StatsReplayPlayer;
  replay: ReplayModel;
  statsTimeline: StatsTimeline;
  statsFrameLookup: StatsFrameLookup;
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
  getTimelineGraphs?(ctx: StatModuleContext): ReplayTimelineGraph[];
  getConfig?(): unknown;
  applyConfig?(config: unknown): void;
  includeBoostPickupAnimationPickup?(pickup: BoostPickupAnimationPickup): boolean;
  renderStats(frameIndex: number, ctx: StatModuleContext): string;
  renderSettings?(ctx: StatModuleContext | null): HTMLElement | null;
  renderFocusedPlayerStats(playerId: string, frameIndex: number, ctx: StatModuleContext): string;
}

export interface StatModuleRuntime {
  rerenderCurrentState(): void;
  refreshTimelineRanges?(): void;
  requestConfigSync?(): void;
}

const MOST_BACK_FORWARD_THRESHOLD_Y = 236.0;

export const RELATIVE_POSITIONING_MODULE_ID = "relative-positioning";

export type DepthRole = "last" | "upfield" | "level" | "mid";

export const DEPTH_ROLE_LABELS: Record<DepthRole, string> = {
  last: "Last",
  upfield: "Upfield",
  level: "Level",
  mid: "Mid",
};

export type Role = DepthRole;
export const ROLE_LABELS = DEPTH_ROLE_LABELS;

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
  const toneClass =
    options.tone === "shared" ? "shared" : options.tone === "blue" ? "team-blue" : "team-orange";

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

export function renderGroupedPlayerCards(
  players: readonly PlayerStatsSnapshot[],
  renderPlayer: (player: PlayerStatsSnapshot) => string,
): string {
  return `<div class="player-team-stack">${([true, false] as const)
    .map((isTeamZero) => {
      const teamPlayers = players.filter((player) => player.is_team_0 === isTeamZero);
      if (teamPlayers.length === 0) {
        return "";
      }

      const teamName = isTeamZero ? "Blue" : "Orange";
      return `<section class="player-team-group ${getTeamClass(isTeamZero)}">
        <div class="player-team-header">
          <h3>${teamName} team</h3>
          <span>${teamPlayers.length} player${teamPlayers.length === 1 ? "" : "s"}</span>
        </div>
        <div class="player-stats-grid">
          ${teamPlayers.map(renderPlayer).join("")}
        </div>
      </section>`;
    })
    .join("")}</div>`;
}

export function renderSharedCard(name: string, bodyHtml: string, metaHtml = ""): string {
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

  return (
    statsFrame.players.find((player) => playerIdToString(player.player_id) === playerId) ?? null
  );
}

export function getCurrentDepthRole(
  replay: ReplayModel,
  playerId: string,
  frameIndex: number,
): DepthRole {
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

    const value = isTeamZero ? candidateFrame.position.y : -candidateFrame.position.y;
    allYs.push(value);
    if (candidate.id === playerId) {
      normalizedY = value;
    }
  }

  if (teamRosterCount < 2 || allYs.length !== teamRosterCount) return "mid";

  const minY = Math.min(...allYs);
  const maxY = Math.max(...allYs);
  const spread = maxY - minY;

  if (spread <= MOST_BACK_FORWARD_THRESHOLD_Y) return "level";

  const nearBack = normalizedY - minY <= MOST_BACK_FORWARD_THRESHOLD_Y;
  const nearFront = maxY - normalizedY <= MOST_BACK_FORWARD_THRESHOLD_Y;

  if (nearBack && !nearFront) return "last";
  if (nearFront && !nearBack) return "upfield";
  return "mid";
}

export const getCurrentRole = getCurrentDepthRole;
