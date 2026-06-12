/**
 * Chat overlay configuration constants
 * Based on Rocket League in-game chat behavior
 */
export const CHAT_OVERLAY_CONFIG = {
  /** Maximum number of messages displayed simultaneously */
  MAX_VISIBLE_MESSAGES: 5,

  /** Duration before messages start fading (in ms) */
  MESSAGE_DISPLAY_DURATION: 10000,

  /** Duration of fade-out animation (in ms) */
  FADE_OUT_DURATION: 500,

  /** Duration of fade-in animation (in ms) */
  FADE_IN_DURATION: 200,

  /** Maximum characters before message truncation */
  MAX_MESSAGE_LENGTH: 150,

  /** Maximum characters before nickname truncation */
  MAX_NICKNAME_LENGTH: 15,

  /** Maximum input length (backend limit) */
  INPUT_MAX_LENGTH: 500,
} as const;

/**
 * Truncate text with ellipsis if exceeding max length
 */
export function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength - 3) + '...';
}
