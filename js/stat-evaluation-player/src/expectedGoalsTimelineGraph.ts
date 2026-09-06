import type {
  ReplayTimelineGraph,
  ReplayTimelineGraphHighlight,
  ReplayTimelineGraphMarker,
} from "@rlrml/player";

import type { ExpectedGoalsTimelineTracks } from "./generated/ExpectedGoalsTimelineTracks.ts";
import type { ExpectedGoalsTeamStats } from "./generated/ExpectedGoalsTeamStats.ts";
import type { ThreatEpisodeEvent } from "./generated/ThreatEpisodeEvent.ts";
import type { StatsTimeline } from "./statsTimeline.ts";

const TEAM_ZERO_COLOR = "#3b82f6";
const TEAM_ONE_COLOR = "#f59e0b";
const DEFAULT_OPEN_THRESHOLD = 0.15;
const DEFAULT_END_THRESHOLD = 0.05;

interface ExpectedGoalsTrackPayload {
  expected_goals_tracks?: Partial<ExpectedGoalsTimelineTracks>;
}

function formatPercent(value: number): string {
  return `${(value * 100).toFixed(1)}%`;
}

function teamLabel(isTeamZero: boolean): string {
  return isTeamZero ? "Blue" : "Orange";
}

function teamColor(isTeamZero: boolean): string {
  return isTeamZero ? TEAM_ZERO_COLOR : TEAM_ONE_COLOR;
}

function frameTimes(timeline: StatsTimeline): Map<number, number> {
  return new Map(timeline.frames.map((frame) => [frame.frame_number, frame.time]));
}

function compactTracks(timeline: StatsTimeline): Partial<ExpectedGoalsTimelineTracks> | null {
  return (timeline as StatsTimeline & ExpectedGoalsTrackPayload).expected_goals_tracks ?? null;
}

function buildSeries(
  timeline: StatsTimeline,
  tracks: Partial<ExpectedGoalsTimelineTracks> | null,
  metric: "current_threat" | "xg" = "current_threat",
) {
  if (!tracks?.teams) {
    return [true, false].map((isTeamZero) => ({
      id: isTeamZero ? "team-zero" : "team-one",
      label: teamLabel(isTeamZero),
      color: teamColor(isTeamZero),
      points: timeline.frames.map((frame) => {
        const team = (isTeamZero ? frame.team_zero : frame.team_one) as {
          expected_goals?: Partial<ExpectedGoalsTeamStats>;
        };
        return {
          time: frame.time,
          value: team.expected_goals?.[metric] ?? null,
        };
      }),
    }));
  }

  const times = frameTimes(timeline);
  return [true, false].map((isTeamZero) => {
    const track = tracks.teams?.find((candidate) => candidate.is_team_0 === isTeamZero);
    return {
      id: isTeamZero ? "team-zero" : "team-one",
      label: teamLabel(isTeamZero),
      color: teamColor(isTeamZero),
      points: (track?.points ?? []).flatMap((point) => {
        const time = times.get(point.frame);
        return time === undefined ? [] : [{ time, value: point.stats[metric] ?? null }];
      }),
    };
  });
}

function incidentHighlights(
  episodes: readonly ThreatEpisodeEvent[],
): ReplayTimelineGraphHighlight[] {
  return episodes.map((episode) => ({
    startTime: episode.start_time,
    endTime: episode.end_time,
    color: teamColor(episode.team_is_team_0),
    label: `${teamLabel(episode.team_is_team_0)} pressure · peak ${formatPercent(episode.peak_value)}`,
  }));
}

function incidentMarkers(episodes: readonly ThreatEpisodeEvent[]): ReplayTimelineGraphMarker[] {
  return episodes.map((episode) => ({
    time: episode.peak_time,
    value: episode.peak_value,
    color: teamColor(episode.team_is_team_0),
    label: `${teamLabel(episode.team_is_team_0)} peak threat ${formatPercent(episode.peak_value)}`,
  }));
}

export function buildExpectedGoalsTimelineGraphs(timeline: StatsTimeline): ReplayTimelineGraph[] {
  const tracks = compactTracks(timeline);
  const series = buildSeries(timeline, tracks);
  if (!series.some((team) => team.points.some((point) => point.value !== null))) {
    return [];
  }

  const episodes = tracks?.episodes ?? [];
  const startThreshold = tracks?.config?.episode_threshold ?? DEFAULT_OPEN_THRESHOLD;
  const endThreshold = tracks?.config?.episode_end_threshold ?? DEFAULT_END_THRESHOLD;

  return [
    {
      id: "expected-goals",
      label: "Expected goals (pre-touch)",
      minValue: 0,
      series: buildSeries(timeline, tracks, "xg"),
    },
    {
      id: "instantaneous-xg",
      label: "Scoring threat (5s)",
      minValue: 0,
      maxValue: 1,
      series,
      references: [
        {
          value: startThreshold,
          label: `open ${formatPercent(startThreshold)}`,
          color: "#d8ebff",
          className: "incident-open",
        },
        {
          value: endThreshold,
          label: `end ${formatPercent(endThreshold)}`,
          color: "#9aaabd",
          className: "incident-end",
        },
      ],
      highlights: incidentHighlights(episodes),
      markers: incidentMarkers(episodes),
    },
  ];
}
