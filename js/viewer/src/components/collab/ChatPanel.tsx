import { useState, useRef, useEffect } from 'react';
import { MessageCircle, Send, ChevronDown, ChevronUp } from 'lucide-react';
import type { ChatMessage } from '@/collab/types';

interface ChatPanelProps {
  messages: ChatMessage[];
  onSendMessage: (text: string) => void;
  disabled?: boolean;
}

/**
 * Chat panel component for collaborative sessions
 */
export function ChatPanel({ messages, onSendMessage, disabled }: ChatPanelProps) {
  const [isExpanded, setIsExpanded] = useState(true);
  const [inputValue, setInputValue] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (isExpanded) {
      messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages, isExpanded]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = inputValue.trim();
    if (!trimmed || disabled) return;

    onSendMessage(trimmed);
    setInputValue('');
  };

  const formatTime = (timestamp: number): string => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  return (
    <div className="bg-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-800 overflow-hidden flex flex-col">
      {/* Header */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full px-4 py-3 flex items-center justify-between hover:bg-gray-800/50 transition-colors"
      >
        <div className="flex items-center gap-2">
          <MessageCircle className="w-4 h-4 text-violet-400" />
          <span className="font-medium text-white">Chat</span>
          {!isExpanded && messages.length > 0 && (
            <span className="text-xs text-gray-500">
              ({messages.length})
            </span>
          )}
        </div>
        {isExpanded ? (
          <ChevronUp className="w-4 h-4 text-gray-400" />
        ) : (
          <ChevronDown className="w-4 h-4 text-gray-400" />
        )}
      </button>

      {/* Chat content */}
      {isExpanded && (
        <div className="border-t border-gray-800 flex flex-col">
          {/* Messages */}
          <div className="h-48 overflow-y-auto p-2 space-y-2">
            {messages.length === 0 ? (
              <div className="flex items-center justify-center h-full">
                <p className="text-sm text-gray-500">No messages yet</p>
              </div>
            ) : (
              messages.map((msg) => (
                <div key={msg.id} className="px-2">
                  <div className="flex items-baseline gap-2">
                    <span
                      className="font-medium text-sm"
                      style={{ color: msg.authorColor }}
                    >
                      {msg.authorNickname}
                    </span>
                    <span className="text-xs text-gray-600">
                      {formatTime(msg.timestamp)}
                    </span>
                  </div>
                  <p className="text-sm text-gray-300 break-words">
                    {msg.text}
                  </p>
                </div>
              ))
            )}
            <div ref={messagesEndRef} />
          </div>

          {/* Input */}
          <form
            onSubmit={handleSubmit}
            className="p-2 border-t border-gray-800"
          >
            <div className="flex gap-2">
              <input
                ref={inputRef}
                type="text"
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
                placeholder={disabled ? 'Connecting...' : 'Type a message...'}
                maxLength={500}
                disabled={disabled}
                className="flex-1 px-3 py-2 text-sm rounded-lg bg-gray-800 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent disabled:opacity-50"
              />
              <button
                type="submit"
                disabled={disabled || !inputValue.trim()}
                className="p-2 rounded-lg bg-violet-600 hover:bg-violet-500 disabled:bg-gray-700 disabled:cursor-not-allowed text-white transition-colors"
              >
                <Send className="w-4 h-4" />
              </button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
}
