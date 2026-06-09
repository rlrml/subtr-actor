import type { FiftyFiftyEvent } from "./generated/FiftyFiftyEvent.ts";
import type { FiftyFiftyPlayerStats } from "./generated/FiftyFiftyPlayerStats.ts";
import type { FiftyFiftyTeamStats } from "./generated/FiftyFiftyTeamStats.ts";
import type { LabeledCounts } from "./generated/LabeledCounts.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";

type FiftyFiftyPlayerStatsWithLabels = FiftyFiftyPlayerStats & {
  labeled_event_counts?: LabeledCounts;
};

function remoteIdKey(playerId: unknown): string {
  if (!playerId || typeof playerId !== "object") {
    return String(playerId);
  }
  const [kind, value] = Object.entries(playerId as Record<string, unknown>)[0] ?? [
    "Unknown",
    "unknown",
  ];
  return `${kind}:${typeof value === "string" ? value : JSON.stringify(value)}`;
}

function defaultFiftyFiftyTeamStats(): FiftyFiftyTeamStats {
  return {
    count: 0,
    wins: 0,
    losses: 0,
    neutral_outcomes: 0,
    kickoff_count: 0,
    kickoff_wins: 0,
    kickoff_losses: 0,
    kickoff_neutral_outcomes: 0,
    possession_after_count: 0,
    opponent_possession_after_count: 0,
    neutral_possession_after_count: 0,
    kickoff_possession_after_count: 0,
    kickoff_opponent_possession_after_count: 0,
    kickoff_neutral_possession_after_count: 0,
  };
}

function defaultFiftyFiftyPlayerStats(): FiftyFiftyPlayerStatsWithLabels {
  return {
    count: 0,
    wins: 0,
    losses: 0,
    neutral_outcomes: 0,
    kickoff_count: 0,
    kickoff_wins: 0,
    kickoff_losses: 0,
    kickoff_neutral_outcomes: 0,
    possession_after_count: 0,
    kickoff_possession_after_count: 0,
  };
}

function sortFiftyFiftyEvents(events: readonly FiftyFiftyEvent[]): FiftyFiftyEvent[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      if (left.event.resolve_frame !== right.event.resolve_frame) {
        return left.event.resolve_frame - right.event.resolve_frame;
      }
      if (left.event.resolve_time !== right.event.resolve_time) {
        return left.event.resolve_time - right.event.resolve_time;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

function phaseLabel(isKickoff: boolean): StatLabel {
  return { key: "phase", value: isKickoff ? "kickoff" : "open_play" };
}

function playerOutcomeLabel(
  playerTeamIsTeam0: boolean,
  winningTeamIsTeam0: boolean | null,
): StatLabel {
  if (winningTeamIsTeam0 == null) {
    return { key: "outcome", value: "neutral" };
  }
  return { key: "outcome", value: winningTeamIsTeam0 === playerTeamIsTeam0 ? "win" : "loss" };
}

function playerPossessionLabel(
  playerTeamIsTeam0: boolean,
  possessionTeamIsTeam0: boolean | null,
): StatLabel {
  if (possessionTeamIsTeam0 == null) {
    return { key: "possession_after", value: "neutral" };
  }
  return {
    key: "possession_after",
    value: possessionTeamIsTeam0 === playerTeamIsTeam0 ? "self" : "opponent",
  };
}

function playerDodgeStateLabel(playerTeamIsTeam0: boolean, event: FiftyFiftyEvent): StatLabel {
  const dodgeContact = playerTeamIsTeam0
    ? event.team_zero_dodge_contact
    : event.team_one_dodge_contact;
  return { key: "dodge_state", value: dodgeContact ? "dodge" : "no_dodge" };
}

function labelSortKey(label: StatLabel): string {
  return `${label.key}\u0000${label.value}`;
}

function labelsSortKey(labels: readonly StatLabel[]): string {
  return labels.map(labelSortKey).join("\u0001");
}

function incrementLabels(stats: FiftyFiftyPlayerStatsWithLabels, labels: StatLabel[]): void {
  labels.sort((left, right) => labelSortKey(left).localeCompare(labelSortKey(right)));
  const labeledCounts = (stats.labeled_event_counts ??= { entries: [] });
  const existing = labeledCounts.entries.find(
    (entry) => labelsSortKey(entry.labels) === labelsSortKey(labels),
  );
  if (existing) {
    existing.count += 1;
  } else {
    labeledCounts.entries.push({ labels: [...labels], count: 1 });
    labeledCounts.entries.sort((left, right) =>
      labelsSortKey(left.labels).localeCompare(labelsSortKey(right.labels)),
    );
  }
}

function cloneLabeledCounts(counts: LabeledCounts): LabeledCounts {
  return {
    entries: counts.entries.map((entry) => ({
      labels: entry.labels.map((label) => ({ ...label })),
      count: entry.count,
    })),
  };
}

function applyFiftyFiftyTeamEvent(
  stats: FiftyFiftyTeamStats,
  teamIsTeam0: boolean,
  event: FiftyFiftyEvent,
): void {
  stats.count += 1;
  if (event.winning_team_is_team_0 == null) {
    stats.neutral_outcomes += 1;
  } else if (event.winning_team_is_team_0 === teamIsTeam0) {
    stats.wins += 1;
  } else {
    stats.losses += 1;
  }

  if (event.possession_team_is_team_0 == null) {
    stats.neutral_possession_after_count += 1;
  } else if (event.possession_team_is_team_0 === teamIsTeam0) {
    stats.possession_after_count += 1;
  } else {
    stats.opponent_possession_after_count += 1;
  }

  if (event.is_kickoff) {
    stats.kickoff_count += 1;
    if (event.winning_team_is_team_0 == null) {
      stats.kickoff_neutral_outcomes += 1;
    } else if (event.winning_team_is_team_0 === teamIsTeam0) {
      stats.kickoff_wins += 1;
    } else {
      stats.kickoff_losses += 1;
    }

    if (event.possession_team_is_team_0 == null) {
      stats.kickoff_neutral_possession_after_count += 1;
    } else if (event.possession_team_is_team_0 === teamIsTeam0) {
      stats.kickoff_possession_after_count += 1;
    } else {
      stats.kickoff_opponent_possession_after_count += 1;
    }
  }
}

function applyFiftyFiftyPlayerEvent(
  stats: FiftyFiftyPlayerStatsWithLabels,
  playerTeamIsTeam0: boolean,
  event: FiftyFiftyEvent,
): void {
  incrementLabels(stats, [
    phaseLabel(event.is_kickoff),
    playerOutcomeLabel(playerTeamIsTeam0, event.winning_team_is_team_0),
    playerPossessionLabel(playerTeamIsTeam0, event.possession_team_is_team_0),
    playerDodgeStateLabel(playerTeamIsTeam0, event),
  ]);

  stats.count += 1;
  if (event.winning_team_is_team_0 == null) {
    stats.neutral_outcomes += 1;
  } else if (event.winning_team_is_team_0 === playerTeamIsTeam0) {
    stats.wins += 1;
  } else {
    stats.losses += 1;
  }
  if (event.possession_team_is_team_0 === playerTeamIsTeam0) {
    stats.possession_after_count += 1;
  }
  if (event.is_kickoff) {
    stats.kickoff_count += 1;
    if (event.winning_team_is_team_0 == null) {
      stats.kickoff_neutral_outcomes += 1;
    } else if (event.winning_team_is_team_0 === playerTeamIsTeam0) {
      stats.kickoff_wins += 1;
    } else {
      stats.kickoff_losses += 1;
    }
    if (event.possession_team_is_team_0 === playerTeamIsTeam0) {
      stats.kickoff_possession_after_count += 1;
    }
  }
}

function assignFiftyFiftyPlayerStats(
  target: FiftyFiftyPlayerStats,
  source: FiftyFiftyPlayerStatsWithLabels | undefined,
): void {
  Object.assign(target, source ?? defaultFiftyFiftyPlayerStats());
  if (source?.labeled_event_counts) {
    (target as FiftyFiftyPlayerStatsWithLabels).labeled_event_counts = cloneLabeledCounts(
      source.labeled_event_counts,
    );
  } else {
    delete (target as FiftyFiftyPlayerStatsWithLabels).labeled_event_counts;
  }
}

function assignFiftyFiftyTeamStats(target: FiftyFiftyTeamStats, source: FiftyFiftyTeamStats): void {
  Object.assign(target, source);
}

export function applyFiftyFiftyEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createFiftyFiftyEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createFiftyFiftyEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortFiftyFiftyEvents(statsEventPayloads(timeline, "fifty_fifty"));

  let eventIndex = 0;
  const teamZero = defaultFiftyFiftyTeamStats();
  const teamOne = defaultFiftyFiftyTeamStats();
  const players = new Map<string, FiftyFiftyPlayerStatsWithLabels>();

  return {
    applyFrame(frame: StatsFrame): void {
      while (
        eventIndex < events.length &&
        events[eventIndex]!.resolve_frame <= frame.frame_number
      ) {
        const event = events[eventIndex] as FiftyFiftyEvent;
        applyFiftyFiftyTeamEvent(teamZero, true, event);
        applyFiftyFiftyTeamEvent(teamOne, false, event);
        if (event.team_zero_player != null) {
          const playerKey = remoteIdKey(event.team_zero_player);
          const stats = players.get(playerKey) ?? defaultFiftyFiftyPlayerStats();
          players.set(playerKey, stats);
          applyFiftyFiftyPlayerEvent(stats, true, event);
        }
        if (event.team_one_player != null) {
          const playerKey = remoteIdKey(event.team_one_player);
          const stats = players.get(playerKey) ?? defaultFiftyFiftyPlayerStats();
          players.set(playerKey, stats);
          applyFiftyFiftyPlayerEvent(stats, false, event);
        }
        eventIndex += 1;
      }

      assignFiftyFiftyTeamStats(frame.team_zero.fifty_fifty, teamZero);
      assignFiftyFiftyTeamStats(frame.team_one.fifty_fifty, teamOne);
      for (const player of frame.players) {
        assignFiftyFiftyPlayerStats(player.fifty_fifty, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
