import type { ReplayPlayerTrack, ReplayTimelineEvent, ReplayTimelineRange } from "@rlrml/player";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import {
  buildMechanicPlaylistEvents,
  buildMechanicTimelineEvents,
  formatMechanicKind,
  getMechanicKinds,
} from "./timelineMarkers.ts";
import { buildMechanicTimelineRanges } from "./timelineRanges.ts";

const DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_IDS = new Set(["module:touch", "module:powerslide"]);
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
    if (count === 0) {
      continue;
    }
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
    if (count === 0) {
      continue;
    }
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
    if (count === 0) {
      continue;
    }
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

  for (const kind of getMechanicKinds(ctx.statsTimeline)) {
    const timelineEvents = buildMechanicTimelineEvents(ctx.statsTimeline, ctx.replay, [kind]);
    const playlistEvents = buildMechanicPlaylistEvents(ctx.statsTimeline, ctx.replay, [kind]);
    const timelineRanges = buildMechanicTimelineRanges(ctx.statsTimeline, ctx.replay, [kind]);
    const count = timelineEvents.length + timelineRanges.length;
    if (count === 0) {
      continue;
    }
    sources.push({
      id: `mechanic:${kind}`,
      playlistId: `mechanic:${kind}`,
      timelineKey: `mechanic:${kind}`,
      timelineId: `mechanic:${kind}`,
      group: "Mechanics",
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
  ].filter((source) => source.events.length > 0);

  const timelineSources = eventSources
    .map((source) => ({
      id: source.playlistId,
      group: source.group,
      label: source.label,
      events: source.buildPlaylistEvents(),
    }))
    .filter((source) => source.events.length > 0);

  return [...replaySources, ...timelineSources];
}

export function getEventPlaylistSelectedSourceIds(
  sources: readonly EventPlaylistSource[],
  activeSourceIds: ReadonlySet<string> | null,
): Set<string> {
  const sourceIds = sources.map((source) => source.id);
  if (activeSourceIds === null) {
    return new Set(sourceIds.filter((id) => !DEFAULT_UNSELECTED_EVENT_PLAYLIST_SOURCE_IDS.has(id)));
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
