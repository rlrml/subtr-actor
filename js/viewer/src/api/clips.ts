// Clips API client
// Uses centralized client for authenticated endpoints with 401 retry logic

import { authenticatedFetch, publicFetch, API_URL } from './client';

// Helper to convert relative API URLs to absolute URLs for production
// In production, frontend (ballcam.tv) and API (api.ballcam.tv) are on different domains
function toAbsoluteUrl(url: string | null | undefined): string | null {
  if (!url) return null;
  // Already absolute
  if (url.startsWith('http://') || url.startsWith('https://')) return url;
  // Relative URL starting with /api - remove /api prefix since API_URL already includes it
  if (url.startsWith('/api')) {
    return `${API_URL}${url.slice(4)}`;
  }
  // Other relative URL
  return `${API_URL}${url}`;
}

// Transform clip thumbnail URLs to absolute
function transformClipUrls<T extends { thumbnailUrl?: string | null }>(clip: T): T {
  return {
    ...clip,
    thumbnailUrl: toAbsoluteUrl(clip.thumbnailUrl),
  };
}

// ============================================
// Types
// ============================================

export interface CameraFrame {
  t: number;   // Time offset in milliseconds
  px: number;  // Position X
  py: number;  // Position Y
  pz: number;  // Position Z
  qx: number;  // Quaternion X
  qy: number;  // Quaternion Y
  qz: number;  // Quaternion Z
  qw: number;  // Quaternion W
  m: 'f' | 'b' | 'p';  // Mode: free, ball, player
  tp?: number; // Target player index
}

export interface CameraRecording {
  type: 'capture';
  sampleRate: number;
  duration: number;
  frames: CameraFrame[];
}

export interface CameraKeyframe {
  id: string;
  t: number;
  px: number;
  py: number;
  pz: number;
  qx: number;
  qy: number;
  qz: number;
  qw: number;
  fov?: number;
  easing?: 'linear' | 'ease-in' | 'ease-out' | 'ease-in-out';
}

export interface CameraKeyframes {
  type: 'cinematic';
  interpolation: 'catmullrom' | 'linear';
  tension?: number;
  keyframes: CameraKeyframe[];
}

export type CameraData = CameraRecording | CameraKeyframes;

export interface ClipCreator {
  id: string;
  username: string;
  avatarUrl: string | null;
}

export interface ClipReplay {
  id: string;
  title: string | null;
  originalFilename: string;
  mapName: string | null;
  durationSeconds: number | null;
}

export interface Clip {
  id: string;
  replayId: string;
  createdBy: string;
  title: string;
  description: string | null;
  startTime: number;
  endTime: number;
  cameraMode: 'capture' | 'cinematic';
  viewCount: number;
  likeCount: number;
  commentCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface ClipEnvironment {
  id: string;
  name: string;
}

export interface ClipWithDetails extends Clip {
  creator: ClipCreator;
  replay: ClipReplay;
  cameraData: CameraData;
  environment: ClipEnvironment | null;
  isLiked?: boolean | null;
  thumbnailUrl?: string | null;
}

export interface ClipListItem extends Clip {
  creator: ClipCreator;
  thumbnailUrl?: string | null;
}

export interface ListClipsResponse {
  clips: ClipListItem[];
  total: number;
  page: number;
  totalPages: number;
  hasMore: boolean;
}

export interface ListClipsParams {
  page?: number;
  limit?: number;
  sortBy?: 'createdAt' | 'viewCount' | 'likeCount';
  sortOrder?: 'asc' | 'desc';
  createdBy?: string;
}

export interface CreateClipData {
  replayId: string;
  title: string;
  description?: string | null;
  startTime: number;
  endTime: number;
  cameraMode: 'capture' | 'cinematic';
  cameraData: CameraData;
  environmentId?: string | null;
  thumbnail?: string; // Base64 data URL (data:image/jpeg;base64,...)
}

export interface UpdateClipData {
  title?: string;
  description?: string | null;
  startTime?: number;
  endTime?: number;
  cameraData?: CameraData;
  environmentId?: string | null;
}

export interface LikeStatus {
  liked: boolean;
  likeCount: number;
}

// ============================================
// CRUD Operations (T016)
// ============================================

// Create a new clip
export async function createClip(data: CreateClipData): Promise<{ clip: ClipWithDetails }> {
  const response = await authenticatedFetch<{ clip: ClipWithDetails }>('/clips', {
    method: 'POST',
    body: JSON.stringify(data),
  });
  return { clip: transformClipUrls(response.clip) };
}

// Get clip details
export async function getClip(id: string): Promise<{ clip: ClipWithDetails }> {
  // Uses publicFetch with credentials: include for optional auth
  const response = await publicFetch<{ clip: ClipWithDetails }>(`/clips/${id}`);
  return { clip: transformClipUrls(response.clip) };
}

// List clips with pagination and filters
export async function listClips(params: ListClipsParams = {}): Promise<ListClipsResponse> {
  const searchParams = new URLSearchParams();
  if (params.page) searchParams.set('page', params.page.toString());
  if (params.limit) searchParams.set('limit', params.limit.toString());
  if (params.sortBy) searchParams.set('sortBy', params.sortBy);
  if (params.sortOrder) searchParams.set('sortOrder', params.sortOrder);
  if (params.createdBy) searchParams.set('createdBy', params.createdBy);

  const query = searchParams.toString();
  const url = query ? `/clips?${query}` : '/clips';

  const response = await publicFetch<ListClipsResponse>(url);
  return {
    ...response,
    clips: response.clips.map(transformClipUrls),
  };
}

// Update a clip
export async function updateClip(id: string, data: UpdateClipData): Promise<{ clip: ClipWithDetails }> {
  const response = await authenticatedFetch<{ clip: ClipWithDetails }>(`/clips/${id}`, {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
  return { clip: transformClipUrls(response.clip) };
}

// Delete a clip
export async function deleteClip(id: string): Promise<{ message: string }> {
  return authenticatedFetch<{ message: string }>(`/clips/${id}`, {
    method: 'DELETE',
  });
}

// List clips for a specific replay
export async function listClipsByReplay(
  replayId: string,
  params: Omit<ListClipsParams, 'createdBy'> = {}
): Promise<ListClipsResponse> {
  const searchParams = new URLSearchParams();
  if (params.page) searchParams.set('page', params.page.toString());
  if (params.limit) searchParams.set('limit', params.limit.toString());
  if (params.sortBy) searchParams.set('sortBy', params.sortBy);
  if (params.sortOrder) searchParams.set('sortOrder', params.sortOrder);

  const query = searchParams.toString();
  const url = query ? `/replays/${replayId}/clips?${query}` : `/replays/${replayId}/clips`;

  const response = await publicFetch<ListClipsResponse>(url);
  return {
    ...response,
    clips: response.clips.map(transformClipUrls),
  };
}

// List current user's clips
export async function listMyClips(
  params: Omit<ListClipsParams, 'createdBy'> = {}
): Promise<ListClipsResponse> {
  const searchParams = new URLSearchParams();
  if (params.page) searchParams.set('page', params.page.toString());
  if (params.limit) searchParams.set('limit', params.limit.toString());
  if (params.sortBy) searchParams.set('sortBy', params.sortBy);
  if (params.sortOrder) searchParams.set('sortOrder', params.sortOrder);

  const query = searchParams.toString();
  const url = query ? `/users/me/clips?${query}` : '/users/me/clips';

  const response = await authenticatedFetch<ListClipsResponse>(url);
  return {
    ...response,
    clips: response.clips.map(transformClipUrls),
  };
}

// ============================================
// Like and View Operations (T017)
// ============================================

// Get like status for a clip
export async function getLikeStatus(clipId: string): Promise<LikeStatus> {
  return authenticatedFetch<LikeStatus>(`/clips/${clipId}/like`);
}

// Toggle like on a clip
export async function toggleLike(clipId: string): Promise<LikeStatus> {
  return authenticatedFetch<LikeStatus>(`/clips/${clipId}/like`, {
    method: 'POST',
  });
}

// Record a view on a clip
export async function recordView(clipId: string): Promise<{ viewCount: number }> {
  return publicFetch<{ viewCount: number }>(`/clips/${clipId}/view`, {
    method: 'POST',
  });
}

// ============================================
// Helper Functions
// ============================================

// Format clip duration as MM:SS
export function formatClipDuration(startTime: number, endTime: number): string {
  const durationSeconds = Math.floor(endTime - startTime);
  const minutes = Math.floor(durationSeconds / 60);
  const seconds = durationSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

// Get clip duration in seconds
export function getClipDuration(clip: Clip): number {
  return clip.endTime - clip.startTime;
}

// Check if camera data is capture mode
export function isCaptureMode(cameraData: CameraData): cameraData is CameraRecording {
  return cameraData.type === 'capture';
}

// Check if camera data is cinematic mode
export function isCinematicMode(cameraData: CameraData): cameraData is CameraKeyframes {
  return cameraData.type === 'cinematic';
}
