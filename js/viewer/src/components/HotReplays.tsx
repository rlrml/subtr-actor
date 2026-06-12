import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Flame, ArrowRight, Loader2 } from 'lucide-react';
import { HotReplayCard } from './HotReplayCard';
import { GradientButton } from '@/components/ui/GradientButton';
import { api } from '@/services/api';

interface Player {
  id: string;
  name: string;
  team: number;
  goals?: number | null;
  assists?: number | null;
  saves?: number | null;
  shots?: number | null;
  score?: number | null;
}

interface ReplayOwner {
  id: string;
  username: string;
  avatarUrl: string | null;
}

interface Replay {
  id: string;
  originalFilename?: string;
  title?: string | null;
  visibility?: 'public' | 'unlisted';
  mapName?: string | null;
  gameMode?: string | null;
  team0Score?: number | null;
  team1Score?: number | null;
  durationSeconds?: number | null;
  playedAt?: string | null;
  status: string;
  ownerId?: string | null;
  owner?: ReplayOwner | null;
  players: Player[];
  likeCount?: number;
  viewCount?: number;
}

interface ReplaysResponse {
  replays: Replay[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

export function HotReplays() {
  const [replays, setReplays] = useState<Replay[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchHotReplays = async () => {
      try {
        setLoading(true);
        const response = await api.get<ReplaysResponse>(
          '/replays?page=1&limit=3&sortBy=viewCount&sortOrder=desc'
        );
        // Filter only ready replays with at least 1 view
        const hotReplays = response.replays.filter(
          (r) => r.status === 'ready' && (r.viewCount ?? 0) > 0
        );
        setReplays(hotReplays);
      } catch (err) {
        console.error('Failed to fetch hot replays:', err);
        setError('Failed to load hot replays');
      } finally {
        setLoading(false);
      }
    };

    fetchHotReplays();
  }, []);

  // Don't render the section if no hot replays or error
  if (error || (!loading && replays.length === 0)) {
    return null;
  }

  return (
    <section className="relative">
      <div className="absolute inset-0 bg-gradient-to-r from-orange-600/5 via-red-600/5 to-orange-600/5 rounded-3xl" />
      <div className="relative rounded-3xl border border-orange-500/20 p-8 lg:p-12">
        {/* Header */}
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 mb-8">
          <div>
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-orange-500/10 border border-orange-500/20 text-orange-400 text-sm mb-4">
              <Flame className="w-4 h-4" />
              Community Favorites
            </div>
            <h2 className="text-3xl font-bold">
              <span className="text-white">Hot </span>
              <span className="bg-gradient-to-r from-orange-400 to-red-400 bg-clip-text text-transparent">
                Replays
              </span>
            </h2>
            <p className="text-gray-400 mt-2">
              The most viewed replays from the community
            </p>
          </div>
          <Link to="/replays?sortBy=viewCount">
            <GradientButton variant="outline" className="border-orange-500/30 hover:border-orange-500/50">
              <span className="text-orange-400">View All</span>
              <ArrowRight className="w-4 h-4 text-orange-400" />
            </GradientButton>
          </Link>
        </div>

        {/* Content */}
        {loading ? (
          <div className="flex items-center justify-center py-16">
            <div className="flex flex-col items-center gap-3">
              <Loader2 className="w-8 h-8 text-orange-400 animate-spin" />
              <span className="text-gray-400 text-sm">Loading hot replays...</span>
            </div>
          </div>
        ) : (
          <div className="flex flex-col gap-3">
            {replays.map((replay, index) => (
              <HotReplayCard key={replay.id} replay={replay} rank={index + 1} />
            ))}
          </div>
        )}
      </div>
    </section>
  );
}
