import type { ReplayPlayerTrack, ReplayTimelineEvent, ReplayTimelineRange } from "@rlrml/player";
import {
  STATS_EVENT_STREAM_COUNT_TYPES,
  STATS_MECHANIC_EVENT_COUNT_TYPES,
  type StatsEventStreamCountType,
} from "./eventCountDerivation.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import type { StatsEvents } from "./statsTimeline.ts";
import {
  buildMechanicPlaylistEvents,
  buildMechanicTimelineEvents,
  formatMechanicKind,
  getMechanicKinds,
  isVisibleMechanicKind,
} from "./timelineMarkers.ts";
import { buildMechanicTimelineRanges } from "./timelineRanges.ts";

const DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_IDS = new Set(["module:touch", "module:powerslide"]);
const DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_PREFIXES = ["stats-stream:"] as const;
const CURATED_STATS_EVENT_STREAM_IDS = new Set<keyof StatsEvents>([
  "timeline",
  "mechanics",
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
type StatsEventStreamTimelinePresentation = "marker" | "span" | "mixed";

// Marker is the default because a single event row is usually a seek target.
// Only list streams that should also occupy time: state/session/stint windows
// with meaningful bounds are spans, and mixed streams intentionally expose both
// a seekable marker and a range.
export const STATS_EVENT_STREAM_TIMELINE_PRESENTATION_OVERRIDES = {
  possession: "span",
  pressure: "span",
  territorial_pressure: "span",
  movement: "span",
  positioning_activity: "span",
  positioning_possession: "span",
  positioning_field_zone: "span",
  positioning_ball_depth: "span",
  positioning_teammate_role: "span",
  positioning_ball_proximity: "span",
  rotation_player: "span",
  rotation_role_span: "span",
  rotation_depth_span: "span",
  rotation_first_man_stint: "span",
  mechanics: "mixed",
  ceiling_shot: "span",
  wall_aerial: "span",
  wall_aerial_shot: "span",
  center: "span",
  flick: "span",
  musty_flick: "span",
  double_tap: "span",
  fifty_fifty: "mixed",
  kickoff: "span",
  one_timer: "span",
  pass: "span",
  ball_carry: "span",
  controlled_play: "span",
  rush: "mixed",
  wavedash: "span",
  powerslide: "mixed",
  boost_ledger: "span",
  boost_state: "span",
} as const satisfies Partial<
  Record<StatsEventStreamCountType, StatsEventStreamTimelinePresentation>
>;
const statsEventStreamTimelinePresentationOverrides: Partial<
  Record<StatsEventStreamCountType, StatsEventStreamTimelinePresentation>
> = STATS_EVENT_STREAM_TIMELINE_PRESENTATION_OVERRIDES;

export const STATS_EVENT_STREAM_TIMELINE_PRESENTATION = Object.fromEntries(
  STATS_EVENT_STREAM_COUNT_TYPES.map((streamId) => [
    streamId,
    statsEventStreamTimelinePresentationOverrides[streamId] ?? "marker",
  ]),
) as Record<StatsEventStreamCountType, StatsEventStreamTimelinePresentation>;
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

function numberField(record: Record<string, unknown>, key: string): number | undefined {
  const value = record[key];
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function stringField(record: Record<string, unknown>, key: string): string | undefined {
  const value = record[key];
  return typeof value === "string" ? value : undefined;
}

function boolField(record: Record<string, unknown>, key: string): boolean | undefined {
  const value = record[key];
  return typeof value === "boolean" ? value : undefined;
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

function getGenericEventTiming(
  event: Record<string, unknown>,
): { time: number; frame?: number } | null {
  const timing = event.timing;
  if (isRecord(timing)) {
    const time =
      numberField(timing, "end_time") ??
      numberField(timing, "time") ??
      numberField(timing, "start_time");
    if (time != null) {
      return {
        time,
        frame:
          numberField(timing, "end_frame") ??
          numberField(timing, "frame") ??
          numberField(timing, "start_frame"),
      };
    }
  }

  const time =
    numberField(event, "end_time") ??
    numberField(event, "resolve_time") ??
    numberField(event, "time") ??
    numberField(event, "start_time");
  if (time == null) {
    return null;
  }
  return {
    time,
    frame:
      numberField(event, "end_frame") ??
      numberField(event, "resolve_frame") ??
      numberField(event, "frame") ??
      numberField(event, "start_frame"),
  };
}

function getGenericEventSpanTiming(
  event: Record<string, unknown>,
): { startTime: number; endTime: number; startFrame?: number; endFrame?: number } | null {
  const timing = event.timing;
  if (isRecord(timing)) {
    const startTime = numberField(timing, "start_time") ?? numberField(timing, "time");
    const endTime = numberField(timing, "end_time") ?? startTime;
    if (startTime != null && endTime != null) {
      return {
        startTime,
        endTime: Math.max(startTime, endTime),
        startFrame: numberField(timing, "start_frame") ?? numberField(timing, "frame"),
        endFrame: numberField(timing, "end_frame") ?? numberField(timing, "frame"),
      };
    }
  }

  const startTime = numberField(event, "start_time") ?? numberField(event, "time");
  const endTime = numberField(event, "end_time") ?? numberField(event, "resolve_time") ?? startTime;
  if (startTime == null || endTime == null) {
    return null;
  }

  return {
    startTime,
    endTime: Math.max(startTime, endTime),
    startFrame: numberField(event, "start_frame") ?? numberField(event, "frame"),
    endFrame:
      numberField(event, "end_frame") ??
      numberField(event, "resolve_frame") ??
      numberField(event, "frame"),
  };
}

function getGenericEventPlayerId(event: Record<string, unknown>): string | null {
  return (
    remoteIdToString(event.player) ??
    remoteIdToString(event.player_id) ??
    remoteIdToString(event.initiator) ??
    remoteIdToString(event.scorer)
  );
}

function getGenericEventTeam(event: Record<string, unknown>): boolean | null {
  return (
    boolField(event, "is_team_0") ??
    boolField(event, "initiator_is_team_0") ??
    boolField(event, "scoring_team_is_team_0") ??
    boolField(event, "team_is_team_0") ??
    null
  );
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

function formatGenericEventValue(value: string): string {
  return formatMechanicKind(value);
}

function getGenericSpanStateLabel(
  streamId: keyof StatsEvents,
  event: Record<string, unknown>,
): string | null {
  const labeledValues: Array<[string, unknown]> = [
    ["state", event.possession_state],
    ["third", event.field_third],
    ["half", event.field_half],
    ["role", event.current_role_state ?? event.teammate_role],
    ["depth", event.current_depth_state],
    ["speed", event.speed_band],
    ["height", event.height_band],
    ["transaction", event.transaction],
  ];

  if (streamId === "movement") {
    return [event.speed_band, event.height_band]
      .filter((value): value is string => typeof value === "string" && value.length > 0)
      .map(formatGenericEventValue)
      .join(" / ");
  }

  if (streamId === "positioning_activity") {
    if (event.demolished === true) return "Demolished";
    if (event.active === true && event.tracked === true) return "Active";
    if (event.tracked === true) return "Tracked";
    return "Inactive";
  }

  if (streamId === "positioning_ball_depth") {
    const values = [
      ["Behind ball", numberField(event, "behind_ball_fraction")],
      ["Level with ball", numberField(event, "level_with_ball_fraction")],
      ["In front of ball", numberField(event, "in_front_of_ball_fraction")],
    ] as const;
    return values.reduce(
      (best, [label, value]) => (value != null && value > best.value ? { label, value } : best),
      { label: "Ball depth", value: 0 },
    ).label;
  }

  if (streamId === "positioning_ball_proximity") {
    const labels = [];
    if (event.closest_to_ball_absolute === true) labels.push("Closest");
    if (event.closest_to_ball_team === true) labels.push("Team closest");
    if (event.farthest_from_ball === true) labels.push("Farthest");
    return labels.join(" / ") || "Ball proximity";
  }

  if (streamId === "boost_state") {
    const amount = numberField(event, "boost_amount");
    return amount == null ? "Boost state" : `${Math.round(amount)} boost`;
  }

  if (streamId === "boost_ledger") {
    const transaction = typeof event.transaction === "string" ? event.transaction : null;
    const amount = numberField(event, "amount");
    return [
      transaction ? formatGenericEventValue(transaction) : "Boost",
      amount != null ? `${Math.round(amount)}` : null,
    ]
      .filter((part): part is string => !!part)
      .join(" ");
  }

  if (streamId === "rush") {
    const attackers = numberField(event, "attackers");
    const defenders = numberField(event, "defenders");
    return attackers != null && defenders != null ? `${attackers}v${defenders}` : "Rush";
  }

  for (const [, value] of labeledValues) {
    if (typeof value === "string" && value.length > 0) {
      return formatGenericEventValue(value);
    }
  }

  return null;
}

function buildGenericStatsEventTimelineEvents(
  ctx: StatModuleContext,
  streamId: keyof StatsEvents,
  events: readonly unknown[],
): ReplayTimelineEvent[] {
  const streamLabel = formatMechanicKind(streamId);
  const playerNames = new Map(ctx.replay.players.map((player) => [player.id, player.name]));

  return events.flatMap((event, index) => {
    if (!isRecord(event)) {
      return [];
    }
    const timing = getGenericEventTiming(event);
    if (!timing) {
      return [];
    }

    const playerId = getGenericEventPlayerId(event);
    const playerName = playerId ? (playerNames.get(playerId) ?? playerId) : null;
    const isTeamZero = getGenericEventTeam(event);
    const eventId =
      stringField(event, "id") ?? `${streamId}:${timing.frame ?? timing.time}:${index}`;

    return [
      {
        id: `stats-stream:${eventId}`,
        time: ctx.replay.frames[timing.frame ?? -1]?.time ?? timing.time,
        frame: timing.frame,
        kind: streamId,
        label: playerName ? `${playerName} ${streamLabel.toLowerCase()}` : streamLabel,
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
  streamId: keyof StatsEvents,
  events: readonly unknown[],
): ReplayTimelineRange[] {
  const streamLabel = formatMechanicKind(streamId);
  const playerNames = new Map(ctx.replay.players.map((player) => [player.id, player.name]));

  return events
    .flatMap((event, index) => {
      if (!isRecord(event)) {
        return [];
      }
      const timing = getGenericEventSpanTiming(event);
      if (!timing) {
        return [];
      }

      const isTeamZero = getGenericEventTeam(event);
      const playerId = getGenericEventPlayerId(event);
      const playerName = playerId ? (playerNames.get(playerId) ?? playerId) : null;
      const teamLabel = isTeamZero == null ? null : isTeamZero ? "Blue" : "Orange";
      const stateLabel = getGenericSpanStateLabel(streamId, event);
      const subjectLabel = playerName ?? teamLabel;
      const eventId =
        stringField(event, "id") ??
        `${streamId}:${timing.startFrame ?? timing.startTime}:${timing.endFrame ?? timing.endTime}:${index}`;

      return [
        {
          id: `stats-stream:${eventId}`,
          startTime: ctx.replay.frames[timing.startFrame ?? -1]?.time ?? timing.startTime,
          endTime: Math.max(
            ctx.replay.frames[timing.startFrame ?? -1]?.time ?? timing.startTime,
            ctx.replay.frames[timing.endFrame ?? -1]?.time ?? timing.endTime,
          ),
          lane: playerId ? `stats-stream:${streamId}:${playerId}` : `stats-stream:${streamId}`,
          laneLabel: playerName ?? streamLabel,
          label: [subjectLabel, stateLabel ?? streamLabel]
            .filter((part): part is string => !!part)
            .join(" "),
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
    const events = ctx.statsTimeline.events[streamId] ?? [];
    if (CURATED_STATS_EVENT_STREAM_IDS.has(streamId) && events.length > 0) {
      return [];
    }

    const presentation = STATS_EVENT_STREAM_TIMELINE_PRESENTATION[streamId];
    const hasTimelineRanges = presentation === "span" || presentation === "mixed";
    const hasTimelineEvents = presentation === "marker" || presentation === "mixed";
    const timelineRanges = hasTimelineRanges
      ? buildGenericStatsEventTimelineRanges(ctx, streamId, events)
      : [];
    const timelineEvents = hasTimelineEvents
      ? buildGenericStatsEventTimelineEvents(ctx, streamId, events)
      : [];

    return [
      {
        id: `stats-stream:${streamId}`,
        playlistId: `stats-stream:${streamId}`,
        timelineKey: `stats-stream:${streamId}`,
        timelineId: `stats-stream:${streamId}`,
        group: "Event streams",
        label: formatMechanicKind(streamId),
        count: timelineEvents.length + timelineRanges.length,
        active: activeTimelineEventSourceIds.has(`stats-stream:${streamId}`),
        buildTimelineEvents() {
          return timelineEvents;
        },
        buildPlaylistEvents() {
          return timelineEvents.length > 0
            ? timelineEvents
            : buildGenericStatsEventTimelineEvents(ctx, streamId, events);
        },
        buildTimelineRanges: hasTimelineRanges ? () => timelineRanges : undefined,
        setActive(enabled) {
          toggleEventSource(`stats-stream:${streamId}`, enabled);
        },
      },
    ];
  });
}

function getEventTimelineMechanicKinds(ctx: StatModuleContext): string[] {
  return [
    ...new Set([
      ...STATS_MECHANIC_EVENT_COUNT_TYPES.filter(isVisibleMechanicKind),
      ...getMechanicKinds(ctx.statsTimeline),
    ]),
  ].sort((left, right) => formatMechanicKind(left).localeCompare(formatMechanicKind(right)));
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

  for (const kind of getEventTimelineMechanicKinds(ctx)) {
    const timelineEvents = buildMechanicTimelineEvents(ctx.statsTimeline, ctx.replay, [kind]);
    const playlistEvents = buildMechanicPlaylistEvents(ctx.statsTimeline, ctx.replay, [kind]);
    const timelineRanges = buildMechanicTimelineRanges(ctx.statsTimeline, ctx.replay, [kind]);
    const count = timelineEvents.length + timelineRanges.length;
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
        return playlistEvents;
      },
      buildTimelineRanges() {
        return timelineRanges;
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
