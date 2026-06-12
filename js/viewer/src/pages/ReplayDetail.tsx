import { useState, useEffect, useCallback } from 'react';
import { useParams, Link } from 'react-router-dom';
import { SEOHead, StructuredData, createReplayStructuredData } from '@/components/SEO';
import { useReplaySEO } from '@/hooks/useSEO';
import { ArrowLeft, Play, Loader2, Trash2, AlertCircle, Clock, MapPin, Target, RefreshCw, Download, User, Pencil, Check, X, FileJson, Eye } from 'lucide-react';
import { api } from '@/services/api';
import { GradientCard } from '@/components/ui/GradientCard';
import { GradientButton } from '@/components/ui/GradientButton';
import { useAuth } from '@/hooks/useAuth';
import { useReplayStats } from '@/hooks/useReplayStats';
import { CommentList } from '@/components/comments';
import { StatsPanel } from '@/components/stats';
import { PlayerLink } from '@/components/player';
import { ReplayClips } from '@/components/clips';
import { LikeButton } from '@/components/LikeButton';
import { VisibilityToggle } from '@/components/VisibilityToggle';
import { VisibilityBadge } from '@/components/VisibilityBadge';
import { TeamSizeBadge } from '@/components/TeamSizeBadge';
import { TechnicalInfoSection } from '@/components/TechnicalInfoSection';
import { QualityIndicator } from '@/components/QualityIndicator';
import { CheatDetectionAlert } from '@/components/cheat/CheatDetectionAlert';
import { CheatDetectionPanel } from '@/components/cheat/CheatDetectionPanel';
import { useReplayCheatAnalysis } from '@/api/cheat';
import { RelatedReplays } from '@/components/replay/RelatedReplays';
import { getDisplayTitle } from '@/utils/replay';
import { toast } from 'sonner';
import type { QualityMetrics } from '@/types/quality';

interface Player {
  id: string;
  name: string;
  team: number;
  goals?: number;
  assists?: number;
  saves?: number;
  shots?: number;
  score?: number;
  platform?: string;
  platformId?: string;
  playerId?: string | null; // ID from players table (for profile linking)
}

interface ReplayOwner {
  id: string;
  username: string;
  avatarUrl: string | null;
}

interface Replay {
  id: string;
  originalFilename: string;
  title?: string | null;
  visibility?: 'public' | 'unlisted';
  mapName?: string;
  gameMode?: string;
  matchType?: string;
  matchGuid?: string;
  teamSize?: number;
  team0Score?: number;
  team1Score?: number;
  durationSeconds?: number;
  playedAt?: string;
  status: string;
  errorMessage?: string;
  frameworkVersion?: string;
  retryCount?: number;
  players: Player[];
  ownerId?: string | null;
  owner?: ReplayOwner | null;
  // Technical version info (015-replay-data-extraction)
  gameVersion?: number | null;
  buildId?: number | null;
  buildVersion?: string | null;
  headerSize?: number | null;
  headerCrc?: number | null;
  majorVersion?: number | null;
  minorVersion?: number | null;
  netVersion?: number | null;
  rlGameType?: string | null;
  replayName?: string | null;
  hadOvertime?: boolean | null;
  // Quality indicator (016-replay-quality-indicator)
  qualityScore?: number | null;
  qualityMetrics?: QualityMetrics | null;
  // Engagement counters (017-ux-polish-batch)
  viewCount?: number;
  likeCount?: number;
}

const MAX_RETRY_COUNT = 3;

interface RelatedReplay {
  id: string;
  title: string | null;
  qualityScore: number | null;
  createdAt: string;
}

interface ReplayResponse {
  replay: Replay;
  currentFrameworkVersion?: string;
  needsRecompilation?: boolean;
  relatedReplays?: RelatedReplay[];
}

function formatDuration(seconds?: number): string {
  if (!seconds) return '--:--';
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

function formatDate(dateStr?: string): string {
  if (!dateStr) return 'Unknown date';
  const date = new Date(dateStr);
  return date.toLocaleDateString('en-US', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function getMapDisplayName(mapName?: string): string {
  if (!mapName) return 'Unknown map';
  const cleaned = mapName.replace(/_P$/, '').replace(/_Standard$/, '').replace(/_/g, ' ');
  return cleaned;
}

export default function ReplayDetail() {
  const { id } = useParams<{ id: string }>();
  const { user } = useAuth();
  const [replay, setReplay] = useState<Replay | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();
  const [deleting, setDeleting] = useState(false);
  const [needsRecompilation, setNeedsRecompilation] = useState(false);
  const [currentFrameworkVersion, setCurrentFrameworkVersion] = useState<string>();
  const [retrying, setRetrying] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);
  const [forceRecompiling, setForceRecompiling] = useState(false);
  // Title editing state
  const [isEditingTitle, setIsEditingTitle] = useState(false);
  const [editedTitle, setEditedTitle] = useState('');
  const [savingTitle, setSavingTitle] = useState(false);

  // Related replays (033-replay-duplicate-detection / T034)
  const [relatedReplays, setRelatedReplays] = useState<RelatedReplay[]>([]);

  // Advanced stats (018-stats-compiler)
  const { data: statsData, loading: statsLoading, error: statsError, refetch: refetchStats } = useReplayStats(id);

  // Cheat detection analysis (032-cheat-detection)
  const { data: cheatAnalysis, refetch: refetchCheatAnalysis } = useReplayCheatAnalysis(id);

  // SEO metadata (022-seo-optimization)
  const seoData = useReplaySEO(replay);

  // Track previous status to detect when recompilation finishes
  const [prevStatus, setPrevStatus] = useState<string | undefined>();

  // Detect when recompilation finishes and refetch stats
  useEffect(() => {
    if (replay?.status && prevStatus) {
      // Detect transition from recompiling/processing to ready
      const wasCompiling = prevStatus === 'recompiling' || prevStatus === 'processing';
      const isNowReady = replay.status === 'ready';

      if (wasCompiling && isNowReady) {
        console.log('[ReplayDetail] Recompilation finished, refetching stats...');
        refetchStats();
      }
    }
    setPrevStatus(replay?.status);
  }, [replay?.status, prevStatus, refetchStats]);

  useEffect(() => {
    if (!id) return;

    const fetchReplay = async (showLoader = true) => {
      try {
        if (showLoader) setLoading(true);
        setError(undefined);
        const data = await api.get<ReplayResponse>(`/replays/${id}`);
        setReplay(data.replay);
        setNeedsRecompilation(data.needsRecompilation ?? false);
        setCurrentFrameworkVersion(data.currentFrameworkVersion);
        // 033-replay-duplicate-detection (T034): Set related replays
        setRelatedReplays(data.relatedReplays ?? []);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load replay');
      } finally {
        if (showLoader) setLoading(false);
      }
    };

    // Only show loader on initial load (when we don't have a replay yet)
    fetchReplay(!replay);

    // Poll for updates if processing, recompiling, or needs recompilation (recompilation just triggered)
    let pollInterval: number | undefined;
    if (replay?.status === 'processing' || replay?.status === 'recompiling' || (needsRecompilation && replay?.status === 'ready')) {
      pollInterval = window.setInterval(() => fetchReplay(false), 2000);
    }

    return () => {
      if (pollInterval) clearInterval(pollInterval);
    };
  }, [id, replay?.status, needsRecompilation, refreshKey]);

  const handleDelete = async () => {
    if (!id || !confirm('Are you sure you want to delete this replay?')) return;

    try {
      setDeleting(true);
      await api.delete(`/replays/${id}`);
      window.location.href = '/replays';
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete replay');
      setDeleting(false);
    }
  };

  const handleRetry = async () => {
    if (!id || !replay) return;

    try {
      setRetrying(true);
      setError(undefined);

      // Use reprocess if no frameworkVersion (parsing failed), otherwise recompile
      const endpoint = replay.frameworkVersion
        ? `/replays/${id}/recompile`
        : `/replays/${id}/reprocess`;

      await api.post(endpoint);
      // Trigger a re-fetch to pick up the new status
      setRefreshKey(k => k + 1);
    } catch (err: unknown) {
      const apiError = err as { message?: string; statusCode?: number };
      setError(apiError.message || 'Failed to start retry');
      // Refresh to update retry count if quota was reached
      if (apiError.statusCode === 429) {
        setRefreshKey(k => k + 1);
      }
    } finally {
      setRetrying(false);
    }
  };

  const handleStartEditTitle = () => {
    setEditedTitle(replay?.title || '');
    setIsEditingTitle(true);
  };

  const handleCancelEditTitle = () => {
    setIsEditingTitle(false);
    setEditedTitle('');
  };

  const handleSaveTitle = async () => {
    if (!id || !replay) return;

    setSavingTitle(true);
    try {
      const newTitle = editedTitle.trim() || null;
      await api.patch(`/replays/${id}`, { title: newTitle });
      setReplay({ ...replay, title: newTitle });
      setIsEditingTitle(false);
      toast.success('Title updated');
    } catch (err) {
      toast.error('Failed to update title');
    } finally {
      setSavingTitle(false);
    }
  };

  const handleVisibilityChange = useCallback(async (newVisibility: 'public' | 'unlisted') => {
    if (!id || !replay) return;

    const oldVisibility = replay.visibility;
    // Optimistic update
    setReplay({ ...replay, visibility: newVisibility });

    try {
      await api.patch(`/replays/${id}`, { visibility: newVisibility });
      toast.success(`Replay is now ${newVisibility}`);
    } catch (err) {
      // Revert on error
      setReplay({ ...replay, visibility: oldVisibility });
      toast.error('Failed to update visibility');
    }
  }, [id, replay]);

  const handleForceRecompile = async () => {
    if (!id) return;

    try {
      setForceRecompiling(true);
      await api.post(`/replays/${id}/force-recompile`);
      toast.success('Force recompilation started');
      setRefreshKey(k => k + 1);
    } catch (err: unknown) {
      const apiError = err as { message?: string };
      toast.error(apiError.message || 'Failed to start force recompilation');
    } finally {
      setForceRecompiling(false);
    }
  };

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-violet-600/20 to-blue-600/20 flex items-center justify-center mb-4">
          <Loader2 className="w-8 h-8 animate-spin text-violet-500" />
        </div>
        <p className="text-gray-400">Loading replay...</p>
      </div>
    );
  }

  if (error || !replay) {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <div className="w-16 h-16 rounded-2xl bg-red-500/10 flex items-center justify-center mb-4">
          <AlertCircle className="w-8 h-8 text-red-400" />
        </div>
        <p className="text-red-400 mb-4">{error || 'Replay not found'}</p>
        <Link to="/replays" className="text-violet-400 hover:text-violet-300 underline">
          Back to replays
        </Link>
      </div>
    );
  }

  const isReady = replay.status === 'ready';
  const isError = replay.status === 'error';
  const isProcessing = replay.status === 'processing';
  const isRecompiling = replay.status === 'recompiling';
  const isOwner = user && replay.owner && user.id === replay.owner.id;

  const team0Players = replay.players.filter(p => p.team === 0);
  const team1Players = replay.players.filter(p => p.team === 1);

  // Determine winner for visual emphasis
  const blueWon = (replay.team0Score ?? 0) > (replay.team1Score ?? 0);
  const orangeWon = (replay.team1Score ?? 0) > (replay.team0Score ?? 0);

  // Structured data for video (replay) rich results
  const replayStructuredData = replay ? createReplayStructuredData({
    id: replay.id,
    title: replay.title ?? undefined,
    mapName: replay.mapName,
    team0Score: replay.team0Score,
    team1Score: replay.team1Score,
    createdAt: replay.playedAt || new Date().toISOString(),
    durationSeconds: replay.durationSeconds,
    viewCount: replay.viewCount,
    likeCount: replay.likeCount,
    players: replay.players.map(p => ({ name: p.name })),
  }) : null;

  return (
    <div className="space-y-6">
      <SEOHead {...seoData} />
      {replayStructuredData && <StructuredData data={replayStructuredData} />}
      {/* Back navigation */}
      <Link
        to="/replays"
        className="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors group"
      >
        <ArrowLeft className="w-4 h-4 group-hover:-translate-x-1 transition-transform" />
        Back to replays
      </Link>

      {/* Score Banner - Full Width Hero */}
      <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-gray-900 via-gray-900 to-gray-900 border border-gray-700/50">
        {/* Animated background gradient based on winner */}
        <div className={`absolute inset-0 ${
          blueWon
            ? 'bg-gradient-to-r from-blue-600/20 via-transparent to-transparent'
            : orangeWon
              ? 'bg-gradient-to-l from-orange-600/20 via-transparent to-transparent'
              : 'bg-gradient-to-r from-blue-600/10 via-violet-600/10 to-orange-600/10'
        }`} />

        {/* Subtle grid pattern overlay */}
        <div className="absolute inset-0 opacity-5" style={{
          backgroundImage: `url("data:image/svg+xml,%3Csvg width='60' height='60' viewBox='0 0 60 60' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='none' fill-rule='evenodd'%3E%3Cg fill='%23ffffff' fill-opacity='1'%3E%3Cpath d='M36 34v-4h-2v4h-4v2h4v4h2v-4h4v-2h-4zm0-30V0h-2v4h-4v2h4v4h2V6h4V4h-4zM6 34v-4H4v4H0v2h4v4h2v-4h4v-2H6zM6 4V0H4v4H0v2h4v4h2V6h4V4H6z'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E")`
        }} />

        <div className="relative p-6 sm:p-8">
          {/* Title Row */}
          <div className="flex items-center justify-center gap-3 mb-6">
            {isEditingTitle ? (
              <div className="flex items-center gap-2">
                <input
                  type="text"
                  value={editedTitle}
                  onChange={(e) => setEditedTitle(e.target.value)}
                  placeholder="Enter title (optional)"
                  maxLength={100}
                  className="px-4 py-2 rounded-lg bg-gray-800/80 border border-gray-600 text-white text-xl font-bold text-center focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent min-w-[300px]"
                  autoFocus
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handleSaveTitle();
                    if (e.key === 'Escape') handleCancelEditTitle();
                  }}
                />
                <button
                  onClick={handleSaveTitle}
                  disabled={savingTitle}
                  className="p-2 rounded-lg bg-green-500/20 text-green-400 hover:bg-green-500/30 transition-colors disabled:opacity-50"
                  title="Save (Enter)"
                >
                  {savingTitle ? <Loader2 className="w-5 h-5 animate-spin" /> : <Check className="w-5 h-5" />}
                </button>
                <button
                  onClick={handleCancelEditTitle}
                  className="p-2 rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors"
                  title="Cancel (Esc)"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>
            ) : (
              <>
                <h1 className="text-2xl sm:text-3xl font-bold text-white">{getDisplayTitle(replay.title, replay.originalFilename)}</h1>
                {isOwner && (
                  <button
                    onClick={handleStartEditTitle}
                    className="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-gray-700/50 transition-colors"
                    title="Edit title"
                  >
                    <Pencil className="w-4 h-4" />
                  </button>
                )}
              </>
            )}
          </div>

          {/* Score Display */}
          <div className="flex items-start justify-center gap-4 sm:gap-8 lg:gap-12">
            {/* Blue Team */}
            <div className={`relative flex-1 max-w-xs text-center p-4 sm:p-6 rounded-xl transition-all ${
              blueWon
                ? 'bg-blue-500/20 border-2 border-blue-400/50 shadow-lg shadow-blue-500/20'
                : 'bg-blue-500/5 border border-blue-500/20'
            }`}>
              {blueWon && (
                <div className="absolute -top-3 left-1/2 -translate-x-1/2 text-yellow-400 text-2xl">👑</div>
              )}
              <div className={`text-5xl sm:text-6xl lg:text-7xl font-black ${
                blueWon ? 'text-blue-300' : 'text-blue-400/70'
              } drop-shadow-[0_0_30px_rgba(96,165,250,0.5)]`}>
                {replay.team0Score ?? '-'}
              </div>
              <div className={`text-sm mt-3 font-semibold uppercase tracking-wider ${
                blueWon ? 'text-blue-300' : 'text-blue-400/60'
              }`}>Blue Team</div>
              <div className="mt-3 flex flex-col items-center gap-1">
                {team0Players.map((player) => (
                  <PlayerLink
                    key={player.id}
                    playerId={player.playerId || undefined}
                    name={player.name}
                    platform={player.platform}
                    platformId={player.platformId}
                    team={player.team}
                    className="text-xs"
                    showPlatform
                    compact
                  />
                ))}
              </div>
            </div>

            {/* VS / Match Info Center */}
            <div className="flex flex-col items-center justify-center px-2 sm:px-4 pt-4">
              <div className="text-xl sm:text-2xl text-gray-600 font-light mb-3">VS</div>

              <div className="flex flex-col items-center gap-2">
                <div className="flex items-center gap-2 text-gray-400 text-sm">
                  <Clock className="w-4 h-4" />
                  <span className="font-mono">{formatDuration(replay.durationSeconds)}</span>
                </div>

                {replay.hadOvertime && (
                  <div className="px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider bg-gradient-to-r from-red-500 to-orange-500 text-white shadow-lg shadow-red-500/30 animate-pulse">
                    ⏱ Overtime
                  </div>
                )}

                <div className="flex items-center gap-2 mt-1">
                  <TeamSizeBadge teamSize={replay.teamSize} playerCount={replay.players.length} />
                </div>
              </div>
            </div>

            {/* Orange Team */}
            <div className={`relative flex-1 max-w-xs text-center p-4 sm:p-6 rounded-xl transition-all ${
              orangeWon
                ? 'bg-orange-500/20 border-2 border-orange-400/50 shadow-lg shadow-orange-500/20'
                : 'bg-orange-500/5 border border-orange-500/20'
            }`}>
              {orangeWon && (
                <div className="absolute -top-3 left-1/2 -translate-x-1/2 text-yellow-400 text-2xl">👑</div>
              )}
              <div className={`text-5xl sm:text-6xl lg:text-7xl font-black ${
                orangeWon ? 'text-orange-300' : 'text-orange-400/70'
              } drop-shadow-[0_0_30px_rgba(251,146,60,0.5)]`}>
                {replay.team1Score ?? '-'}
              </div>
              <div className={`text-sm mt-3 font-semibold uppercase tracking-wider ${
                orangeWon ? 'text-orange-300' : 'text-orange-400/60'
              }`}>Orange Team</div>
              <div className="mt-3 flex flex-col items-center gap-1">
                {team1Players.map((player) => (
                  <PlayerLink
                    key={player.id}
                    playerId={player.playerId || undefined}
                    name={player.name}
                    platform={player.platform}
                    platformId={player.platformId}
                    team={player.team}
                    className="text-xs"
                    showPlatform
                    compact
                  />
                ))}
              </div>
            </div>
          </div>

          {/* Map name badge */}
          <div className="flex justify-center mt-6">
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-gray-800/60 border border-gray-700/50 text-sm">
              <MapPin className="w-4 h-4 text-violet-400" />
              <span className="text-gray-300">{getMapDisplayName(replay.mapName)}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Status Banners */}
      {isProcessing && (
        <div className="flex items-center gap-4 p-4 rounded-xl bg-yellow-500/10 border border-yellow-500/20">
          <div className="w-10 h-10 rounded-lg bg-yellow-500/20 flex items-center justify-center">
            <Loader2 className="w-5 h-5 text-yellow-400 animate-spin" />
          </div>
          <div>
            <div className="font-medium text-yellow-400">Processing</div>
            <div className="text-sm text-gray-400">The replay is being compiled, please wait...</div>
          </div>
        </div>
      )}

      {isError && (
        <div className="flex items-center gap-4 p-4 rounded-xl bg-red-500/10 border border-red-500/20">
          <div className="w-10 h-10 rounded-lg bg-red-500/20 flex items-center justify-center">
            <AlertCircle className="w-5 h-5 text-red-400" />
          </div>
          <div className="flex-1">
            <div className="font-medium text-red-400">Error</div>
            <div className="text-sm text-gray-400">{replay.errorMessage || 'An error occurred during processing.'}</div>
          </div>
          {/* Show retry button with quota info */}
          {(replay.retryCount ?? 0) < MAX_RETRY_COUNT ? (
            <button
              onClick={handleRetry}
              disabled={retrying}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30 disabled:opacity-50 transition-all border border-red-500/30"
            >
              <RefreshCw className={`w-4 h-4 ${retrying ? 'animate-spin' : ''}`} />
              {retrying ? 'Retrying...' : `Retry (${(replay.retryCount ?? 0) + 1}/${MAX_RETRY_COUNT})`}
            </button>
          ) : (
            <div className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-500/20 text-gray-400 border border-gray-500/30">
              <AlertCircle className="w-4 h-4" />
              Max retries reached
            </div>
          )}
        </div>
      )}

      {isRecompiling && (
        <div className="flex items-center gap-4 p-4 rounded-xl bg-violet-500/10 border border-violet-500/20">
          <div className="w-10 h-10 rounded-lg bg-violet-500/20 flex items-center justify-center">
            <RefreshCw className="w-5 h-5 text-violet-400 animate-spin" />
          </div>
          <div>
            <div className="font-medium text-violet-400">Recompiling</div>
            <div className="text-sm text-gray-400">
              Updating to framework v{currentFrameworkVersion}...
            </div>
          </div>
        </div>
      )}

      {needsRecompilation && isReady && (
        <div className="flex items-center gap-4 p-4 rounded-xl bg-amber-500/10 border border-amber-500/20">
          <div className="w-10 h-10 rounded-lg bg-amber-500/20 flex items-center justify-center">
            <RefreshCw className="w-5 h-5 text-amber-400 animate-spin" />
          </div>
          <div className="flex-1">
            <div className="font-medium text-amber-400">Updating...</div>
            <div className="text-sm text-gray-400">
              Recompiling from v{replay.frameworkVersion} to v{currentFrameworkVersion}...
            </div>
          </div>
        </div>
      )}

      {/* Cheat Detection Alert (032-cheat-detection) */}
      {cheatAnalysis && cheatAnalysis.status === 'completed' && cheatAnalysis.hasCheater && (
        <CheatDetectionAlert
          status={cheatAnalysis.status}
          hasCheater={cheatAnalysis.hasCheater}
          cheaterCount={cheatAnalysis.cheaterCount}
          error={cheatAnalysis.error}
        />
      )}

      {/* Two Column Layout: Main Content + Sidebar */}
      <div className="flex flex-col lg:flex-row gap-6">
        {/* Main Content Area */}
        <div className="flex-1 space-y-6 min-w-0">
          {/* Quality Indicator - shown when replay is ready and has quality data */}
          {isReady && replay.qualityScore !== undefined && replay.qualityScore !== null && (
            <QualityIndicator
              score={replay.qualityScore}
              metrics={replay.qualityMetrics}
              showDetails={true}
            />
          )}

          {/* Advanced Stats Panel (018-stats-compiler) */}
          {isReady && (
            <GradientCard>
              <StatsPanel
                data={statsData}
                loading={statsLoading}
                error={statsError}
              />
            </GradientCard>
          )}

          {/* Clips Section (024-clip-system) */}
          <GradientCard>
            <ReplayClips replayId={id!} isReady={isReady && !needsRecompilation} />
          </GradientCard>

          {/* Comments Section */}
          <GradientCard>
            <CommentList entityType="replay" entityId={id!} />
          </GradientCard>
        </div>

        {/* Sidebar */}
        <div className="lg:w-80 shrink-0">
          <div className="lg:sticky lg:top-4 space-y-4">
            {/* Primary Actions */}
            <GradientCard className="border-violet-500/20">
              <div className="space-y-3">
                {/* Watch Button */}
                {isReady && !needsRecompilation && (
                  <Link to={`/viewer/${id}`} className="block">
                    <GradientButton size="lg" className="w-full justify-center">
                      <Play className="w-5 h-5" />
                      Watch Replay
                    </GradientButton>
                  </Link>
                )}
                {isReady && needsRecompilation && (
                  <GradientButton size="lg" disabled className="w-full justify-center opacity-50 cursor-not-allowed">
                    <RefreshCw className="w-5 h-5 animate-spin" />
                    Updating...
                  </GradientButton>
                )}
                {(isProcessing || isRecompiling) && (
                  <div className="flex items-center justify-center gap-2 py-3 px-4 rounded-xl bg-violet-500/10 text-violet-400 border border-violet-500/20">
                    <Loader2 className="w-5 h-5 animate-spin" />
                    {isProcessing ? 'Processing...' : 'Recompiling...'}
                  </div>
                )}

                {/* Like Button */}
                <div className="flex items-center justify-center">
                  <LikeButton replayId={id!} size="lg" showCount />
                </div>

                {/* Download buttons */}
                <div className="flex gap-2">
                  <a
                    href={`${import.meta.env.VITE_API_URL || '/api'}/replays/${id}/download`}
                    download
                    className="flex-1 flex items-center justify-center gap-2 p-3 rounded-xl bg-gray-500/10 text-gray-400 hover:bg-gray-500/20 hover:text-white transition-all border border-gray-500/20"
                    title="Download .replay file"
                  >
                    <Download className="w-4 h-4" />
                    <span className="text-sm">.replay</span>
                  </a>
                  <a
                    href={`${import.meta.env.VITE_API_URL || '/api'}/replays/${id}/json`}
                    download
                    className="flex-1 flex items-center justify-center gap-2 p-3 rounded-xl bg-amber-500/10 text-amber-400 hover:bg-amber-500/20 hover:text-amber-300 transition-all border border-amber-500/20"
                    title="Download parsed JSON"
                  >
                    <FileJson className="w-4 h-4" />
                    <span className="text-sm">.json</span>
                  </a>
                </div>

                {/* Owner actions */}
                {isOwner && (
                  <button
                    onClick={handleDelete}
                    disabled={deleting}
                    className="w-full flex items-center justify-center gap-2 p-3 rounded-xl bg-red-500/10 text-red-400 hover:bg-red-500/20 disabled:opacity-50 transition-all border border-red-500/20"
                  >
                    <Trash2 className="w-4 h-4" />
                    <span className="text-sm">{deleting ? 'Deleting...' : 'Delete Replay'}</span>
                  </button>
                )}
              </div>
            </GradientCard>

            {/* Uploader & Stats */}
            <GradientCard>
              <div className="space-y-4">
                {/* Uploader */}
                <div className="flex items-center gap-3">
                  <div className="w-12 h-12 rounded-full bg-gradient-to-br from-violet-500 to-blue-500 flex items-center justify-center overflow-hidden">
                    {replay.owner?.avatarUrl ? (
                      <img src={replay.owner.avatarUrl} alt={replay.owner.username} className="w-full h-full object-cover" />
                    ) : (
                      <User className="w-6 h-6 text-white" />
                    )}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="font-medium text-white truncate">
                      {replay.owner?.username || 'Anonymous'}
                    </div>
                    <div className="text-xs text-gray-500">Uploader</div>
                  </div>
                </div>

                {/* Visibility */}
                {isOwner ? (
                  <div className="flex items-center justify-between p-3 rounded-lg bg-gray-800/50">
                    <span className="text-sm text-gray-400">Visibility</span>
                    <VisibilityToggle
                      visibility={replay.visibility ?? 'public'}
                      onToggle={handleVisibilityChange}
                    />
                  </div>
                ) : replay.visibility === 'unlisted' && (
                  <div className="flex items-center justify-center">
                    <VisibilityBadge visibility="unlisted" />
                  </div>
                )}

                {/* Stats Grid */}
                <div className="grid grid-cols-2 gap-3">
                  <div className="p-3 rounded-lg bg-gray-800/50 text-center">
                    <div className="flex items-center justify-center gap-1 text-blue-400 mb-1">
                      <Eye className="w-4 h-4" />
                    </div>
                    <div className="text-lg font-bold text-white">{(replay.viewCount ?? 0).toLocaleString()}</div>
                    <div className="text-xs text-gray-500">Views</div>
                  </div>
                  <div className="p-3 rounded-lg bg-gray-800/50 text-center">
                    <div className="flex items-center justify-center gap-1 text-red-400 mb-1">
                      <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
                      </svg>
                    </div>
                    <div className="text-lg font-bold text-white">{(replay.likeCount ?? 0).toLocaleString()}</div>
                    <div className="text-xs text-gray-500">Likes</div>
                  </div>
                </div>
              </div>
            </GradientCard>

            {/* Match Info */}
            <GradientCard>
              <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">Match Info</h3>
              <div className="space-y-3">
                <InfoRow icon={<Clock className="w-4 h-4 text-violet-400" />} label="Played" value={formatDate(replay.playedAt)} />
                <InfoRow
                  icon={<Clock className="w-4 h-4 text-green-400" />}
                  label="Duration"
                  value={
                    <span className="flex items-center gap-2">
                      {formatDuration(replay.durationSeconds)}
                      {replay.hadOvertime && (
                        <span className="px-1.5 py-0.5 rounded-full text-[10px] font-semibold bg-gradient-to-r from-red-500 to-orange-500 text-white">OT</span>
                      )}
                    </span>
                  }
                />
                <InfoRow icon={<Target className="w-4 h-4 text-blue-400" />} label="Type" value={replay.matchType || replay.gameMode || '-'} />
              </div>
            </GradientCard>

            {/* Cheat Detection Panel (032-cheat-detection) */}
            {cheatAnalysis && (
              <CheatDetectionPanel
                analysis={cheatAnalysis}
                replayId={id}
                onAnalysisRequested={refetchCheatAnalysis}
              />
            )}

            {/* Related Replays (033-replay-duplicate-detection / T034) */}
            {relatedReplays.length > 0 && (
              <RelatedReplays relatedReplays={relatedReplays} />
            )}

            {/* Technical Info */}
            <GradientCard>
              <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">Technical</h3>
              <div className="space-y-2 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-gray-500">File</span>
                  <span className="text-gray-300 truncate max-w-[150px]" title={replay.originalFilename}>
                    {replay.originalFilename}
                  </span>
                </div>
                {replay.frameworkVersion && (
                  <div className="flex items-center justify-between">
                    <span className="text-gray-500">Compiler</span>
                    <span className={needsRecompilation ? 'text-amber-400' : 'text-gray-300'}>
                      v{replay.frameworkVersion}
                      {needsRecompilation && currentFrameworkVersion && (
                        <span className="text-gray-600"> → v{currentFrameworkVersion}</span>
                      )}
                    </span>
                  </div>
                )}

                {/* Admin force recompile */}
                {user?.isAdmin && !isProcessing && !isRecompiling && (
                  <button
                    onClick={handleForceRecompile}
                    disabled={forceRecompiling}
                    className="w-full mt-2 flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-violet-500/20 text-violet-400 hover:bg-violet-500/30 disabled:opacity-50 transition-all border border-violet-500/30 text-xs"
                  >
                    <RefreshCw className={`w-3 h-3 ${forceRecompiling ? 'animate-spin' : ''}`} />
                    {forceRecompiling ? 'Recompiling...' : 'Force Recompile'}
                  </button>
                )}
              </div>

              {/* Expandable Technical Info */}
              <div className="mt-3 pt-3 border-t border-gray-700/50">
                <TechnicalInfoSection
                  gameVersion={replay.gameVersion}
                  buildId={replay.buildId}
                  buildVersion={replay.buildVersion}
                  headerSize={replay.headerSize}
                  headerCrc={replay.headerCrc}
                  majorVersion={replay.majorVersion}
                  minorVersion={replay.minorVersion}
                  netVersion={replay.netVersion}
                  rlGameType={replay.rlGameType}
                  compact
                />
              </div>
            </GradientCard>
          </div>
        </div>
      </div>
    </div>
  );
}

function InfoRow({ icon, label, value }: { icon: React.ReactNode; label: string; value: React.ReactNode }) {
  return (
    <div className="flex items-center gap-3">
      <div className="w-8 h-8 rounded-lg bg-gray-800/50 flex items-center justify-center shrink-0">
        {icon}
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-xs text-gray-500">{label}</div>
        <div className="text-sm text-white">{value}</div>
      </div>
    </div>
  );
}
