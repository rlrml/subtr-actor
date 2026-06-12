/**
 * RealtimePossession - Real-time ball possession bar during playback
 * (018-stats-compiler)
 */

import { useMemo } from 'react';

interface TeamStatsTimelineEntry {
  time: number;
  possessionPercentage: number;
  avgTeamSpeed: number;
  totalBoostPickups: number;
}

interface RealtimePossessionProps {
  teamTimelines: Record<number, TeamStatsTimelineEntry[]> | null;
  currentTime: number;
  compact?: boolean;
}

// Binary search to find entry at or before given time
function findAtTime<T extends { time: number }>(timeline: T[], targetTime: number): T | null {
  if (!timeline || timeline.length === 0) return null;
  if (targetTime <= timeline[0].time) return timeline[0];
  if (targetTime >= timeline[timeline.length - 1].time) return timeline[timeline.length - 1];

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

  return timeline[low];
}

export function RealtimePossession({ teamTimelines, currentTime, compact = false, inline = false }: RealtimePossessionProps & { inline?: boolean }) {
  const possession = useMemo(() => {
    if (!teamTimelines) return { blue: 50, orange: 50 };

    const team0Entry = findAtTime(teamTimelines[0] || [], currentTime);
    const team1Entry = findAtTime(teamTimelines[1] || [], currentTime);

    const blue = team0Entry?.possessionPercentage ?? 50;
    const orange = team1Entry?.possessionPercentage ?? 50;

    // Normalize
    const total = blue + orange;
    if (total === 0) return { blue: 50, orange: 50 };

    return {
      blue: (blue / total) * 100,
      orange: (orange / total) * 100,
    };
  }, [teamTimelines, currentTime]);

  // Inline mode - for integration into score display with percentages
  if (inline) {
    return (
      <div className="flex items-center w-full h-full gap-1">
        {/* Blue percentage */}
        <span className="text-[9px] font-semibold text-blue-300 w-6 text-right tabular-nums">
          {possession.blue.toFixed(0)}%
        </span>
        {/* Bar - blue and orange side by side */}
        <div className="flex flex-1 h-1.5 rounded-full overflow-hidden">
          <div
            className="bg-blue-500 transition-all duration-300"
            style={{ width: `${possession.blue}%` }}
          />
          <div
            className="bg-orange-500 transition-all duration-300"
            style={{ width: `${possession.orange}%` }}
          />
        </div>
        {/* Orange percentage */}
        <span className="text-[9px] font-semibold text-orange-300 w-6 text-left tabular-nums">
          {possession.orange.toFixed(0)}%
        </span>
      </div>
    );
  }

  if (compact) {
    return (
      <div className="flex items-center gap-2 px-2 py-1 rounded bg-black/60">
        <span className="text-blue-400 text-xs font-mono w-8 text-right">{possession.blue.toFixed(0)}%</span>
        <div className="relative w-16 h-1.5 rounded-full bg-gray-800 overflow-hidden">
          <div
            className="absolute inset-y-0 left-0 bg-blue-500"
            style={{ width: `${possession.blue}%` }}
          />
          <div
            className="absolute inset-y-0 right-0 bg-orange-500"
            style={{ width: `${possession.orange}%` }}
          />
        </div>
        <span className="text-orange-400 text-xs font-mono w-8">{possession.orange.toFixed(0)}%</span>
      </div>
    );
  }

  return (
    <div className="bg-black/60 backdrop-blur-sm rounded-lg p-3">
      <div className="flex justify-between text-xs mb-1">
        <span className="text-blue-400 font-medium">{possession.blue.toFixed(0)}%</span>
        <span className="text-gray-400 uppercase tracking-wider text-[10px]">Possession</span>
        <span className="text-orange-400 font-medium">{possession.orange.toFixed(0)}%</span>
      </div>

      <div className="relative h-2 rounded-full bg-gray-800 overflow-hidden">
        <div
          className="absolute inset-y-0 left-0 bg-gradient-to-r from-blue-500 to-blue-400 transition-all duration-200"
          style={{ width: `${possession.blue}%` }}
        />
        <div
          className="absolute inset-y-0 right-0 bg-gradient-to-l from-orange-500 to-orange-400 transition-all duration-200"
          style={{ width: `${possession.orange}%` }}
        />
      </div>
    </div>
  );
}
