import { memo, useState, useEffect, useRef } from 'react';
import type { ChatMessage } from '@/collab/types';
import { CHAT_OVERLAY_CONFIG, truncateText } from './constants';

/**
 * Internal message state with animation metadata
 */
interface OverlayMessage {
  message: ChatMessage;
  opacity: number;
  expiresAt: number;
}

interface ChatOverlayMessagesProps {
  /** Array of chat messages from CollabProvider */
  messages: ChatMessage[];
  /** Maximum visible messages (default: 5) */
  maxVisible?: number;
  /** Display duration before fade starts in ms (default: 10000) */
  displayDuration?: number;
}

/**
 * Chat overlay component displaying messages in Rocket League style
 * Messages appear on the left side, stack vertically, and auto-fade after timeout
 */
export const ChatOverlayMessages = memo(function ChatOverlayMessages({
  messages,
  maxVisible = CHAT_OVERLAY_CONFIG.MAX_VISIBLE_MESSAGES,
  displayDuration = CHAT_OVERLAY_CONFIG.MESSAGE_DISPLAY_DURATION,
}: ChatOverlayMessagesProps) {
  const [overlayMessages, setOverlayMessages] = useState<OverlayMessage[]>([]);
  const fadeTimerRef = useRef<number | null>(null);
  const prevMessagesLengthRef = useRef(0);

  // Sync messages from props to overlay state
  useEffect(() => {
    const visibleMessages = messages.slice(-maxVisible);
    const now = Date.now();
    const newExpiresAt = now + displayDuration;

    // Check if new messages arrived
    const hasNewMessages = messages.length > prevMessagesLengthRef.current;
    prevMessagesLengthRef.current = messages.length;

    setOverlayMessages((prev) => {
      return visibleMessages.map((msg) => {
        const existing = prev.find((om) => om.message.id === msg.id);
        if (existing) {
          // Keep existing message, but reset expiry if new messages arrived
          return {
            ...existing,
            expiresAt: hasNewMessages ? newExpiresAt : existing.expiresAt,
          };
        }
        // New message
        return {
          message: msg,
          opacity: 1,
          expiresAt: newExpiresAt,
        };
      });
    });
  }, [messages, maxVisible, displayDuration]);

  // Fade timer - runs independently
  useEffect(() => {
    if (overlayMessages.length === 0) {
      return;
    }

    // Clear any existing timer
    if (fadeTimerRef.current) {
      clearInterval(fadeTimerRef.current);
    }

    fadeTimerRef.current = window.setInterval(() => {
      const now = Date.now();

      setOverlayMessages((prev) => {
        const updated = prev.map((om) => {
          const timeLeft = om.expiresAt - now;

          if (timeLeft <= 0) {
            return { ...om, opacity: 0 };
          }

          if (timeLeft <= CHAT_OVERLAY_CONFIG.FADE_OUT_DURATION) {
            return { ...om, opacity: timeLeft / CHAT_OVERLAY_CONFIG.FADE_OUT_DURATION };
          }

          return om;
        });

        // Remove fully faded messages
        const visible = updated.filter((om) => om.opacity > 0);

        // Stop timer if no messages left
        if (visible.length === 0 && fadeTimerRef.current) {
          clearInterval(fadeTimerRef.current);
          fadeTimerRef.current = null;
        }

        return visible;
      });
    }, 50);

    return () => {
      if (fadeTimerRef.current) {
        clearInterval(fadeTimerRef.current);
        fadeTimerRef.current = null;
      }
    };
  }, [overlayMessages.length > 0]); // Only restart when going from 0 to >0 messages

  // Don't render if no messages
  if (overlayMessages.length === 0) {
    return null;
  }

  return (
    <div className="pointer-events-none flex flex-col gap-0.5 max-w-sm">
      {overlayMessages.map((om) => (
        <div
          key={om.message.id}
          className="px-2 py-0.5"
          style={{
            opacity: om.opacity,
          }}
        >
          <span
            className="text-sm"
            style={{
              textShadow: '0 1px 3px rgba(0,0,0,0.9), 0 0 8px rgba(0,0,0,0.5)',
            }}
          >
            <span
              className="font-semibold"
              style={{ color: om.message.authorColor }}
            >
              {truncateText(om.message.authorNickname, CHAT_OVERLAY_CONFIG.MAX_NICKNAME_LENGTH)}
            </span>
            <span className="text-white/60 mx-1">:</span>
            <span className="text-white/90">
              {truncateText(om.message.text, CHAT_OVERLAY_CONFIG.MAX_MESSAGE_LENGTH)}
            </span>
          </span>
        </div>
      ))}
    </div>
  );
});
