/**
 * useClipDraft - Hook for managing clip draft persistence in localStorage
 * (026-clip-editor-redesign)
 *
 * Provides automatic local storage of clip editor state to prevent
 * accidental loss of work when the editor is closed.
 */

import { useCallback, useEffect, useRef } from 'react';
import type { CameraKeyframe, CameraData } from '@/api/clips';

// Storage key prefix
const STORAGE_PREFIX = 'rlview:clip-draft:';

// Maximum age for drafts (30 days in ms)
const MAX_DRAFT_AGE_MS = 30 * 24 * 60 * 60 * 1000;

// Current schema version
const DRAFT_VERSION = 1;

/**
 * ClipDraft interface - represents a saved draft in localStorage
 */
export interface ClipDraft {
  // Metadata
  version: number; // Schema version for future migrations
  replayId: string; // UUID of the source replay
  updatedAt: number; // Unix timestamp (ms) of last modification

  // Segment definition
  segment: {
    start: number; // Start time in seconds
    end: number; // End time in seconds
  };

  // Editor state
  mode: 'capture' | 'cinematic';

  // Mode-specific data (mutually exclusive)
  keyframes?: CameraKeyframe[]; // For cinematic mode
  captureData?: CameraData; // For capture mode (if recording completed)

  // UI state (optional, for better UX on restore)
  selectedKeyframeId?: string;
  autoSpaceEnabled?: boolean;
  autoSpaceInterval?: number; // In seconds
}

/**
 * Result interface for useClipDraft hook
 */
export interface UseClipDraftResult {
  // Query
  hasDraft: () => boolean;
  getDraft: () => ClipDraft | null;

  // Mutations
  saveDraft: (draft: Omit<ClipDraft, 'version' | 'updatedAt'>) => boolean;
  deleteDraft: () => void;

  // Cleanup
  cleanupOldDrafts: () => number; // Returns number of deleted drafts
}

/**
 * Migrate draft from older versions to current version
 * @param draft - Draft object to migrate
 * @returns Migrated draft or null if migration fails
 */
function migrateDraft(draft: unknown): ClipDraft | null {
  if (!draft || typeof draft !== 'object') return null;

  const obj = draft as Record<string, unknown>;

  // Check version
  if (typeof obj.version !== 'number') return null;

  // Version 1 is current, no migration needed
  if (obj.version === 1) {
    // Validate required fields
    if (
      typeof obj.replayId !== 'string' ||
      typeof obj.updatedAt !== 'number' ||
      !obj.segment ||
      typeof (obj.segment as Record<string, unknown>).start !== 'number' ||
      typeof (obj.segment as Record<string, unknown>).end !== 'number' ||
      (obj.mode !== 'capture' && obj.mode !== 'cinematic')
    ) {
      return null;
    }

    return obj as unknown as ClipDraft;
  }

  // Future: add migration logic for version 2, 3, etc.

  // Unknown version: cannot migrate
  console.warn('[useClipDraft] Unknown draft version:', obj.version);
  return null;
}

/**
 * Check if a draft is expired (older than MAX_DRAFT_AGE_MS)
 */
function isDraftExpired(draft: ClipDraft): boolean {
  return Date.now() - draft.updatedAt > MAX_DRAFT_AGE_MS;
}

/**
 * Get storage key for a replay ID
 */
function getStorageKey(replayId: string): string {
  return `${STORAGE_PREFIX}${replayId}`;
}

/**
 * Hook for managing clip draft persistence
 *
 * @param replayId - The replay ID to manage drafts for
 * @returns Draft management functions
 */
export function useClipDraft(replayId: string): UseClipDraftResult {
  const storageKey = getStorageKey(replayId);

  // Ref to track if cleanup has been run this session
  const cleanupRunRef = useRef(false);

  /**
   * Check if a draft exists for this replay
   */
  const hasDraft = useCallback((): boolean => {
    try {
      const data = localStorage.getItem(storageKey);
      if (!data) return false;

      const draft = migrateDraft(JSON.parse(data));
      if (!draft) return false;

      // Check if expired
      if (isDraftExpired(draft)) {
        localStorage.removeItem(storageKey);
        return false;
      }

      return true;
    } catch (error) {
      console.warn('[useClipDraft] Error checking draft:', error);
      return false;
    }
  }, [storageKey]);

  /**
   * Get the draft for this replay
   */
  const getDraft = useCallback((): ClipDraft | null => {
    try {
      const data = localStorage.getItem(storageKey);
      if (!data) return null;

      const draft = migrateDraft(JSON.parse(data));
      if (!draft) {
        // Invalid draft, remove it
        localStorage.removeItem(storageKey);
        return null;
      }

      // Check if expired
      if (isDraftExpired(draft)) {
        localStorage.removeItem(storageKey);
        return null;
      }

      return draft;
    } catch (error) {
      console.warn('[useClipDraft] Error getting draft:', error);
      // Remove corrupted data
      try {
        localStorage.removeItem(storageKey);
      } catch {}
      return null;
    }
  }, [storageKey]);

  /**
   * Save a draft for this replay
   * Returns true on success, false on failure
   */
  const saveDraft = useCallback(
    (draftData: Omit<ClipDraft, 'version' | 'updatedAt'>): boolean => {
      try {
        const draft: ClipDraft = {
          ...draftData,
          version: DRAFT_VERSION,
          updatedAt: Date.now(),
        };

        const serialized = JSON.stringify(draft);

        // Check approximate size (rough estimate)
        const sizeKB = serialized.length / 1024;
        if (sizeKB > 100) {
          console.warn('[useClipDraft] Draft size exceeds 100KB:', sizeKB.toFixed(2), 'KB');
        }

        localStorage.setItem(storageKey, serialized);
        return true;
      } catch (error) {
        // Handle quota exceeded
        if (
          error instanceof DOMException &&
          (error.code === 22 ||
            error.code === 1014 ||
            error.name === 'QuotaExceededError' ||
            error.name === 'NS_ERROR_DOM_QUOTA_REACHED')
        ) {
          console.error('[useClipDraft] localStorage quota exceeded');
          // Try to cleanup old drafts and retry
          cleanupOldDrafts();
          try {
            const draft: ClipDraft = {
              ...draftData,
              version: DRAFT_VERSION,
              updatedAt: Date.now(),
            };
            localStorage.setItem(storageKey, JSON.stringify(draft));
            return true;
          } catch {
            return false;
          }
        }

        console.error('[useClipDraft] Error saving draft:', error);
        return false;
      }
    },
    [storageKey]
  );

  /**
   * Delete the draft for this replay
   */
  const deleteDraft = useCallback(() => {
    try {
      localStorage.removeItem(storageKey);
    } catch (error) {
      console.warn('[useClipDraft] Error deleting draft:', error);
    }
  }, [storageKey]);

  /**
   * Cleanup old drafts across all replays
   * Returns the number of deleted drafts
   */
  const cleanupOldDrafts = useCallback((): number => {
    let deletedCount = 0;

    try {
      const keysToDelete: string[] = [];

      // Find all draft keys
      for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i);
        if (key && key.startsWith(STORAGE_PREFIX)) {
          try {
            const data = localStorage.getItem(key);
            if (data) {
              const draft = JSON.parse(data) as ClipDraft;
              if (isDraftExpired(draft)) {
                keysToDelete.push(key);
              }
            }
          } catch {
            // Invalid data, mark for deletion
            keysToDelete.push(key);
          }
        }
      }

      // Delete expired drafts
      for (const key of keysToDelete) {
        localStorage.removeItem(key);
        deletedCount++;
      }

      if (deletedCount > 0) {
        console.log('[useClipDraft] Cleaned up', deletedCount, 'old drafts');
      }
    } catch (error) {
      console.warn('[useClipDraft] Error during cleanup:', error);
    }

    return deletedCount;
  }, []);

  // Run cleanup once per session when hook is first used
  useEffect(() => {
    if (!cleanupRunRef.current) {
      cleanupRunRef.current = true;
      cleanupOldDrafts();
    }
  }, [cleanupOldDrafts]);

  return {
    hasDraft,
    getDraft,
    saveDraft,
    deleteDraft,
    cleanupOldDrafts,
  };
}

/**
 * Utility function to get all draft replay IDs
 * Useful for debugging or showing a list of drafts
 */
export function getAllDraftReplayIds(): string[] {
  const replayIds: string[] = [];

  try {
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key && key.startsWith(STORAGE_PREFIX)) {
        const replayId = key.slice(STORAGE_PREFIX.length);
        replayIds.push(replayId);
      }
    }
  } catch (error) {
    console.warn('[useClipDraft] Error getting all draft IDs:', error);
  }

  return replayIds;
}
