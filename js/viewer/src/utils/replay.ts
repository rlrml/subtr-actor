/**
 * Get the display title for a replay
 * Returns the custom title if set, otherwise falls back to the original filename
 */
export function getDisplayTitle(
  title: string | null | undefined,
  originalFilename: string | undefined
): string {
  if (title && title.trim()) {
    return title;
  }
  if (originalFilename) {
    // Remove .replay extension for display
    return originalFilename.replace(/\.replay$/i, '');
  }
  return 'Untitled Replay';
}
