import type {
  ReplayBoostPadSize,
  ReplayModel,
  ReplayTimelineRange,
} from "@rlrml/player";
import type { StatsTimeline } from "./statsTimeline.ts";
import type { BoostPickupActivity } from "./generated/BoostPickupActivity.ts";
import type { BoostPickupComparison } from "./generated/BoostPickupComparison.ts";
import type { BoostPickupFieldHalf } from "./generated/BoostPickupFieldHalf.ts";
import type { BoostPickupPadType } from "./generated/BoostPickupPadType.ts";

const BOOST_PICKUP_TICK_SECONDS = 0.08;
const BLUE_TIMELINE_COLOR = "#3b82f6";
const ORANGE_TIMELINE_COLOR = "#f59e0b";
const BOOST_PICKUP_COLORS: Record<ReplayBoostPadSize, string> = {
  big: "rgba(245, 158, 11, 0.92)",
  small: "rgba(52, 211, 153, 0.86)",
};
const BOOST_PICKUP_COMPARISON_COLORS: Record<BoostPickupComparison, string> = {
  both: "rgba(52, 211, 153, 0.86)",
  ghost: "rgba(239, 68, 68, 0.9)",
  missed: "rgba(59, 130, 246, 0.9)",
};

export interface BoostPickupTimelineRangeOptions {
  sizes?: Iterable<ReplayBoostPadSize>;
  padTypes?: Iterable<BoostPickupPadType>;
  comparisons?: Iterable<BoostPickupComparison>;
  activities?: Iterable<BoostPickupActivity>;
  fieldHalves?: Iterable<BoostPickupFieldHalf>;
  playerIds?: Iterable<string>;
}

export function buildBoostPickupTimelineRanges(
  timeline: StatsTimeline,
  replay?: ReplayModel,
  options: BoostPickupTimelineRangeOptions = {},
): ReplayTimelineRange[] {
  const events = timeline.events?.boost_pickups ?? [];
  if (events.length === 0 && replay) {
    return buildReplayBoostPickupTimelineRanges(replay, options);
  }

  const enabledPadTypes = padTypesFromOptions(options);
  const enabledComparisons = new Set<BoostPickupComparison>(options.comparisons ?? ["both"]);
  const enabledActivities = new Set<BoostPickupActivity>(
    options.activities ?? ["active", "inactive", "unknown"],
  );
  const enabledFieldHalves = new Set<BoostPickupFieldHalf>(
    options.fieldHalves ?? ["own", "opponent", "unknown"],
  );
  const enabledPlayerIds = options.playerIds ? new Set(options.playerIds) : null;
  if (
    enabledPadTypes.size === 0 ||
    enabledComparisons.size === 0 ||
    enabledActivities.size === 0 ||
    enabledFieldHalves.size === 0 ||
    enabledPlayerIds?.size === 0
  ) {
    return [];
  }

  const playerNames = new Map((replay?.players ?? []).map((player) => [player.id, player.name]));
  return events
    .filter((event) => {
      const playerId = remoteIdToString(event.player_id as Record<string, unknown>);
      return (
        enabledPadTypes.has(event.pad_type) &&
        enabledComparisons.has(event.comparison) &&
        enabledActivities.has(event.activity) &&
        enabledFieldHalves.has(event.field_half) &&
        (!enabledPlayerIds || enabledPlayerIds.has(playerId))
      );
    })
    .map((event, index): ReplayTimelineRange => {
      const playerId = remoteIdToString(event.player_id as Record<string, unknown>);
      const playerName = playerNames.get(playerId) ?? playerId;
      const startTime = Math.max(0, getReplayFrameTime(replay, event.frame, event.time));
      const comparisonLabel = formatBoostPickupComparison(event.comparison);
      const padLabel = formatBoostPickupPadType(event.pad_type);
      return {
        id: `boost-pickup:${event.comparison}:${event.frame}:${playerId}:${index}`,
        startTime,
        endTime: Math.max(startTime + BOOST_PICKUP_TICK_SECONDS, startTime),
        lane: "boost-pickups",
        laneLabel: "Boost Pickups",
        label: `${playerName} ${comparisonLabel} ${padLabel} boost pickup`,
        shortLabel: boostPickupShortLabel(event.comparison, event.pad_type),
        color:
          teamTimelineColor(event.is_team_0) ??
          (event.comparison === "both"
            ? event.pad_type === "big"
              ? BOOST_PICKUP_COLORS.big
              : event.pad_type === "small"
                ? BOOST_PICKUP_COLORS.small
                : BOOST_PICKUP_COMPARISON_COLORS.both
            : BOOST_PICKUP_COMPARISON_COLORS[event.comparison]),
        isTeamZero: event.is_team_0,
      };
    })
    .sort(compareTimelineRanges);
}

function buildReplayBoostPickupTimelineRanges(
  replay: ReplayModel,
  options: BoostPickupTimelineRangeOptions = {},
): ReplayTimelineRange[] {
  const enabledPadTypes = padTypesFromOptions(options);
  const enabledComparisons = new Set<BoostPickupComparison>(options.comparisons ?? ["both"]);
  const enabledActivities = new Set<BoostPickupActivity>(
    options.activities ?? ["active", "inactive", "unknown"],
  );
  const enabledFieldHalves = new Set<BoostPickupFieldHalf>(
    options.fieldHalves ?? ["own", "opponent", "unknown"],
  );
  const enabledPlayerIds = options.playerIds ? new Set(options.playerIds) : null;
  if (
    enabledPadTypes.size === 0 ||
    !enabledComparisons.has("both") ||
    !enabledActivities.has("unknown") ||
    !enabledFieldHalves.has("unknown") ||
    enabledPlayerIds?.size === 0
  ) {
    return [];
  }

  const playerTeams = new Map(replay.players.map((player) => [player.id, player.isTeamZero]));
  const ranges: ReplayTimelineRange[] = [];

  for (const pad of replay.boostPads) {
    if (!enabledPadTypes.has(pad.size)) {
      continue;
    }

    for (let eventIndex = 0; eventIndex < pad.events.length; eventIndex += 1) {
      const event = pad.events[eventIndex]!;
      if (event.available || !Number.isFinite(event.time)) {
        continue;
      }
      if (enabledPlayerIds && !event.playerId) {
        continue;
      }
      if (event.playerId && enabledPlayerIds && !enabledPlayerIds.has(event.playerId)) {
        continue;
      }

      const startTime = Math.max(0, getReplayFrameTime(replay, event.frame, event.time));
      const sizeLabel = pad.size === "big" ? "Big" : "Small";
      const playerLabel = event.playerName ? `${event.playerName} ` : "";
      const isTeamZero = event.playerId ? (playerTeams.get(event.playerId) ?? null) : null;
      ranges.push({
        id: `boost-pickup:${pad.index}:${event.frame}:${eventIndex}`,
        startTime,
        endTime: Math.max(startTime + BOOST_PICKUP_TICK_SECONDS, startTime),
        lane: "boost-pickups",
        laneLabel: "Boost Pickups",
        label: `${playerLabel}picked up ${sizeLabel.toLowerCase()} boost pad ${pad.index}`,
        shortLabel: pad.size === "big" ? "100" : "12",
        color: teamTimelineColor(isTeamZero) ?? BOOST_PICKUP_COLORS[pad.size],
        isTeamZero,
      });
    }
  }

  return ranges.sort(compareTimelineRanges);
}

function padTypesFromOptions(options: BoostPickupTimelineRangeOptions): Set<BoostPickupPadType> {
  if (options.padTypes) {
    return new Set(options.padTypes);
  }

  if (options.sizes) {
    const sizes = new Set(options.sizes);
    const padTypes = new Set<BoostPickupPadType>();
    if (sizes.has("big")) {
      padTypes.add("big");
    }
    if (sizes.has("small")) {
      padTypes.add("small");
    }
    if (sizes.has("big") && sizes.has("small")) {
      padTypes.add("ambiguous");
    }
    return padTypes;
  }

  return new Set(["big", "small", "ambiguous"]);
}

function getReplayFrameTime(
  replay: ReplayModel | undefined,
  frame: number | undefined,
  fallbackTime: number,
): number {
  return replay?.frames?.[frame ?? -1]?.time ?? fallbackTime;
}

function teamTimelineColor(isTeamZero: boolean | null | undefined): string | null {
  if (isTeamZero === true) {
    return BLUE_TIMELINE_COLOR;
  }
  if (isTeamZero === false) {
    return ORANGE_TIMELINE_COLOR;
  }

  return null;
}

function remoteIdToString(playerId: Record<string, unknown>): string {
  const [kind, value] = Object.entries(playerId)[0] ?? ["Unknown", "unknown"];
  const normalizedValue = typeof value === "string" ? value : JSON.stringify(value);
  return `${kind}:${normalizedValue}`;
}

function formatBoostPickupPadType(padType: BoostPickupPadType): string {
  return {
    big: "big",
    small: "small",
    ambiguous: "ambiguous",
  }[padType];
}

function formatBoostPickupComparison(comparison: BoostPickupComparison): string {
  return {
    both: "counted",
    ghost: "ghost",
    missed: "missed",
  }[comparison];
}

function boostPickupShortLabel(
  comparison: BoostPickupComparison,
  padType: BoostPickupPadType,
): string {
  if (comparison === "ghost") {
    return "G";
  }
  if (comparison === "missed") {
    return "M";
  }
  return {
    big: "100",
    small: "12",
    ambiguous: "?",
  }[padType];
}

function compareTimelineRanges(left: ReplayTimelineRange, right: ReplayTimelineRange): number {
  if (left.startTime !== right.startTime) {
    return left.startTime - right.startTime;
  }
  return (left.id ?? "").localeCompare(right.id ?? "");
}
