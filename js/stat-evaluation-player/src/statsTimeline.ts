export interface StatsTimeline {
  replay_meta: unknown;
  timeline_events: unknown[];
  frames: StatsFrame[];
}

export interface StatsFrame {
  frame_number: number;
  time: number;
  dt: number;
  players: PlayerStatsSnapshot[];
  [key: string]: unknown;
}

export interface PlayerStatsSnapshot {
  player_id: Record<string, string>;
  name: string;
  is_team_0: boolean;
  positioning?: {
    active_game_time: number;
    time_defensive_zone: number;
    time_neutral_zone: number;
    time_offensive_zone: number;
    time_defensive_half: number;
    time_offensive_half: number;
    time_demolished: number;
    time_no_teammates: number;
    time_most_back: number;
    time_most_forward: number;
    time_other_role: number;
    time_even: number;
    [key: string]: unknown;
  };
  boost?: {
    amount_collected: number;
    amount_collected_big: number;
    amount_collected_small: number;
    amount_respawned: number;
    overfill_total: number;
    overfill_from_stolen: number;
    amount_used: number;
    amount_stolen: number;
    big_pads_collected: number;
    small_pads_collected: number;
    amount_used_while_supersonic: number;
    time_zero_boost: number;
    time_hundred_boost: number;
    boost_integral: number;
    tracked_time: number;
    [key: string]: unknown;
  };
  [key: string]: unknown;
}

export function createStatsFrameLookup(statsTimeline: StatsTimeline): Map<number, StatsFrame> {
  return new Map(statsTimeline.frames.map((frame) => [frame.frame_number, frame]));
}

export function getStatsFrameForReplayFrame(
  statsFrameLookup: Map<number, StatsFrame>,
  replayFrameNumber: number,
): StatsFrame | null {
  return statsFrameLookup.get(replayFrameNumber) ?? null;
}
