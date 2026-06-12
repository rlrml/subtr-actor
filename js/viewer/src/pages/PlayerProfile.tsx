/**
 * PlayerProfile - Player profile page with stats and replay history
 * (018-stats-compiler)
 */

import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { SEOHead, StructuredData, createPlayerStructuredData } from '@/components/SEO';
import { usePlayerSEO } from '@/hooks/useSEO';
import { ArrowLeft, Loader2, AlertCircle } from 'lucide-react';
import { api } from '@/services/api';
import { GradientCard } from '@/components/ui/GradientCard';
import { PlayerProfileHeader, PlayerStatsOverview, PlayerReplayList } from '@/components/player';
import { PlayerCheatHistory } from '@/components/cheat';
import { usePlayerCheatHistory } from '@/api/cheat';
import type { FlaggedReplay } from '@/types/cheat';

interface PlayerData {
  id: string;
  platform: string;
  platformId: string;
  displayName: string;
  avatarUrl?: string | null;
  stats: {
    totalMatches: number;
    totalGoals: number;
    totalAssists: number;
    totalSaves: number;
    totalShots: number;
    totalDemos: number;
    avgSpeed?: number | null;
    avgBoostConsumption?: number | null;
    avgAirTimePercentage?: number | null;
    avgOffensivePercentage?: number | null;
  };
  firstSeenAt?: string;
  lastSeenAt?: string;
}

interface PlayerReplay {
  replayId: string;
  playedAt?: string;
  mapName?: string;
  team: number;
  goals: number;
  assists: number;
  saves: number;
  shots: number;
  score: number;
  avgSpeed?: number | null;
}

interface PlayerResponse {
  player: PlayerData;
}

interface ReplaysResponse {
  playerId: string;
  playerName: string;
  replays: PlayerReplay[];
  pagination: {
    page: number;
    limit: number;
    total: number;
  };
}

export default function PlayerProfile() {
  const { id } = useParams<{ id: string }>();
  const [player, setPlayer] = useState<PlayerData | null>(null);
  const playerSeo = usePlayerSEO(player);
  const [replays, setReplays] = useState<PlayerReplay[]>([]);
  const [loading, setLoading] = useState(true);
  const [replaysLoading, setReplaysLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Cheat detection history (032-cheat-detection)
  const { data: cheatHistory, loading: cheatHistoryLoading } = usePlayerCheatHistory(id);

  useEffect(() => {
    if (!id) return;

    const fetchPlayer = async () => {
      setLoading(true);
      setError(null);

      try {
        const data = await api.get<PlayerResponse>(`/players/${id}`);
        setPlayer(data.player);

        // Also fetch replays
        setReplaysLoading(true);
        try {
          const replaysData = await api.get<ReplaysResponse>(`/players/${id}/replays?limit=20`);
          setReplays(replaysData.replays);
        } catch {
          // Silently fail on replays - they're optional
          setReplays([]);
        } finally {
          setReplaysLoading(false);
        }
      } catch (err) {
        const apiError = err as { message?: string };
        setError(apiError.message || 'Failed to load player profile');
      } finally {
        setLoading(false);
      }
    };

    fetchPlayer();
  }, [id]);

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-violet-600/20 to-blue-600/20 flex items-center justify-center mb-4">
          <Loader2 className="w-8 h-8 animate-spin text-violet-500" />
        </div>
        <p className="text-gray-400">Loading player profile...</p>
      </div>
    );
  }

  if (error || !player) {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <div className="w-16 h-16 rounded-2xl bg-red-500/10 flex items-center justify-center mb-4">
          <AlertCircle className="w-8 h-8 text-red-400" />
        </div>
        <p className="text-red-400 mb-4">{error || 'Player not found'}</p>
        <Link to="/replays" className="text-violet-400 hover:text-violet-300 underline">
          Back to replays
        </Link>
      </div>
    );
  }

  // Structured data for player profile
  const playerStructuredData = createPlayerStructuredData({
    id: player.id,
    displayName: player.displayName,
    platform: player.platform,
    platformId: player.platformId,
  });

  return (
    <div className="space-y-6">
      <SEOHead {...playerSeo} />
      <StructuredData data={playerStructuredData} />

      {/* Back navigation */}
      <Link
        to="/replays"
        className="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors group"
      >
        <ArrowLeft className="w-4 h-4 group-hover:-translate-x-1 transition-transform" />
        Back to replays
      </Link>

      {/* Player Header */}
      <PlayerProfileHeader player={player} isFlaggedCheater={cheatHistory?.isFlaggedCheater} />

      {/* Two Column Layout */}
      <div className="grid lg:grid-cols-3 gap-6">
        {/* Main Content - Stats Overview */}
        <div className="lg:col-span-2 space-y-6">
          <GradientCard>
            <PlayerStatsOverview stats={player.stats} />
          </GradientCard>

          {/* Replay History */}
          <GradientCard>
            <PlayerReplayList
              replays={replays}
              loading={replaysLoading}
              flaggedReplayIds={cheatHistory?.flaggedReplays.map((r: FlaggedReplay) => r.replayId)}
            />
          </GradientCard>
        </div>

        {/* Sidebar */}
        <div className="space-y-4">
          {/* Cheat History (032-cheat-detection) */}
          <PlayerCheatHistory history={cheatHistory ?? null} loading={cheatHistoryLoading} />

          {/* Player Info Card */}
          <GradientCard>
            <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-4">
              Player Info
            </h3>
            <div className="space-y-3">
              <InfoRow label="Platform" value={player.platform} />
              <InfoRow label="Platform ID" value={player.platformId} />
              {player.stats.totalMatches > 0 && (
                <InfoRow
                  label="Matches Tracked"
                  value={player.stats.totalMatches.toLocaleString()}
                />
              )}
            </div>
          </GradientCard>

          {/* Quick Stats */}
          <GradientCard>
            <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-4">
              Career Highlights
            </h3>
            <div className="space-y-3">
              {player.stats.avgSpeed != null && (
                <div className="flex items-center justify-between">
                  <span className="text-gray-500">Avg Speed</span>
                  <span className="text-white font-mono">
                    {player.stats.avgSpeed.toFixed(1)} km/h
                  </span>
                </div>
              )}
              {player.stats.avgAirTimePercentage != null && (
                <div className="flex items-center justify-between">
                  <span className="text-gray-500">Air Time</span>
                  <span className="text-white font-mono">
                    {player.stats.avgAirTimePercentage.toFixed(1)}%
                  </span>
                </div>
              )}
              {player.stats.avgOffensivePercentage != null && (
                <div className="flex items-center justify-between">
                  <span className="text-gray-500">Offensive</span>
                  <span className="text-white font-mono">
                    {player.stats.avgOffensivePercentage.toFixed(1)}%
                  </span>
                </div>
              )}
              {player.stats.totalShots > 0 && (
                <div className="flex items-center justify-between">
                  <span className="text-gray-500">Shot Accuracy</span>
                  <span className="text-white font-mono">
                    {((player.stats.totalGoals / player.stats.totalShots) * 100).toFixed(1)}%
                  </span>
                </div>
              )}
            </div>
          </GradientCard>
        </div>
      </div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-gray-500">{label}</span>
      <span className="text-gray-300 font-mono text-sm truncate max-w-[180px]" title={String(value)}>
        {value}
      </span>
    </div>
  );
}
