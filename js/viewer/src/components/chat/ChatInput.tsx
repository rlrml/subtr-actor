import { useState, useRef, useEffect, useCallback } from 'react';
import { CHAT_OVERLAY_CONFIG } from './constants';

interface ChatInputProps {
  /** Whether the input is visible */
  isOpen: boolean;
  /** Callback when input should close */
  onClose: () => void;
  /** Callback to send a message */
  onSend: (text: string) => void;
  /** Maximum input length (default: 500) */
  maxLength?: number;
  /** Whether the connection is active */
  isConnected?: boolean;
}

/**
 * Chat input component for sending messages
 * Activated by pressing Enter, closed by Escape or sending
 */
export function ChatInput({
  isOpen,
  onClose,
  onSend,
  maxLength = CHAT_OVERLAY_CONFIG.INPUT_MAX_LENGTH,
  isConnected = true,
}: ChatInputProps) {
  const [value, setValue] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  // Auto-focus when opened
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  // Clear value when closed
  useEffect(() => {
    if (!isOpen) {
      setValue('');
    }
  }, [isOpen]);

  // Handle key events
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      // Stop propagation to prevent viewer shortcuts from triggering
      e.stopPropagation();

      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
        return;
      }

      if (e.key === 'Enter') {
        e.preventDefault();
        const trimmed = value.trim();

        // Don't send empty messages
        if (!trimmed) {
          return;
        }

        // Don't send if disconnected
        if (!isConnected) {
          return;
        }

        onSend(trimmed);
        setValue('');
        onClose();
      }
    },
    [value, onClose, onSend, isConnected]
  );

  // Handle input change
  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = e.target.value;
      if (newValue.length <= maxLength) {
        setValue(newValue);
      }
    },
    [maxLength]
  );

  // Handle click on container to prevent closing
  const handleContainerClick = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
  }, []);

  if (!isOpen) {
    return null;
  }

  return (
    <div
      className="pointer-events-auto max-w-sm"
      onClick={handleContainerClick}
    >
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        placeholder={isConnected ? 'Message...' : 'Reconnecting...'}
        disabled={!isConnected}
        className="w-64 bg-black/30 text-white text-sm rounded px-3 py-1.5
                   placeholder-white/40 focus:outline-none focus:bg-black/40
                   border-none disabled:opacity-50 disabled:cursor-not-allowed"
        style={{
          textShadow: '0 1px 2px rgba(0,0,0,0.8)',
        }}
        maxLength={maxLength}
      />
    </div>
  );
}
