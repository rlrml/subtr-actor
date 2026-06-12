/**
 * useReplayStats - Hook for fetching replay stats
 * (018-stats-compiler)
 */

import { useState, useEffect, useCallback } from 'react';
import { api } from '@/services/api';
import type { ReplayStatsData } from '@/components/stats';

interface UseReplayStatsResult {
  data: ReplayStatsData | null;
  loading: boolean;
  error: string | null;
  refetch: () => void;
}

export function useReplayStats(replayId: string | undefined): UseReplayStatsResult {
  const [data, setData] = useState<ReplayStatsData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    if (!replayId) {
      setData(null);
      setLoading(false);
      setError(null);
      return;
    }

    const fetchStats = async () => {
      setLoading(true);
      setError(null);

      try {
        const result = await api.get<ReplayStatsData>(`/replays/${replayId}/stats`);
        setData(result);
      } catch (err) {
        const apiError = err as { message?: string };
        setError(apiError.message || 'Failed to load stats');
        setData(null);
      } finally {
        setLoading(false);
      }
    };

    fetchStats();
  }, [replayId, refreshKey]);

  const refetch = useCallback(() => setRefreshKey(k => k + 1), []);

  return { data, loading, error, refetch };
}
