import type { ExpectedGoalsPlayerStats } from "./generated/ExpectedGoalsPlayerStats.ts";
import type { ExpectedGoalsPlayerTimelinePoint } from "./generated/ExpectedGoalsPlayerTimelinePoint.ts";
import type { ExpectedGoalsTeamStats } from "./generated/ExpectedGoalsTeamStats.ts";
import type { ExpectedGoalsTeamTimelinePoint } from "./generated/ExpectedGoalsTeamTimelinePoint.ts";
import type { ExpectedGoalsTimelineTracks } from "./generated/ExpectedGoalsTimelineTracks.ts";
import type { MaterializedStatsTimeline, StatsFrame } from "./statsTimeline.ts";

function remoteIdKey(playerId: unknown): string {
  return JSON.stringify(playerId);
}

class SnapshotCursor<T extends { frame: number; stats: unknown }> {
  private index = 0;

  constructor(private readonly points: readonly T[]) {}

  sample(frameNumber: number): T["stats"] | undefined {
    while (
      this.index + 1 < this.points.length &&
      this.points[this.index + 1]!.frame <= frameNumber
    ) {
      this.index += 1;
    }
    const point = this.points[this.index];
    return point && point.frame <= frameNumber ? point.stats : undefined;
  }
}

function expectedGoalsTracks(timeline: MaterializedStatsTimeline): ExpectedGoalsTimelineTracks {
  return (
    (
      timeline as MaterializedStatsTimeline & {
        expected_goals_tracks?: ExpectedGoalsTimelineTracks;
      }
    ).expected_goals_tracks ?? { teams: [], players: [] }
  );
}

export function createExpectedGoalsTrackDerivedStatsAccumulator(
  timeline: MaterializedStatsTimeline,
): { applyFrame(frame: StatsFrame): void } {
  const tracks = expectedGoalsTracks(timeline);
  const teamCursors = new Map(
    tracks.teams.map(
      (track) =>
        [
          track.is_team_0,
          new SnapshotCursor<ExpectedGoalsTeamTimelinePoint>(track.points),
        ] as const,
    ),
  );
  const playerCursors = new Map(
    tracks.players.map(
      (track) =>
        [
          remoteIdKey(track.player_id),
          new SnapshotCursor<ExpectedGoalsPlayerTimelinePoint>(track.points),
        ] as const,
    ),
  );

  return {
    applyFrame(frame): void {
      const teamZero = teamCursors.get(true)?.sample(frame.frame_number) as
        | ExpectedGoalsTeamStats
        | undefined;
      const teamOne = teamCursors.get(false)?.sample(frame.frame_number) as
        | ExpectedGoalsTeamStats
        | undefined;
      if (teamZero) Object.assign(frame.team_zero.expected_goals, teamZero);
      if (teamOne) Object.assign(frame.team_one.expected_goals, teamOne);

      for (const player of frame.players) {
        const stats = playerCursors
          .get(remoteIdKey(player.player_id))
          ?.sample(frame.frame_number) as ExpectedGoalsPlayerStats | undefined;
        if (stats) Object.assign(player.expected_goals, stats);
      }
    },
  };
}

export function applyExpectedGoalsTrackDerivedStats(
  timeline: MaterializedStatsTimeline,
): MaterializedStatsTimeline {
  const accumulator = createExpectedGoalsTrackDerivedStatsAccumulator(timeline);
  for (const frame of timeline.frames) accumulator.applyFrame(frame);
  return timeline;
}
