import { useMemo } from 'react';

const SITE_NAME = 'BallCam.tv';
const DEFAULT_DESCRIPTION = 'Watch Rocket League matches live in immersive 3D. Stream your games, analyze replays, and share the experience with your team. The ultimate 3D viewer for Rocket League.';

export interface SEOConfig {
  title: string;
  description: string;
  image?: string;
  type?: 'website' | 'article' | 'video.other';
}

/**
 * Hook for generating consistent SEO metadata
 *
 * @example
 * // For a replay page
 * const seo = useSEO({
 *   replay: {
 *     team0Score: 3,
 *     team1Score: 2,
 *     mapName: 'DFH Stadium',
 *     players: [{ name: 'Player1' }, { name: 'Player2' }]
 *   }
 * });
 *
 * return <SEOHead {...seo} />;
 */

interface ReplayData {
  title?: string | null;
  team0Score?: number | null;
  team1Score?: number | null;
  mapName?: string | null;
  players?: Array<{ name: string }>;
  durationSeconds?: number | null;
}

interface PlayerData {
  displayName?: string;
  totalMatches?: number;
  totalGoals?: number;
  totalAssists?: number;
  totalSaves?: number;
}

export function useReplaySEO(replay: ReplayData | null): SEOConfig {
  return useMemo(() => {
    if (!replay) {
      return {
        title: 'Loading Replay',
        description: DEFAULT_DESCRIPTION,
        type: 'video.other' as const,
      };
    }

    const playerNames = replay.players?.map(p => p.name).join(', ') || 'Unknown players';
    const score = replay.team0Score !== undefined && replay.team1Score !== undefined
      ? `${replay.team0Score} - ${replay.team1Score}`
      : '';

    const title = replay.title
      || (score ? `Match ${score} on ${replay.mapName || 'Unknown Map'}` : `Replay on ${replay.mapName || 'Unknown Map'}`);

    const durationText = replay.durationSeconds
      ? `${Math.floor(replay.durationSeconds / 60)}m ${replay.durationSeconds % 60}s`
      : '';

    const description = `Watch this Rocket League match${score ? ` (${score})` : ''} featuring ${playerNames}${replay.mapName ? ` on ${replay.mapName}` : ''}${durationText ? `. Duration: ${durationText}` : ''}.`;

    return {
      title,
      description,
      type: 'video.other' as const,
    };
  }, [replay]);
}

export function usePlayerSEO(player: PlayerData | null): SEOConfig {
  return useMemo(() => {
    if (!player) {
      return {
        title: 'Loading Player',
        description: DEFAULT_DESCRIPTION,
        type: 'website' as const,
      };
    }

    const title = player.displayName || 'Unknown Player';

    const stats = [];
    if (player.totalMatches !== undefined) stats.push(`${player.totalMatches} matches`);
    if (player.totalGoals !== undefined) stats.push(`${player.totalGoals} goals`);
    if (player.totalAssists !== undefined) stats.push(`${player.totalAssists} assists`);
    if (player.totalSaves !== undefined) stats.push(`${player.totalSaves} saves`);

    const description = `${title} - Rocket League player statistics${stats.length > 0 ? `: ${stats.join(', ')}` : ''}. View match history and performance analysis on ${SITE_NAME}.`;

    return {
      title: `${title} - Player Profile`,
      description,
      type: 'website' as const,
    };
  }, [player]);
}

export function usePageSEO(pageTitle: string, pageDescription?: string): SEOConfig {
  return useMemo(() => ({
    title: pageTitle,
    description: pageDescription || DEFAULT_DESCRIPTION,
    type: 'website' as const,
  }), [pageTitle, pageDescription]);
}

interface ClipData {
  title?: string | null;
  description?: string | null;
  creator?: { username: string } | null;
  startTime?: number;
  endTime?: number;
  thumbnailUrl?: string | null;
}

export function useClipSEO(clip: ClipData | null): SEOConfig {
  return useMemo(() => {
    if (!clip) {
      return {
        title: 'Loading Clip',
        description: DEFAULT_DESCRIPTION,
        type: 'video.other' as const,
      };
    }

    const title = clip.title || 'Untitled Clip';
    const creatorName = clip.creator?.username || 'Unknown';

    const durationSeconds = (clip.startTime !== undefined && clip.endTime !== undefined)
      ? Math.floor(clip.endTime - clip.startTime)
      : null;
    const durationText = durationSeconds
      ? `${Math.floor(durationSeconds / 60)}m ${durationSeconds % 60}s`
      : '';

    const description = clip.description
      || `Watch this Rocket League clip by ${creatorName}${durationText ? `. Duration: ${durationText}` : ''} on ${SITE_NAME}.`;

    return {
      title,
      description,
      image: clip.thumbnailUrl || undefined,
      type: 'video.other' as const,
    };
  }, [clip]);
}

interface LiveSessionData {
  id?: string;
  title?: string | null;
  channelName?: string | null;
  broadcaster?: { username: string } | null;
  viewerCount?: number;
  thumbnailUrl?: string | null;
  gameMode?: string | null;
  mapName?: string | null;
}

export function useLiveSEO(session: LiveSessionData | null): SEOConfig {
  return useMemo(() => {
    if (!session) {
      return {
        title: 'Live Stream',
        description: `Watch Rocket League live in immersive 3D on ${SITE_NAME}. Experience matches from any angle with our unique 3D viewer.`,
        type: 'video.other' as const,
      };
    }

    const broadcasterName = session.broadcaster?.username || session.channelName || 'Unknown';
    const title = session.title || `${broadcasterName}'s Live Stream`;

    const parts = [`Watch ${broadcasterName} play Rocket League live in 3D`];
    if (session.mapName) {
      parts.push(`on ${session.mapName}`);
    }
    if (session.viewerCount && session.viewerCount > 0) {
      parts.push(`• ${session.viewerCount} watching`);
    }
    parts.push(`on ${SITE_NAME}`);

    const description = parts.join(' ');

    return {
      title,
      description,
      image: session.thumbnailUrl || undefined,
      type: 'video.other' as const,
    };
  }, [session]);
}
