import { useState, useEffect, useRef, useCallback } from 'react';
import { Video, User, Circle, Keyboard, Sun, X, Lock, Loader2, Check, ChevronDown, Activity, Scissors } from 'lucide-react';
import { environmentApi } from '../services/environment.api';
import type { EnvironmentListItem } from '../types/environment';

type CameraMode = 'free' | 'follow' | 'ball' | 'ballOrbit' | 'player';

interface QuickToolbarProps {
  // Camera
  cameraMode: CameraMode;
  onCameraModeChange: (mode: CameraMode) => void;
  // Environment
  currentEnvironmentId: string | null;
  onEnvironmentChange: (environmentId: string) => void;
  isLoadingEnvironment?: boolean;
  hideEnvironment?: boolean; // Hide environment selector (e.g., in fullscreen mode)
  // Collab
  isInSession?: boolean;
  isHost?: boolean;
  // Quality indicator (016-replay-quality-indicator)
  qualityScore?: number | null;
  // Clip system (024-clip-system)
  onCreateClip?: () => void;
  isClipEditorOpen?: boolean;
}

const SHORTCUTS_STORAGE_KEY = 'keyboard-shortcuts-visible';

interface ShortcutGroup {
  title: string;
  shortcuts: { key: string; description: string }[];
}

/**
 * QuickToolbar - Unified toolbar for quick controls in the top-right of the viewer
 * Contains: Camera mode selector, Environment selector, Keyboard shortcuts
 */
export function QuickToolbar({
  cameraMode,
  onCameraModeChange,
  currentEnvironmentId,
  onEnvironmentChange,
  isLoadingEnvironment = false,
  isInSession = false,
  isHost = false,
  qualityScore,
  onCreateClip,
  isClipEditorOpen = false,
  hideEnvironment = false,
}: QuickToolbarProps) {
  // Environment dropdown state
  const [envDropdownOpen, setEnvDropdownOpen] = useState(false);
  const [environments, setEnvironments] = useState<EnvironmentListItem[]>([]);
  const [isLoadingEnvList, setIsLoadingEnvList] = useState(false);
  const envDropdownRef = useRef<HTMLDivElement>(null);

  // Shortcuts dropdown state
  const [shortcutsOpen, setShortcutsOpen] = useState(() => {
    const stored = localStorage.getItem(SHORTCUTS_STORAGE_KEY);
    return stored === null ? false : stored === 'true';
  });
  const shortcutsDropdownRef = useRef<HTMLDivElement>(null);

  // Can change environment in collab mode?
  const canChangeEnv = !isInSession || isHost;

  // Camera modes config
  const cameraModes = [
    { id: 'free' as const, icon: Video, label: 'Free', tooltip: 'Free Camera (WASD + Mouse)' },
    { id: 'player' as const, icon: User, label: 'Player', tooltip: 'Player Camera (Click a player)' },
    { id: 'ballOrbit' as const, icon: Circle, label: 'Ball', tooltip: 'Ball Orbit Camera (Scroll to zoom)' },
  ];

  // Load environments
  const loadEnvironments = useCallback(async () => {
    setIsLoadingEnvList(true);
    try {
      const response = await environmentApi.list();
      setEnvironments(response.environments || []);
    } catch (err) {
      console.error('Failed to load environments:', err);
    } finally {
      setIsLoadingEnvList(false);
    }
  }, []);

  useEffect(() => {
    loadEnvironments();
  }, [loadEnvironments]);

  // Save shortcuts visibility preference
  useEffect(() => {
    localStorage.setItem(SHORTCUTS_STORAGE_KEY, String(shortcutsOpen));
  }, [shortcutsOpen]);

  // Close dropdowns when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (envDropdownRef.current && !envDropdownRef.current.contains(e.target as Node)) {
        setEnvDropdownOpen(false);
      }
      if (shortcutsDropdownRef.current && !shortcutsDropdownRef.current.contains(e.target as Node)) {
        setShortcutsOpen(false);
      }
    };

    if (envDropdownOpen || shortcutsOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [envDropdownOpen, shortcutsOpen]);

  // Get shortcuts based on current mode
  const getShortcuts = (): ShortcutGroup[] => {
    const groups: ShortcutGroup[] = [];

    if (cameraMode === 'free') {
      groups.push({
        title: 'Free Camera',
        shortcuts: [
          { key: 'WASD', description: 'Move camera' },
          { key: 'Space', description: 'Move up' },
          { key: 'Shift', description: 'Move down' },
          { key: 'Right-click + drag', description: 'Look around' },
        ],
      });
    } else if (cameraMode === 'ballOrbit') {
      groups.push({
        title: 'Ball Camera',
        shortcuts: [
          { key: 'Click + drag', description: 'Orbit around ball' },
          { key: 'Scroll', description: 'Zoom in/out' },
        ],
      });
    } else if (cameraMode === 'player') {
      groups.push({
        title: 'Player Camera',
        shortcuts: [{ key: 'Click player', description: 'Switch player' }],
      });
    }

    if (isInSession) {
      groups.push({
        title: 'Collaboration',
        shortcuts: [
          { key: 'T', description: 'Open chat' },
          { key: 'P', description: 'Ping mode' },
          { key: 'B', description: 'Draw mode (Brush)' },
          { key: 'X', description: 'Eraser mode' },
          { key: 'Z', description: 'Undo stroke' },
          { key: 'Esc', description: 'Select mode' },
        ],
      });
    }

    groups.push({
      title: 'Debug',
      shortcuts: [{ key: 'F3', description: 'Debug panel' }],
    });

    return groups;
  };

  const currentEnvironment = environments.find(env => env.id === currentEnvironmentId);
  const shortcuts = getShortcuts();

  // Handle environment selection
  const handleEnvSelect = (envId: string) => {
    if (envId !== currentEnvironmentId && canChangeEnv) {
      onEnvironmentChange(envId);
    }
    setEnvDropdownOpen(false);
  };

  return (
    <div className="flex items-center bg-black/70 backdrop-blur-sm rounded-xl border border-white/10 shadow-lg">
      {/* Environment Section (hidden in fullscreen mode) */}
      {!hideEnvironment && (
      <div ref={envDropdownRef} className="relative px-1 py-1">
        <button
          onClick={() => canChangeEnv && !isLoadingEnvironment && setEnvDropdownOpen(!envDropdownOpen)}
          disabled={!canChangeEnv || isLoadingEnvironment}
          className={`
            flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-medium transition-all duration-150
            ${isLoadingEnvironment
              ? 'text-amber-400'
              : !canChangeEnv
              ? 'text-gray-500 cursor-not-allowed'
              : envDropdownOpen
              ? 'bg-amber-500/20 text-amber-300 ring-1 ring-amber-500/50'
              : 'text-gray-300 hover:bg-white/10 hover:text-white'
            }
          `}
          title={
            isLoadingEnvironment
              ? 'Loading environment...'
              : !canChangeEnv
              ? 'Only the host can change the environment'
              : currentEnvironment?.name || 'Select environment'
          }
        >
          {isLoadingEnvironment ? (
            <Loader2 size={14} className="animate-spin" />
          ) : !canChangeEnv ? (
            <Lock size={12} />
          ) : (
            <Sun size={14} className="text-amber-400" />
          )}
          <span className="max-w-[70px] truncate hidden sm:inline">
            {isLoadingEnvList ? '...' : currentEnvironment?.name || 'Env'}
          </span>
          {canChangeEnv && !isLoadingEnvironment && (
            <ChevronDown size={12} className={`transition-transform ${envDropdownOpen ? 'rotate-180' : ''}`} />
          )}
        </button>

        {/* Environment Dropdown */}
        {envDropdownOpen && environments.length > 0 && (
          <div className="absolute right-0 top-full mt-2 w-56 bg-gray-900/95 backdrop-blur-sm rounded-lg border border-gray-700 shadow-xl z-50 max-h-[300px] overflow-y-auto">
            <div className="px-3 py-2 border-b border-gray-700">
              <div className="flex items-center gap-2 text-gray-300">
                <Sun size={14} className="text-amber-400" />
                <span className="text-sm font-medium">Environment</span>
              </div>
            </div>
            <div className="py-1">
              {environments.map((env) => (
                <button
                  key={env.id}
                  onClick={() => handleEnvSelect(env.id)}
                  className={`w-full flex items-center gap-2 px-3 py-2 text-left transition-colors hover:bg-gray-800 ${
                    env.id === currentEnvironmentId ? 'bg-amber-900/30' : ''
                  }`}
                >
                  <Sun size={12} className="text-amber-400 flex-shrink-0" />
                  <div className="flex-1 min-w-0">
                    <div className="text-xs text-white truncate">
                      {env.name}
                      {env.isDefault && (
                        <span className="ml-1 text-[9px] text-yellow-400">(default)</span>
                      )}
                    </div>
                    <div className="text-[10px] text-gray-500">
                      {env.meshCount} mesh{env.meshCount !== 1 ? 'es' : ''} · {env.lightCount} light{env.lightCount !== 1 ? 's' : ''}
                    </div>
                  </div>
                  {env.id === currentEnvironmentId && (
                    <Check size={12} className="text-amber-400 flex-shrink-0" />
                  )}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
      )}

      {/* Separator (only show if environment section is visible) */}
      {!hideEnvironment && <div className="w-px h-8 bg-white/20" />}

      {/* Camera Mode Section */}
      <div className="flex items-center gap-0.5 px-1 py-1">
        {cameraModes.map((mode) => {
          const Icon = mode.icon;
          const isActive = cameraMode === mode.id;

          return (
            <button
              key={mode.id}
              onClick={() => onCameraModeChange(mode.id)}
              title={mode.tooltip}
              className={`
                flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-medium transition-all duration-150
                ${isActive
                  ? 'bg-violet-500/20 text-violet-300 ring-1 ring-violet-500/50'
                  : 'text-gray-300 hover:bg-white/10 hover:text-white'
                }
              `}
            >
              <Icon size={14} className={isActive ? 'text-violet-400' : ''} />
              <span className="hidden sm:inline">{mode.label}</span>
            </button>
          );
        })}
      </div>

      {/* Separator */}
      <div className="w-px h-8 bg-white/20" />

      {/* Create Clip Button (024-clip-system) */}
      {onCreateClip && (
        <>
          <div className="px-1 py-1">
            <button
              onClick={onCreateClip}
              disabled={isClipEditorOpen}
              title={isClipEditorOpen ? 'Clip editor is open' : 'Create a clip from this replay'}
              className={`
                flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-medium transition-all duration-150
                ${isClipEditorOpen
                  ? 'bg-red-500/20 text-red-300 ring-1 ring-red-500/50'
                  : 'text-gray-300 hover:bg-white/10 hover:text-white'
                }
              `}
            >
              <Scissors size={14} className={isClipEditorOpen ? 'text-red-400' : ''} />
              <span className="hidden sm:inline">Clip</span>
            </button>
          </div>

          {/* Separator */}
          <div className="w-px h-8 bg-white/20" />
        </>
      )}

      {/* Keyboard Shortcuts Section */}
      <div ref={shortcutsDropdownRef} className="relative px-1 py-1">
        <button
          onClick={() => setShortcutsOpen(!shortcutsOpen)}
          title={shortcutsOpen ? 'Hide keyboard shortcuts' : 'Show keyboard shortcuts'}
          className={`
            flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-medium transition-all duration-150
            ${shortcutsOpen
              ? 'bg-gray-500/20 text-gray-200 ring-1 ring-gray-500/50'
              : 'text-gray-400 hover:bg-white/10 hover:text-white'
            }
          `}
        >
          <Keyboard size={14} />
        </button>

        {/* Shortcuts Dropdown */}
        {shortcutsOpen && (
          <div className="absolute right-0 top-full mt-2 w-64 bg-gray-900/95 backdrop-blur-sm rounded-lg border border-gray-700 shadow-xl z-50">
            {/* Header */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-gray-700">
              <div className="flex items-center gap-2 text-gray-300">
                <Keyboard size={14} />
                <span className="text-sm font-medium">Keyboard Shortcuts</span>
              </div>
              <button
                onClick={() => setShortcutsOpen(false)}
                className="p-1 text-gray-500 hover:text-white transition-colors"
                title="Hide shortcuts"
              >
                <X size={14} />
              </button>
            </div>

            {/* Shortcuts list */}
            <div className="p-2 max-h-[60vh] overflow-y-auto">
              {shortcuts.map((group, groupIndex) => (
                <div key={group.title} className={groupIndex > 0 ? 'mt-3' : ''}>
                  <div className="text-xs font-semibold text-gray-500 uppercase tracking-wider px-1 mb-1">
                    {group.title}
                  </div>
                  <div className="space-y-0.5">
                    {group.shortcuts.map((shortcut) => (
                      <div
                        key={shortcut.key}
                        className="flex items-center justify-between px-1 py-0.5 text-xs"
                      >
                        <span className="text-gray-400">{shortcut.description}</span>
                        <kbd className="px-1.5 py-0.5 bg-gray-800 border border-gray-600 rounded text-gray-300 font-mono text-[10px]">
                          {shortcut.key}
                        </kbd>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Quality Indicator Section */}
      {qualityScore !== null && qualityScore !== undefined && (
        <>
          {/* Separator */}
          <div className="w-px h-8 bg-white/20" />

          {/* Quality Score with custom tooltip */}
          <div className="relative px-1 py-1 group">
            <div
              className={`
                flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-medium cursor-default
                ${qualityScore >= 70
                  ? 'text-green-400'
                  : qualityScore >= 50
                    ? 'text-amber-400'
                    : 'text-red-400'
                }
              `}
            >
              <Activity size={14} />
              <span>{qualityScore}%</span>
            </div>

            {/* Custom Tooltip */}
            <div className="absolute right-0 top-full mt-2 w-48 bg-gray-900/95 backdrop-blur-sm rounded-lg border border-gray-700 shadow-xl z-50 p-3 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-150">
              <div className="flex items-center gap-2 mb-2">
                <Activity size={14} className={
                  qualityScore >= 70 ? 'text-green-400' :
                  qualityScore >= 50 ? 'text-amber-400' : 'text-red-400'
                } />
                <span className="text-sm font-medium text-white">Data Quality</span>
              </div>
              <div className={`text-2xl font-bold mb-1 ${
                qualityScore >= 70 ? 'text-green-400' :
                qualityScore >= 50 ? 'text-amber-400' : 'text-red-400'
              }`}>
                {qualityScore}%
              </div>
              <p className="text-xs text-gray-400">
                {qualityScore >= 70
                  ? 'Good quality - smooth playback expected'
                  : qualityScore >= 50
                    ? 'Acceptable quality - minor visual issues possible'
                    : 'Low quality - visual glitches may occur'}
              </p>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

export default QuickToolbar;
