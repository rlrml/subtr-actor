/**
 * StructuredData - JSON-LD structured data component for rich search results
 * (022-seo-optimization)
 */

import { Helmet } from 'react-helmet-async';

const SITE_NAME = 'BallCam.tv';
const BASE_URL = typeof window !== 'undefined' ? window.location.origin : 'https://ballcam.tv';

interface Organization {
  '@type': 'Organization';
  name: string;
  url: string;
  logo?: string;
}

interface WebSite {
  '@type': 'WebSite';
  name: string;
  description?: string;
  url: string;
  potentialAction?: {
    '@type': 'SearchAction';
    target: string;
    'query-input': string;
  };
}

interface VideoObject {
  '@type': 'VideoObject';
  name: string;
  description: string;
  thumbnailUrl?: string;
  uploadDate: string;
  duration?: string;
  contentUrl?: string;
  embedUrl?: string;
  interactionStatistic?: {
    '@type': 'InteractionCounter';
    interactionType: { '@type': string };
    userInteractionCount: number;
  }[];
}

interface Person {
  '@type': 'Person';
  name: string;
  identifier?: string;
  url?: string;
}

interface BreadcrumbList {
  '@type': 'BreadcrumbList';
  itemListElement: {
    '@type': 'ListItem';
    position: number;
    name: string;
    item?: string;
  }[];
}

type StructuredDataType = Organization | WebSite | VideoObject | Person | BreadcrumbList;

interface StructuredDataProps {
  data: StructuredDataType | StructuredDataType[];
}

/**
 * Renders JSON-LD structured data in the document head
 */
export function StructuredData({ data }: StructuredDataProps) {
  const jsonLd = {
    '@context': 'https://schema.org',
    ...(Array.isArray(data) ? { '@graph': data } : data),
  };

  return (
    <Helmet>
      <script type="application/ld+json">
        {JSON.stringify(jsonLd)}
      </script>
    </Helmet>
  );
}

/**
 * Create structured data for a replay (video)
 */
export function createReplayStructuredData(replay: {
  id: string;
  title?: string;
  description?: string;
  mapName?: string;
  team0Score?: number;
  team1Score?: number;
  createdAt?: string;
  durationSeconds?: number;
  viewCount?: number;
  likeCount?: number;
  players?: { name: string }[];
}): VideoObject {
  const title = replay.title ||
    (replay.mapName ? `${replay.team0Score ?? 0} - ${replay.team1Score ?? 0} on ${replay.mapName}` : 'Rocket League Replay');

  const playerNames = replay.players?.map(p => p.name).join(', ') || 'Unknown players';
  const description = replay.description || `Watch this Rocket League match replay${replay.mapName ? ` on ${replay.mapName}` : ''}. Final score: ${replay.team0Score ?? 0} - ${replay.team1Score ?? 0}. Players: ${playerNames}`;

  const videoData: VideoObject = {
    '@type': 'VideoObject',
    name: title,
    description,
    uploadDate: replay.createdAt || new Date().toISOString(),
    contentUrl: `${BASE_URL}/viewer/${replay.id}`,
    embedUrl: `${BASE_URL}/viewer/${replay.id}`,
  };

  // Add duration in ISO 8601 format
  if (replay.durationSeconds) {
    const minutes = Math.floor(replay.durationSeconds / 60);
    const seconds = replay.durationSeconds % 60;
    videoData.duration = `PT${minutes}M${seconds}S`;
  }

  // Add interaction statistics
  const interactions: VideoObject['interactionStatistic'] = [];

  if (replay.viewCount !== undefined && replay.viewCount > 0) {
    interactions.push({
      '@type': 'InteractionCounter',
      interactionType: { '@type': 'WatchAction' },
      userInteractionCount: replay.viewCount,
    });
  }

  if (replay.likeCount !== undefined && replay.likeCount > 0) {
    interactions.push({
      '@type': 'InteractionCounter',
      interactionType: { '@type': 'LikeAction' },
      userInteractionCount: replay.likeCount,
    });
  }

  if (interactions.length > 0) {
    videoData.interactionStatistic = interactions;
  }

  return videoData;
}

/**
 * Create structured data for a player profile
 */
export function createPlayerStructuredData(player: {
  id: string;
  displayName: string;
  platform: string;
  platformId: string;
}): Person {
  return {
    '@type': 'Person',
    name: player.displayName,
    identifier: `${player.platform}:${player.platformId}`,
    url: `${BASE_URL}/players/${player.id}`,
  };
}

/**
 * Create WebSite structured data for the homepage
 */
export function createWebSiteStructuredData(): WebSite {
  return {
    '@type': 'WebSite',
    name: SITE_NAME,
    description: 'Watch Rocket League replays in 3D. The immersive 3D replay platform for Rocket League.',
    url: BASE_URL,
    potentialAction: {
      '@type': 'SearchAction',
      target: `${BASE_URL}/replays?search={search_term_string}`,
      'query-input': 'required name=search_term_string',
    },
  };
}

/**
 * Create Organization structured data
 */
export function createOrganizationStructuredData(): Organization {
  return {
    '@type': 'Organization',
    name: SITE_NAME,
    url: BASE_URL,
    logo: `${BASE_URL}/logo.png`,
  };
}

/**
 * Create breadcrumb structured data
 */
export function createBreadcrumbStructuredData(
  items: { name: string; url?: string }[]
): BreadcrumbList {
  return {
    '@type': 'BreadcrumbList',
    itemListElement: items.map((item, index) => ({
      '@type': 'ListItem' as const,
      position: index + 1,
      name: item.name,
      ...(item.url ? { item: item.url } : {}),
    })),
  };
}

