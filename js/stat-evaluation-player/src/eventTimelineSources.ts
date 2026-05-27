import type { ReplayTimelineEvent, ReplayTimelineRange } from "@rlrml/subtr-actor-player";
import type { StatModule, StatModuleContext } from "./statModules.ts";
import {
  buildMechanicPlaylistEvents,
  buildMechanicTimelineEvents,
  formatMechanicKind,
  getMechanicKinds,
} from "./timelineMarkers.ts";
import { buildMechanicTimelineRanges } from "./timelineRanges.ts";

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

export interface EventTimelineSourceDeps {
  getActiveMechanicTimelineKinds(): Set<string>;
  getActiveTimelineEventSourceIds(): Set<string>;
  getModules(): readonly StatModule[];
  scheduleConfigUrlUpdate(): void;
  setMechanicTimelineKind(kind: string, enabled: boolean): void;
  toggleCapability(id: string, kind: "events", enabled: boolean): void;
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

export function buildEventTimelineSources(
  ctx: StatModuleContext | null,
  deps: EventTimelineSourceDeps,
): EventTimelineSource[] {
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
      active: deps.getActiveTimelineEventSourceIds().has(source.id),
      buildTimelineEvents() {
        return events;
      },
      buildPlaylistEvents() {
        return events;
      },
      setActive(enabled) {
        deps.toggleCapability(source.id, "events", enabled);
      },
    });
  }

  for (const mod of deps.getModules().filter((module) => module.getTimelineEvents)) {
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
      active: deps.getActiveTimelineEventSourceIds().has(mod.id),
      buildTimelineEvents() {
        return events;
      },
      buildPlaylistEvents() {
        return events;
      },
      setActive(enabled) {
        deps.toggleCapability(mod.id, "events", enabled);
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
      active: deps.getActiveTimelineEventSourceIds().has(source.id),
      buildTimelineEvents() {
        return events;
      },
      buildPlaylistEvents() {
        return events;
      },
      setActive(enabled) {
        deps.toggleCapability(source.id, "events", enabled);
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
      active: deps.getActiveMechanicTimelineKinds().has(kind),
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
        deps.setMechanicTimelineKind(kind, enabled);
        deps.scheduleConfigUrlUpdate();
      },
    });
  }

  return sources.sort((left, right) => left.label.localeCompare(right.label));
}
