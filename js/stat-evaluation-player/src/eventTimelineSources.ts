import type { ReplayPlayerTrack, ReplayTimelineEvent, ReplayTimelineRange } from "@rlrml/player";
import { STATS_EVENT_STREAM_COUNT_TYPES } from "./eventCountDerivation.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import type { Event } from "./statsTimeline.ts";
import { statsEventEnvelopes } from "./statsTimeline.ts";
import {
  EVENT_TYPE_TIMELINE_BUILDERS,
  EVENT_TYPE_TIMELINE_KINDS,
  formatMechanicKind,
} from "./timelineMarkers.ts";

const DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_IDS = new Set(["module:touch", "module:powerslide"]);
const DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_PREFIXES = ["stats-stream:"] as const;
const CURATED_STATS_EVENT_STREAM_IDS = new Set<string>([
  "timeline",
  "backboard",
  "ceiling_shot",
  "wall_aerial",
  "wall_aerial_shot",
  "center",
  "flick",
  "musty_flick",
  "dodge_reset",
  "double_tap",
  "fifty_fifty",
  "one_timer",
  "pass",
  "ball_carry",
  "rush",
  "dodge",
  "speed_flip",
  "half_flip",
  "half_volley",
  "wavedash",
  "whiff",
  "powerslide",
  "touch",
  "bump",
]);
const SPAN_BASED_STATS_EVENT_STREAM_IDS = new Set<string>([
  "possession",
  "ball_half",
  "territorial_pressure",
  "controlled_play",
  "player_activity",
  "field_third",
  "field_half",
  "ball_depth",
  "depth_role",
  "ball_proximity",
  "rotation_role",
  "rush",
]);
const EVENT_PLAYLIST_PLAYER_COLORS = [
  "#3b82f6",
  "#06b6d4",
  "#22c55e",
  "#a855f7",
  "#f97316",
  "#ef4444",
  "#f59e0b",
  "#ec4899",
];
const EVENT_PLAYLIST_NEUTRAL_COLOR = "#d1d9e0";

interface EventWindowSourceDefinition {
  id: string;
  label: string;
  buildEvents(ctx: StatModuleContext): ReplayTimelineEvent[];
}

export interface EventTimelineSource {
  id: string;
  playlistId: string;
  timelineKey: string;
  timelineId: string;
  group: string;
  label: string;
  count: number;
  active: boolean;
  buildTimelineEvents(): ReplayTimelineEvent[];
  buildPlaylistEvents(): ReplayTimelineEvent[];
  buildTimelineRanges?(): ReplayTimelineRange[];
  setActive(enabled: boolean): void;
}

export interface EventPlaylistSource {
  id: string;
  group: string;
  label: string;
  events: ReplayTimelineEvent[];
}

export interface EventPlaylistItem {
  key: string;
  sourceId: string;
  sourceLabel: string;
  event: ReplayTimelineEvent;
  color: string;
}

export interface EventTimelineSourceOptions {
  ctx: StatModuleContext | null;
  modules: readonly StatModule[];
  activeTimelineEventSourceIds: ReadonlySet<string>;
  activeMechanicTimelineKinds: ReadonlySet<string>;
  toggleEventSource(id: string, enabled: boolean): void;
  setMechanicTimelineKind(kind: string, enabled: boolean): void;
}

export interface EventPlaylistItemOptions {
  sources: EventPlaylistSource[];
  activeSourceIds: ReadonlySet<string> | null;
  replayPlayers: readonly ReplayPlayerTrack[];
}

const REPLAY_EVENT_SOURCE_DEFINITIONS: EventWindowSourceDefinition[] = [
  {
    id: "core",
    label: "Shots, saves, assists",
    buildEvents(ctx) {
      return ctx.replay.timelineEvents.filter(
        (event) => event.kind === "shot" || event.kind === "save" || event.kind === "assist",
      );
    },
  },
  {
    id: "demo",
    label: "Demos",
    buildEvents(ctx) {
      return ctx.replay.timelineEvents.filter((event) => event.kind === "demo");
    },
  },
];

const EXTRA_EVENT_SOURCE_DEFINITIONS: EventWindowSourceDefinition[] = [];

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function remoteIdToString(value: unknown): string | null {
  if (typeof value === "string" && value.length > 0) {
    return value;
  }
  if (!isRecord(value)) {
    return null;
  }

  const [kind, id] = Object.entries(value)[0] ?? [];
  if (!kind || id == null) {
    return null;
  }
  if (typeof id === "string" || typeof id === "number") {
    return `${kind}:${id}`;
  }
  return `${kind}:${JSON.stringify(id)}`;
}

function eventStreamShortLabel(streamId: string): string {
  return (
    streamId
      .split(/[_-]+/)
      .filter((part) => part.length > 0)
      .map((part) => part.slice(0, 1).toUpperCase())
      .join("")
      .slice(0, 3) || "E"
  );
}

function formatSeconds(value: unknown): string | null {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return null;
  }

  const digits = Math.abs(value) < 1 ? 2 : 1;
  const seconds = value
    .toFixed(digits)
    .replace(/\.0+$/, "")
    .replace(/(\.\d*[1-9])0+$/, "$1");
  return `${seconds}s`;
}

function titleCaseValue(value: unknown): string | null {
  if (typeof value !== "string" || value.length === 0) {
    return null;
  }

  return value
    .split(/[_-]+/)
    .filter((part) => part.length > 0)
    .map((part) => `${part.slice(0, 1).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function formatFieldHalf(value: unknown): string | null {
  if (value === "team_zero_side") return "Blue side";
  if (value === "team_one_side") return "Orange side";
  if (value === "neutral") return "Neutral zone";
  return titleCaseValue(value);
}

function formatFieldThird(value: unknown): string | null {
  const label = titleCaseValue(value);
  return label ? `${label.toLowerCase()} third` : null;
}

function payloadRecord(event: Event): Record<string, unknown> {
  return isRecord(event.payload.payload) ? event.payload.payload : {};
}

function joinEventDetails(parts: Array<string | null>): string {
  return parts.filter((part): part is string => Boolean(part)).join(" | ");
}

function formatGenericStatsEventLabel({
  event,
  playerName,
  streamLabel,
  teamLabel,
}: {
  event: Event;
  playerName: string | null;
  streamLabel: string;
  teamLabel: string | null;
}): string {
  const payload = payloadRecord(event);
  const duration = formatSeconds(payload.duration);

  if (event.payload.kind === "ball_half") {
    const half = formatFieldHalf(payload.field_half);
    const state =
      payload.active === false
        ? "Ball half inactive"
        : half
          ? `Ball on ${half.toLowerCase()}`
          : null;
    return joinEventDetails([state ?? streamLabel, duration]);
  }

  if (event.payload.kind === "territorial_pressure") {
    const reason = titleCaseValue(payload.end_reason);
    const main = `${teamLabel ?? ""} territorial pressure`.trim();
    return joinEventDetails([reason ? `${main} ended: ${reason.toLowerCase()}` : main, duration]);
  }

  if (event.payload.kind === "possession") {
    const state = titleCaseValue(payload.possession_state);
    const third = formatFieldThird(payload.field_third);
    const main = state ? `${state} possession` : streamLabel;
    return joinEventDetails([main, third, duration]);
  }

  if (event.payload.kind === "controlled_play") {
    const prefix = playerName ? `${playerName} controlled play` : streamLabel;
    return joinEventDetails([prefix, duration]);
  }

  if (event.payload.kind === "player_activity") {
    const state = titleCaseValue(payload.state);
    const prefix = playerName ? `${playerName} positioning` : streamLabel;
    return joinEventDetails([state ? `${prefix} ${state.toLowerCase()}` : prefix, duration]);
  }

  if (event.payload.kind === "field_third") {
    const zone = titleCaseValue(payload.state);
    const prefix = playerName ? `${playerName} positioning` : streamLabel;
    return joinEventDetails([zone ? `${prefix} in ${zone.toLowerCase()} third` : prefix, duration]);
  }

  if (event.payload.kind === "field_half") {
    const half = titleCaseValue(payload.state);
    const prefix = playerName ? `${playerName} positioning` : streamLabel;
    return joinEventDetails([half ? `${prefix} in ${half.toLowerCase()} half` : prefix, duration]);
  }

  if (event.payload.kind === "ball_depth") {
    const depth = titleCaseValue(payload.state);
    const prefix = playerName ? `${playerName} ball depth` : streamLabel;
    return joinEventDetails([depth ? `${prefix}: ${depth.toLowerCase()}` : prefix, duration]);
  }

  if (event.payload.kind === "depth_role") {
    const role = titleCaseValue(payload.state);
    const prefix = playerName ? `${playerName} depth role` : streamLabel;
    return joinEventDetails([role ? `${prefix}: ${role.toLowerCase()}` : prefix, duration]);
  }

  if (event.payload.kind === "rotation_role") {
    const role = titleCaseValue(payload.state);
    const prefix = playerName ? `${playerName} rotation` : streamLabel;
    return joinEventDetails([role ? `${prefix}: ${role.toLowerCase()}` : prefix, duration]);
  }

  return playerName ? `${playerName} ${streamLabel.toLowerCase()}` : streamLabel;
}

function buildGenericStatsEventTimelineEvents(
  ctx: StatModuleContext,
  streamId: string,
  events: readonly Event[],
): ReplayTimelineEvent[] {
  const streamLabel = formatMechanicKind(streamId);
  const playerNames = new Map(ctx.replay.players.map((player) => [player.id, player.name]));

  return events.flatMap((event, index) => {
    const timing =
      event.meta.timing.type === "moment"
        ? { time: event.meta.timing.time, frame: event.meta.timing.frame }
        : { time: event.meta.timing.end_time, frame: event.meta.timing.end_frame };
    const playerId = remoteIdToString(event.meta.primary_player);
    const playerName = playerId ? (playerNames.get(playerId) ?? playerId) : null;
    const isTeamZero = event.meta.team_is_team_0 ?? null;
    const teamLabel = isTeamZero == null ? null : isTeamZero ? "Blue" : "Orange";
    const eventId = event.meta.id || `${streamId}:${timing.frame ?? timing.time}:${index}`;

    return [
      {
        id: `stats-stream:${eventId}`,
        time: ctx.replay.frames[timing.frame ?? -1]?.time ?? timing.time,
        frame: timing.frame,
        kind: streamId,
        label: formatGenericStatsEventLabel({
          event,
          playerName,
          streamLabel,
          teamLabel,
        }),
        shortLabel: eventStreamShortLabel(streamId),
        playerId,
        playerName,
        isTeamZero,
        color:
          isTeamZero == null
            ? EVENT_PLAYLIST_NEUTRAL_COLOR
            : isTeamZero
              ? EVENT_PLAYLIST_PLAYER_COLORS[0]
              : EVENT_PLAYLIST_PLAYER_COLORS[4],
      },
    ];
  });
}

function buildGenericStatsEventTimelineRanges(
  ctx: StatModuleContext,
  streamId: string,
  events: readonly Event[],
): ReplayTimelineRange[] {
  const streamLabel = formatMechanicKind(streamId);
  const playerNames = new Map(ctx.replay.players.map((player) => [player.id, player.name]));

  return events
    .flatMap((event, index) => {
      if (event.meta.timing.type !== "span") {
        return [];
      }
      const timing = {
        startTime: event.meta.timing.start_time,
        endTime: event.meta.timing.end_time,
        startFrame: event.meta.timing.start_frame,
        endFrame: event.meta.timing.end_frame,
      };

      const isTeamZero = event.meta.team_is_team_0 ?? null;
      const teamLabel = isTeamZero == null ? null : isTeamZero ? "Blue" : "Orange";
      const playerId = remoteIdToString(event.meta.primary_player);
      const playerName = playerId ? (playerNames.get(playerId) ?? playerId) : null;
      const eventId =
        event.meta.id ||
        `${streamId}:${timing.startFrame ?? timing.startTime}:${timing.endFrame ?? timing.endTime}:${index}`;
      const label = playerName
        ? `${playerName} ${streamLabel.toLowerCase()}`
        : teamLabel
          ? `${teamLabel} ${streamLabel.toLowerCase()}`
          : streamLabel;

      return [
        {
          id: `stats-stream:${eventId}`,
          startTime: ctx.replay.frames[timing.startFrame ?? -1]?.time ?? timing.startTime,
          endTime: Math.max(
            ctx.replay.frames[timing.startFrame ?? -1]?.time ?? timing.startTime,
            ctx.replay.frames[timing.endFrame ?? -1]?.time ?? timing.endTime,
          ),
          lane: `stats-stream:${streamId}`,
          laneLabel: streamLabel,
          label,
          shortLabel: eventStreamShortLabel(streamId),
          isTeamZero,
          color:
            isTeamZero == null
              ? EVENT_PLAYLIST_NEUTRAL_COLOR
              : isTeamZero
                ? EVENT_PLAYLIST_PLAYER_COLORS[0]
                : EVENT_PLAYLIST_PLAYER_COLORS[4],
        },
      ];
    })
    .sort((left, right) => {
      if (left.startTime !== right.startTime) {
        return left.startTime - right.startTime;
      }
      return (left.id ?? "").localeCompare(right.id ?? "");
    });
}

function buildGenericStatsEventSources(
  ctx: StatModuleContext,
  activeTimelineEventSourceIds: ReadonlySet<string>,
  toggleEventSource: (id: string, enabled: boolean) => void,
): EventTimelineSource[] {
  return STATS_EVENT_STREAM_COUNT_TYPES.flatMap((streamId) => {
    const events = statsEventEnvelopes(ctx.statsTimeline).filter(
      (event) => event.meta.stream === streamId,
    );
    if (CURATED_STATS_EVENT_STREAM_IDS.has(streamId) && events.length > 0) {
      return [];
    }

    const isSpanBased = SPAN_BASED_STATS_EVENT_STREAM_IDS.has(streamId);
    const timelineRanges = isSpanBased
      ? buildGenericStatsEventTimelineRanges(ctx, streamId, events)
      : [];
    const timelineEvents = buildGenericStatsEventTimelineEvents(ctx, streamId, events);

    return [
      {
        id: `stats-stream:${streamId}`,
        playlistId: `stats-stream:${streamId}`,
        timelineKey: `stats-stream:${streamId}`,
        timelineId: `stats-stream:${streamId}`,
        group: "Event streams",
        label: formatMechanicKind(streamId),
        count: isSpanBased ? timelineRanges.length : timelineEvents.length,
        active: activeTimelineEventSourceIds.has(`stats-stream:${streamId}`),
        buildTimelineEvents() {
          return isSpanBased ? [] : timelineEvents;
        },
        buildPlaylistEvents() {
          return timelineEvents;
        },
        buildTimelineRanges: isSpanBased ? () => timelineRanges : undefined,
        setActive(enabled) {
          toggleEventSource(`stats-stream:${streamId}`, enabled);
        },
      },
    ];
  });
}

function getEventTimelineMechanicKinds(): string[] {
  return [...EVENT_TYPE_TIMELINE_KINDS].sort((left, right) =>
    formatMechanicKind(left).localeCompare(formatMechanicKind(right)),
  );
}

export function getEventTimelineSources({
  ctx,
  modules,
  activeTimelineEventSourceIds,
  activeMechanicTimelineKinds,
  toggleEventSource,
  setMechanicTimelineKind,
}: EventTimelineSourceOptions): EventTimelineSource[] {
  if (!ctx) {
    return [];
  }

  const sources: EventTimelineSource[] = [];
  for (const source of REPLAY_EVENT_SOURCE_DEFINITIONS) {
    const events = source.buildEvents(ctx);
    const count = events.length;
    sources.push({
      id: source.id,
      playlistId: `replay:${source.id}`,
      timelineKey: `events:${source.id}`,
      timelineId: `events:${source.id}`,
      group: "Replay",
      label: source.label,
      count,
      active: activeTimelineEventSourceIds.has(source.id),
      buildTimelineEvents() {
        return events;
      },
      buildPlaylistEvents() {
        return events;
      },
      setActive(enabled) {
        toggleEventSource(source.id, enabled);
      },
    });
  }

  for (const mod of modules.filter((module) => module.getTimelineEvents)) {
    const events = mod.getTimelineEvents?.(ctx) ?? [];
    const count = events.length;
    sources.push({
      id: mod.id,
      playlistId: `module:${mod.id}`,
      timelineKey: `module:${mod.id}`,
      timelineId: `module:${mod.id}`,
      group: "Stats",
      label: mod.label,
      count,
      active: activeTimelineEventSourceIds.has(mod.id),
      buildTimelineEvents() {
        return events;
      },
      buildPlaylistEvents() {
        return events;
      },
      setActive(enabled) {
        toggleEventSource(mod.id, enabled);
      },
    });
  }

  for (const source of EXTRA_EVENT_SOURCE_DEFINITIONS) {
    const events = source.buildEvents(ctx);
    const count = events.length;
    sources.push({
      id: source.id,
      playlistId: `extra:${source.id}`,
      timelineKey: `extra:${source.id}`,
      timelineId: `extra:${source.id}`,
      group: "Stats",
      label: source.label,
      count,
      active: activeTimelineEventSourceIds.has(source.id),
      buildTimelineEvents() {
        return events;
      },
      buildPlaylistEvents() {
        return events;
      },
      setActive(enabled) {
        toggleEventSource(source.id, enabled);
      },
    });
  }

  sources.push(
    ...buildGenericStatsEventSources(ctx, activeTimelineEventSourceIds, toggleEventSource),
  );

  for (const kind of getEventTimelineMechanicKinds()) {
    const timelineEvents = EVENT_TYPE_TIMELINE_BUILDERS[kind]!(ctx.statsTimeline, ctx.replay);
    const count = timelineEvents.length;
    sources.push({
      id: `mechanic:${kind}`,
      playlistId: `mechanic:${kind}`,
      timelineKey: `mechanic:${kind}`,
      timelineId: `mechanic:${kind}`,
      group: "Event types",
      label: formatMechanicKind(kind),
      count,
      active: activeMechanicTimelineKinds.has(kind),
      buildTimelineEvents() {
        return timelineEvents;
      },
      buildPlaylistEvents() {
        return timelineEvents;
      },
      setActive(enabled) {
        setMechanicTimelineKind(kind, enabled);
      },
    });
  }

  return sources.sort((left, right) => left.label.localeCompare(right.label));
}

export function getEventPlaylistSources(
  ctx: StatModuleContext | null,
  eventSources: readonly EventTimelineSource[],
): EventPlaylistSource[] {
  if (!ctx) {
    return [];
  }

  const replaySources = [
    {
      id: "replay:goals",
      group: "Replay",
      label: "Goals",
      events: ctx.replay.timelineEvents.filter((event) => event.kind === "goal"),
    },
  ];

  const timelineSources = eventSources.map((source) => ({
    id: source.playlistId,
    group: source.group,
    label: source.label,
    events: source.buildPlaylistEvents(),
  }));

  return [...replaySources, ...timelineSources];
}

export function getEventPlaylistSelectedSourceIds(
  sources: readonly EventPlaylistSource[],
  activeSourceIds: ReadonlySet<string> | null,
): Set<string> {
  const sourceIds = sources.map((source) => source.id);
  if (activeSourceIds === null) {
    return new Set(
      sourceIds.filter(
        (id) =>
          !DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_IDS.has(id) &&
          !DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_PREFIXES.some((prefix) =>
            id.startsWith(prefix),
          ),
      ),
    );
  }
  return new Set(sourceIds.filter((id) => activeSourceIds.has(id)));
}

function getEventPlaylistPlayerColor(
  event: ReplayTimelineEvent,
  replayPlayers: readonly ReplayPlayerTrack[],
): string {
  const playerId = event.playerId ?? null;
  const playerIndex = playerId ? replayPlayers.findIndex((player) => player.id === playerId) : -1;
  if (playerIndex >= 0) {
    return EVENT_PLAYLIST_PLAYER_COLORS[playerIndex % EVENT_PLAYLIST_PLAYER_COLORS.length]!;
  }
  return event.color ?? EVENT_PLAYLIST_NEUTRAL_COLOR;
}

export function buildEventPlaylistItems({
  sources,
  activeSourceIds,
  replayPlayers,
}: EventPlaylistItemOptions): EventPlaylistItem[] {
  const selectedSourceIds = getEventPlaylistSelectedSourceIds(sources, activeSourceIds);
  return sources
    .filter((source) => selectedSourceIds.has(source.id))
    .flatMap((source) =>
      source.events.map((event, index) => ({
        key: `${source.id}:${event.id ?? `${event.kind}:${event.time}:${index}`}`,
        sourceId: source.id,
        sourceLabel: source.label,
        event,
        color: getEventPlaylistPlayerColor(event, replayPlayers),
      })),
    )
    .sort((left, right) => {
      if (left.event.time !== right.event.time) {
        return left.event.time - right.event.time;
      }
      return (left.event.label ?? left.sourceLabel).localeCompare(
        right.event.label ?? right.sourceLabel,
      );
    });
}
