/**
 * ClipModeSelector
 *
 * Toggle between Capture and Cinematic camera modes.
 *
 * Feature: 024-clip-system
 */

import { Video, Film } from 'lucide-react';
import { cn } from '@/lib/utils';

type CameraMode = 'capture' | 'cinematic';

interface ClipModeSelectorProps {
  mode: CameraMode;
  onChange: (mode: CameraMode) => void;
  disabled?: boolean;
}

export function ClipModeSelector({
  mode,
  onChange,
  disabled = false,
}: ClipModeSelectorProps) {
  return (
    <div className="flex gap-1">
      <button
        type="button"
        onClick={() => onChange('capture')}
        disabled={disabled}
        title="Record your camera movement"
        className={cn(
          "flex items-center gap-1.5 px-2.5 py-1.5 rounded text-xs font-medium transition-colors",
          mode === 'capture'
            ? "bg-blue-500 text-white"
            : "bg-zinc-700 text-zinc-300 hover:bg-zinc-600",
          disabled && "opacity-50 cursor-not-allowed"
        )}
      >
        <Video className="w-3.5 h-3.5" />
        <span>Capture</span>
      </button>
      <button
        type="button"
        onClick={() => onChange('cinematic')}
        disabled={disabled}
        title="Create keyframe-based camera path"
        className={cn(
          "flex items-center gap-1.5 px-2.5 py-1.5 rounded text-xs font-medium transition-colors",
          mode === 'cinematic'
            ? "bg-purple-500 text-white"
            : "bg-zinc-700 text-zinc-300 hover:bg-zinc-600",
          disabled && "opacity-50 cursor-not-allowed"
        )}
      >
        <Film className="w-3.5 h-3.5" />
        <span>Cinematic</span>
      </button>
    </div>
  );
}
