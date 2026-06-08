import type { KickoffEvent } from "./generated/KickoffEvent.ts";
import type { KickoffPlayerStats } from "./generated/KickoffPlayerStats.ts";
import type { KickoffSupportEvent } from "./generated/KickoffSupportEvent.ts";
import type { KickoffTakerEvent } from "./generated/KickoffTakerEvent.ts";
import type { KickoffTeamStats } from "./generated/KickoffTeamStats.ts";
import type { LabeledCounts } from "./generated/LabeledCounts.ts";
import type { StatLabel } from "./generated/StatLabel.ts";
import type { StatsFrame, MaterializedStatsTimeline } from "./statsTimeline.ts";

type KickoffPlayerStatsWithLabels = KickoffPlayerStats & {
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

function defaultKickoffTeamStats(): KickoffTeamStats {
  return {
    count: 0,
    wins: 0,
    losses: 0,
    neutral_outcomes: 0,
    kickoff_possessions: 0,
    opponent_kickoff_possessions: 0,
    kickoff_possession_advantages: 0,
    opponent_kickoff_possession_advantages: 0,
    contested_kickoff_possessions: 0,
    kickoff_goal_count: 0,
    kickoff_goals_for: 0,
    kickoff_goals_against: 0,
    win_strength_sample_count: 0,
    cumulative_win_strength: 0,
    boost_after_sample_count: 0,
    cumulative_boost_after: 0,
    fake_count: 0,
    missed_count: 0,
  };
}

function defaultKickoffPlayerStats(): KickoffPlayerStatsWithLabels {
  return {
    count: 0,
    touches: 0,
    fakes: 0,
    misses: 0,
    support_go_for_boosts: 0,
    support_cheats: 0,
    support_other: 0,
    kickoff_goal_count: 0,
    boost_after_sample_count: 0,
    cumulative_boost_after: 0,
  };
}

function sortKickoffEvents(events: readonly KickoffEvent[]): KickoffEvent[] {
  return events
    .map((event, index) => ({ event, index }))
    .sort((left, right) => {
      if (left.event.end_frame !== right.event.end_frame) {
        return left.event.end_frame - right.event.end_frame;
      }
      if (left.event.end_time !== right.event.end_time) {
        return left.event.end_time - right.event.end_time;
      }
      return left.index - right.index;
    })
    .map(({ event }) => event);
}

function labelSortKey(label: StatLabel): string {
  return `${label.key}\u0000${label.value}`;
}

function labelsSortKey(labels: readonly StatLabel[]): string {
  return labels.map(labelSortKey).join("\u0001");
}

function incrementLabels(stats: KickoffPlayerStatsWithLabels, labels: StatLabel[]): void {
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

function takerLabels(event: KickoffTakerEvent): StatLabel[] {
  return [
    { key: "kickoff_spawn", value: event.spawn_position },
    { key: "taker_outcome", value: event.outcome },
    { key: "kickoff_approach", value: event.approach },
  ];
}

function supportLabels(event: KickoffSupportEvent): StatLabel[] {
  return [
    { key: "kickoff_spawn", value: event.spawn_position },
    { key: "support_behavior", value: event.support_behavior },
  ];
}

function applyTeamEvent(stats: KickoffTeamStats, teamIsTeam0: boolean, event: KickoffEvent): void {
  stats.count += 1;
  if (event.outcome === "neutral") {
    stats.neutral_outcomes += 1;
  } else if (
    event.outcome === (teamIsTeam0 ? "team_zero_win" : "team_one_win")
  ) {
    stats.wins += 1;
  } else if (
    event.outcome === (teamIsTeam0 ? "team_one_win" : "team_zero_win")
  ) {
    stats.losses += 1;
  }

  if (event.kickoff_possession_outcome === "contested") {
    stats.contested_kickoff_possessions += 1;
  } else if (
    event.kickoff_possession_outcome ===
    (teamIsTeam0 ? "team_zero_possession" : "team_one_possession")
  ) {
    stats.kickoff_possessions += 1;
  } else if (
    event.kickoff_possession_outcome ===
    (teamIsTeam0 ? "team_one_possession" : "team_zero_possession")
  ) {
    stats.opponent_kickoff_possessions += 1;
  } else if (
    event.kickoff_possession_outcome ===
    (teamIsTeam0 ? "team_zero_advantage" : "team_one_advantage")
  ) {
    stats.kickoff_possession_advantages += 1;
  } else {
    stats.opponent_kickoff_possession_advantages += 1;
  }

  if (event.kickoff_goal) {
    stats.kickoff_goal_count += 1;
    if (event.scoring_team_is_team_0 === teamIsTeam0) {
      stats.kickoff_goals_for += 1;
    } else if (event.scoring_team_is_team_0 != null) {
      stats.kickoff_goals_against += 1;
    }
  }

  if (event.win_strength != null) {
    stats.win_strength_sample_count += 1;
    stats.cumulative_win_strength += event.win_strength;
  }
}

function applyGlobalTakerStats(
  teamZero: KickoffTeamStats,
  teamOne: KickoffTeamStats,
  taker: KickoffTakerEvent | null,
): void {
  if (!taker) {
    return;
  }
  if (taker.boost_after != null) {
    teamZero.boost_after_sample_count += 1;
    teamZero.cumulative_boost_after += taker.boost_after;
    teamOne.boost_after_sample_count += 1;
    teamOne.cumulative_boost_after += taker.boost_after;
  }
  if (taker.outcome === "fake") {
    teamZero.fake_count += 1;
    teamOne.fake_count += 1;
  } else if (taker.outcome === "missed") {
    teamZero.missed_count += 1;
    teamOne.missed_count += 1;
  }
}

function applyTakerPlayerEvent(
  stats: KickoffPlayerStatsWithLabels,
  kickoff: KickoffEvent,
  taker: KickoffTakerEvent,
): void {
  stats.count += 1;
  incrementLabels(stats, takerLabels(taker));
  if (taker.outcome === "touched") {
    stats.touches += 1;
  } else if (taker.outcome === "fake") {
    stats.fakes += 1;
  } else if (taker.outcome === "missed") {
    stats.misses += 1;
  }
  if (kickoff.kickoff_goal && kickoff.scoring_team_is_team_0 === taker.is_team_0) {
    stats.kickoff_goal_count += 1;
  }
  if (taker.boost_after != null) {
    stats.boost_after_sample_count += 1;
    stats.cumulative_boost_after += taker.boost_after;
  }
}

function applySupportPlayerEvent(
  stats: KickoffPlayerStatsWithLabels,
  kickoff: KickoffEvent,
  support: KickoffSupportEvent,
): void {
  stats.count += 1;
  incrementLabels(stats, supportLabels(support));
  if (support.first_touch_time != null) {
    stats.touches += 1;
  }
  if (support.support_behavior === "go_for_boost") {
    stats.support_go_for_boosts += 1;
  } else if (support.support_behavior === "cheat") {
    stats.support_cheats += 1;
  } else if (support.support_behavior === "other") {
    stats.support_other += 1;
  }
  if (kickoff.kickoff_goal && kickoff.scoring_team_is_team_0 === support.is_team_0) {
    stats.kickoff_goal_count += 1;
  }
  if (support.boost_after != null) {
    stats.boost_after_sample_count += 1;
    stats.cumulative_boost_after += support.boost_after;
  }
}

function applyPlayerEvent(
  players: Map<string, KickoffPlayerStatsWithLabels>,
  kickoff: KickoffEvent,
  player: KickoffTakerEvent | KickoffSupportEvent,
): void {
  const playerKey = remoteIdKey(player.player);
  const stats = players.get(playerKey) ?? defaultKickoffPlayerStats();
  players.set(playerKey, stats);
  if ("outcome" in player) {
    applyTakerPlayerEvent(stats, kickoff, player);
  } else {
    applySupportPlayerEvent(stats, kickoff, player);
  }
}

function assignKickoffTeamStats(target: KickoffTeamStats, source: KickoffTeamStats): void {
  Object.assign(target, source);
}

function assignKickoffPlayerStats(
  target: KickoffPlayerStats,
  source: KickoffPlayerStatsWithLabels | undefined,
): void {
  Object.assign(target, source ?? defaultKickoffPlayerStats());
  if (source?.labeled_event_counts) {
    target.labeled_event_counts = cloneLabeledCounts(source.labeled_event_counts);
  } else {
    delete target.labeled_event_counts;
  }
}

export function applyKickoffEventDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createKickoffEventDerivedStatsAccumulator(timeline);

  for (const frame of timeline.frames) {
    accumulator.applyFrame(frame);
  }

  return timeline;
}

export function createKickoffEventDerivedStatsAccumulator(timeline: MaterializedStatsTimeline): {
  applyFrame(frame: StatsFrame): void;
} {
  const events = sortKickoffEvents(timeline.events.kickoff ?? []);

  let eventIndex = 0;
  const teamZero = defaultKickoffTeamStats();
  const teamOne = defaultKickoffTeamStats();
  const players = new Map<string, KickoffPlayerStatsWithLabels>();

  return {
    applyFrame(frame: StatsFrame): void {
      while (eventIndex < events.length && events[eventIndex]!.end_frame <= frame.frame_number) {
        const event = events[eventIndex] as KickoffEvent;
        applyTeamEvent(teamZero, true, event);
        applyTeamEvent(teamOne, false, event);
        applyGlobalTakerStats(teamZero, teamOne, event.team_zero_taker);
        applyGlobalTakerStats(teamZero, teamOne, event.team_one_taker);

        if (event.team_zero_taker) {
          applyPlayerEvent(players, event, event.team_zero_taker);
        }
        if (event.team_one_taker) {
          applyPlayerEvent(players, event, event.team_one_taker);
        }
        for (const player of event.team_zero_non_takers) {
          applyPlayerEvent(players, event, player);
        }
        for (const player of event.team_one_non_takers) {
          applyPlayerEvent(players, event, player);
        }

        eventIndex += 1;
      }

      assignKickoffTeamStats(frame.team_zero.kickoff, teamZero);
      assignKickoffTeamStats(frame.team_one.kickoff, teamOne);
      for (const player of frame.players) {
        assignKickoffPlayerStats(player.kickoff, players.get(remoteIdKey(player.player_id)));
      }
    },
  };
}
