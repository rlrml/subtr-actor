// Announcements (News) API client
// Public endpoints for listing/reading published announcements,
// and admin-only endpoints for the editorial backoffice.

import { authenticatedFetch, publicFetch } from './client';

// =====================================
// Types
// =====================================

export interface AnnouncementAuthor {
  id: string;
  username: string;
  isAdmin: boolean;
  avatarUrl: string | null;
}

export interface Announcement {
  id: string;
  slug: string;
  title: string;
  excerpt: string | null;
  contentMd: string;
  authorId: string;
  isPublished: boolean;
  publishedAt: string | null;
  viewCount: number;
  commentCount: number;
  createdAt: string;
  updatedAt: string;
  author?: AnnouncementAuthor;
}

export interface ListAnnouncementsResponse {
  items: Announcement[];
  total: number;
  hasMore: boolean;
}

export interface LatestAnnouncementsResponse {
  items: Announcement[];
}

export interface AnnouncementResponse {
  announcement: Announcement;
}

export interface CreateAnnouncementInput {
  title: string;
  contentMd: string;
  excerpt?: string;
  isPublished?: boolean;
  slug?: string;
}

export interface UpdateAnnouncementInput {
  title?: string;
  contentMd?: string;
  excerpt?: string | null;
  isPublished?: boolean;
  slug?: string;
}

export type AnnouncementStatusFilter = 'all' | 'draft' | 'published';

// =====================================
// Public endpoints
// =====================================

export async function listAnnouncements(
  params: { limit?: number; offset?: number } = {},
): Promise<ListAnnouncementsResponse> {
  const search = new URLSearchParams();
  if (params.limit !== undefined) search.set('limit', String(params.limit));
  if (params.offset !== undefined) search.set('offset', String(params.offset));
  const query = search.toString();
  const url = query ? `/announcements?${query}` : '/announcements';
  return publicFetch<ListAnnouncementsResponse>(url);
}

export async function getLatestAnnouncements(
  limit: number = 3,
): Promise<Announcement[]> {
  const search = new URLSearchParams({ limit: String(limit) });
  const response = await publicFetch<LatestAnnouncementsResponse>(
    `/announcements/latest?${search.toString()}`,
  );
  return response.items;
}

export async function getAnnouncementBySlug(slug: string): Promise<Announcement> {
  const response = await publicFetch<AnnouncementResponse>(
    `/announcements/${encodeURIComponent(slug)}`,
  );
  return response.announcement;
}

// =====================================
// Admin endpoints (require admin auth)
// =====================================

export async function listAllAnnouncementsAdmin(
  params: {
    status?: AnnouncementStatusFilter;
    limit?: number;
    offset?: number;
  } = {},
): Promise<ListAnnouncementsResponse> {
  const search = new URLSearchParams();
  if (params.status) search.set('status', params.status);
  if (params.limit !== undefined) search.set('limit', String(params.limit));
  if (params.offset !== undefined) search.set('offset', String(params.offset));
  const query = search.toString();
  const url = query
    ? `/admin/announcements?${query}`
    : '/admin/announcements';
  return authenticatedFetch<ListAnnouncementsResponse>(url);
}

export async function getAnnouncementByIdAdmin(id: string): Promise<Announcement> {
  const response = await authenticatedFetch<AnnouncementResponse>(
    `/admin/announcements/${id}`,
  );
  return response.announcement;
}

export async function createAnnouncement(
  input: CreateAnnouncementInput,
): Promise<Announcement> {
  const response = await authenticatedFetch<AnnouncementResponse>(
    '/admin/announcements',
    {
      method: 'POST',
      body: JSON.stringify(input),
    },
  );
  return response.announcement;
}

export async function updateAnnouncement(
  id: string,
  partial: UpdateAnnouncementInput,
): Promise<Announcement> {
  const response = await authenticatedFetch<AnnouncementResponse>(
    `/admin/announcements/${id}`,
    {
      method: 'PATCH',
      body: JSON.stringify(partial),
    },
  );
  return response.announcement;
}

export async function publishAnnouncement(id: string): Promise<Announcement> {
  const response = await authenticatedFetch<AnnouncementResponse>(
    `/admin/announcements/${id}/publish`,
    { method: 'POST' },
  );
  return response.announcement;
}

export async function unpublishAnnouncement(id: string): Promise<Announcement> {
  const response = await authenticatedFetch<AnnouncementResponse>(
    `/admin/announcements/${id}/unpublish`,
    { method: 'POST' },
  );
  return response.announcement;
}

export async function deleteAnnouncement(
  id: string,
): Promise<{ success: boolean; message: string }> {
  return authenticatedFetch<{ success: boolean; message: string }>(
    `/admin/announcements/${id}`,
    { method: 'DELETE' },
  );
}
