import { useState, useRef, useEffect } from 'react';
import { Smile } from 'lucide-react';
import { Button } from '@/components/ui/button';

// Preset emojis for quick selection
const EMOJI_CATEGORIES = {
  'Reactions': ['👍', '👎', '😂', '😮', '😍', '🔥', '💯', '👏'],
  'Rocket League': ['🚗', '⚽', '🥅', '🏆', '🎮', '💨', '💥', '✨'],
  'Expressions': ['😊', '😎', '🤔', '😅', '🙌', '👀', '❤️', '🎉'],
};

interface EmojiPickerProps {
  onEmojiSelect: (emoji: string) => void;
}

export function EmojiPicker({ onEmojiSelect }: EmojiPickerProps) {
  const [isOpen, setIsOpen] = useState(false);
  const pickerRef = useRef<HTMLDivElement>(null);

  // Close on click outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (pickerRef.current && !pickerRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [isOpen]);

  const handleEmojiClick = (emoji: string) => {
    onEmojiSelect(emoji);
    setIsOpen(false);
  };

  return (
    <div className="relative" ref={pickerRef}>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        onClick={() => setIsOpen(!isOpen)}
        className="text-gray-400 hover:text-gray-300"
      >
        <Smile className="h-5 w-5" />
      </Button>

      {isOpen && (
        <div className="absolute bottom-full left-0 mb-2 p-3 bg-gray-800 border border-gray-700 rounded-lg shadow-xl z-50 min-w-[280px]">
          {Object.entries(EMOJI_CATEGORIES).map(([category, emojis]) => (
            <div key={category} className="mb-3 last:mb-0">
              <div className="text-xs text-gray-500 mb-1.5 font-medium">
                {category}
              </div>
              <div className="flex flex-wrap gap-1">
                {emojis.map((emoji) => (
                  <button
                    key={emoji}
                    type="button"
                    onClick={() => handleEmojiClick(emoji)}
                    className="w-8 h-8 flex items-center justify-center text-lg hover:bg-gray-700 rounded transition-colors"
                  >
                    {emoji}
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
