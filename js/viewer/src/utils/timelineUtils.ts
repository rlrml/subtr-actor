/**
 * Timeline utility functions for binary search and stats lookup
 * Used for real-time player stats display during replay playback
 */

interface TimelineEntry {
  time: number;
  [key: string]: unknown;
}

/**
 * Binary search to find the index of the entry at or just before the given time.
 * Returns the index of the entry where entry.time <= targetTime < nextEntry.time
 *
 * @param timeline - Sorted array of entries with 'time' property
 * @param targetTime - Time to search for
 * @param lastIndex - Optional hint for starting search (optimization for sequential access)
 * @returns Index of the entry, or -1 if timeline is empty or time is before first entry
 */
export function findTimelineIndex<T extends TimelineEntry>(
  timeline: T[],
  targetTime: number,
  lastIndex = 0
): number {
  if (!timeline || timeline.length === 0) return -1;

  // Before first entry
  if (targetTime < timeline[0].time) return -1;

  // After or at last entry
  if (targetTime >= timeline[timeline.length - 1].time) {
    return timeline.length - 1;
  }

  // Quick check: is lastIndex still valid? (sequential playback optimization)
  const idx = Math.max(0, Math.min(lastIndex, timeline.length - 2));
  if (timeline[idx].time <= targetTime && timeline[idx + 1].time > targetTime) {
    return idx;
  }

  // Check next index (common case for forward playback)
  if (idx + 2 < timeline.length && timeline[idx + 1].time <= targetTime && timeline[idx + 2].time > targetTime) {
    return idx + 1;
  }

  // Fall back to binary search
  let low = 0;
  let high = timeline.length - 1;

  while (low < high) {
    const mid = Math.floor((low + high + 1) / 2);
    if (timeline[mid].time <= targetTime) {
      low = mid;
    } else {
      high = mid - 1;
    }
  }

  return low;
}

/**
 * Get the entry from a timeline at or just before the given time.
 *
 * @param timeline - Sorted array of entries with 'time' property
 * @param targetTime - Time to look up
 * @param lastIndex - Optional hint for starting search
 * @returns The entry at or before targetTime, or null if not found
 */
export function getEntryAtTime<T extends TimelineEntry>(
  timeline: T[],
  targetTime: number,
  lastIndex = 0
): T | null {
  const idx = findTimelineIndex(timeline, targetTime, lastIndex);
  if (idx === -1) return null;
  return timeline[idx];
}

/**
 * Interface for player stats at a specific time point
 */
export interface PlayerStatsAtTime {
  time: number;
  frame: number;
  ping: number;
  goals: number;
  assists: number;
  saves: number;
  shots: number;
  score: number;
  demos: number;
}

/**
 * Get player stats at a specific time from the stats timeline.
 * If no timeline data exists, returns null.
 *
 * @param playerStatsTimelines - Map of player names to their stats timelines
 * @param playerName - Name of the player
 * @param currentTime - Current playback time in seconds
 * @returns Player stats at the given time, or null if not available
 */
export function getPlayerStatsAtTime(
  playerStatsTimelines: Record<string, Array<{
    time: number;
    frame: number;
    ping: number;
    goals: number;
    assists: number;
    saves: number;
    shots: number;
    score: number;
    demos: number;
    timePlayed?: number;
  }>> | undefined | null,
  playerName: string,
  currentTime: number
): PlayerStatsAtTime | null {
  if (!playerStatsTimelines) return null;

  const timeline = playerStatsTimelines[playerName];
  if (!timeline || timeline.length === 0) return null;

  const entry = getEntryAtTime(timeline, currentTime);
  if (!entry) return null;

  return {
    time: entry.time,
    frame: entry.frame,
    ping: entry.ping,
    goals: entry.goals,
    assists: entry.assists,
    saves: entry.saves,
    shots: entry.shots,
    score: entry.score,
    demos: entry.demos,
  };
}
