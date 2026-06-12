import { MousePointer, MapPin, Pencil, Eraser, Trash2, Undo2 } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { ToolType } from '@/collab/types';

interface ToolBarProps {
  activeTool: ToolType;
  onToolChange: (tool: ToolType) => void;
  drawColor: string;
  onColorChange: (color: string) => void;
  drawThickness: number;
  onThicknessChange: (thickness: number) => void;
  canUndo: boolean;
  onUndo: () => void;
  isHost: boolean;
  onClearAll: () => void;
}

/**
 * Toolbar for collaborative drawing and ping tools
 */
export function ToolBar({
  activeTool,
  onToolChange,
  drawColor,
  onColorChange,
  drawThickness,
  onThicknessChange,
  canUndo,
  onUndo,
  isHost,
  onClearAll,
}: ToolBarProps) {
  const tools = [
    { id: 'select' as ToolType, icon: MousePointer, label: 'Select (Esc)', shortcut: 'Esc' },
    { id: 'ping' as ToolType, icon: MapPin, label: 'Ping (P)', shortcut: 'P' },
    { id: 'draw' as ToolType, icon: Pencil, label: 'Draw (B)', shortcut: 'B' },
    { id: 'eraser' as ToolType, icon: Eraser, label: 'Eraser (X)', shortcut: 'X' },
  ];

  return (
    <div className="flex items-center gap-2 p-2 bg-gray-900/90 rounded-lg backdrop-blur-sm border border-gray-700">
      {/* Tool buttons */}
      <div className="flex gap-1">
        {tools.map((tool) => (
          <button
            key={tool.id}
            onClick={() => onToolChange(tool.id)}
            className={cn(
              'p-2 rounded-md transition-colors',
              activeTool === tool.id
                ? 'bg-violet-600 text-white'
                : 'text-gray-400 hover:text-white hover:bg-gray-800'
            )}
            title={tool.label}
          >
            <tool.icon className="w-5 h-5" />
          </button>
        ))}
      </div>

      {/* Divider */}
      <div className="w-px h-6 bg-gray-700" />

      {/* Color picker (only visible when draw tool is active) */}
      {activeTool === 'draw' && (
        <>
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={drawColor}
              onChange={(e) => onColorChange(e.target.value)}
              className="w-8 h-8 rounded cursor-pointer border-0 p-0"
              title="Draw color"
            />
            <input
              type="range"
              min="1"
              max="10"
              value={drawThickness}
              onChange={(e) => onThicknessChange(Number(e.target.value))}
              className="w-20 accent-violet-500"
              title={`Brush size: ${drawThickness}`}
            />
          </div>
          <div className="w-px h-6 bg-gray-700" />
        </>
      )}

      {/* Undo button */}
      <button
        onClick={onUndo}
        disabled={!canUndo}
        className={cn(
          'p-2 rounded-md transition-colors',
          canUndo
            ? 'text-gray-400 hover:text-white hover:bg-gray-800'
            : 'text-gray-600 cursor-not-allowed'
        )}
        title="Undo (Ctrl+Z)"
      >
        <Undo2 className="w-5 h-5" />
      </button>

      {/* Clear all button (host only) */}
      {isHost && (
        <button
          onClick={onClearAll}
          className="p-2 rounded-md text-red-400 hover:text-red-300 hover:bg-gray-800 transition-colors"
          title="Clear all drawings"
        >
          <Trash2 className="w-5 h-5" />
        </button>
      )}
    </div>
  );
}

export default ToolBar;
