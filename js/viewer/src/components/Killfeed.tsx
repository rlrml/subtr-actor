import { useState, useEffect, useRef } from 'react';

// Team colors matching Rocket League
const TEAM_COLORS = {
  blue: '#3b82f6',
  orange: '#f97316',
};

export interface KillfeedEntry {
  id: string;
  attacker: string;
  victim: string;
  attackerTeam: number; // 0 = blue, 1 = orange
  victimTeam: number;
  timestamp: number; // replay time when event occurred
}

interface KillfeedProps {
  entries: KillfeedEntry[];
  maxEntries?: number;
  displayDuration?: number; // milliseconds
}

interface DisplayedEntry extends KillfeedEntry {
  addedAt: number; // Date.now() when entry was added
  isExiting: boolean;
}

export function Killfeed({
  entries,
  maxEntries = 5,
  displayDuration = 5000,
}: KillfeedProps) {
  const [displayedEntries, setDisplayedEntries] = useState<DisplayedEntry[]>([]);
  const lastProcessedRef = useRef<string | null>(null);

  // Process new entries
  useEffect(() => {
    if (entries.length === 0) return;

    const latestEntry = entries[entries.length - 1];

    // Skip if we already processed this entry
    if (lastProcessedRef.current === latestEntry.id) return;
    lastProcessedRef.current = latestEntry.id;

    // Add new entry with timestamp
    setDisplayedEntries((prev) => {
      const newEntry: DisplayedEntry = {
        ...latestEntry,
        addedAt: Date.now(),
        isExiting: false,
      };

      // Add to top, limit to maxEntries
      const updated = [newEntry, ...prev].slice(0, maxEntries);
      return updated;
    });
  }, [entries, maxEntries]);

  // Auto-dismiss entries after displayDuration
  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now();

      setDisplayedEntries((prev) => {
        // Mark entries that should exit
        const updated = prev.map((entry) => {
          if (!entry.isExiting && now - entry.addedAt >= displayDuration - 500) {
            return { ...entry, isExiting: true };
          }
          return entry;
        });

        // Remove entries that have fully exited
        return updated.filter((entry) => now - entry.addedAt < displayDuration);
      });
    }, 100);

    return () => clearInterval(interval);
  }, [displayDuration]);

  if (displayedEntries.length === 0) return null;

  return (
    <div className="absolute top-20 right-4 z-40 flex flex-col gap-2 pointer-events-none">
      {displayedEntries.map((entry) => {
        const attackerColor = entry.attackerTeam === 0 ? TEAM_COLORS.blue : TEAM_COLORS.orange;
        const victimColor = entry.victimTeam === 0 ? TEAM_COLORS.blue : TEAM_COLORS.orange;

        return (
          <div
            key={entry.id}
            className={`flex items-stretch h-8 rounded-full overflow-hidden shadow-lg transition-all duration-300 ${
              entry.isExiting
                ? 'opacity-0 translate-x-10'
                : 'opacity-100 translate-x-0 animate-slide-in-right'
            }`}
          >
            {/* Attacker side */}
            <div
              className="flex items-center justify-end px-3"
              style={{ backgroundColor: attackerColor }}
            >
              <span className="text-white font-bold text-sm drop-shadow-md truncate max-w-[100px]">
                {entry.attacker}
              </span>
            </div>

            {/* Center bomb icon with gradient transition */}
            <div
              className="flex items-center justify-center px-2 relative"
              style={{
                background: `linear-gradient(to right, ${attackerColor}, ${victimColor})`,
              }}
            >
              {/* Bomb SVG icon */}
              <svg
                viewBox="0 0 24 24"
                className="w-5 h-5 flex-shrink-0 drop-shadow-md"
                fill="none"
              >
                {/* Bomb body - dark circle */}
                <circle cx="12" cy="14" r="7" fill="#1a1a1a" />
                {/* Highlight on bomb */}
                <ellipse cx="9" cy="11" rx="2" ry="1.5" fill="#444" />
                {/* Fuse */}
                <path d="M12 7 L12 4 L14 2" stroke="#8B4513" strokeWidth="2" strokeLinecap="round" fill="none" />
                {/* Spark */}
                <circle cx="14" cy="2" r="1.5" fill="#ffcc00" />
                <circle cx="14" cy="2" r="2.5" fill="#ff6600" opacity="0.5" />
              </svg>
            </div>

            {/* Victim side */}
            <div
              className="flex items-center justify-start px-3"
              style={{ backgroundColor: victimColor }}
            >
              <span className="text-white font-bold text-sm drop-shadow-md truncate max-w-[100px]">
                {entry.victim}
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
