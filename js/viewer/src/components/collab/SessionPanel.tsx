import { useState } from 'react';
import { Users, Loader2 } from 'lucide-react';
import { useCollab } from '../../collab/useCollab';

interface SessionPanelProps {
  replayId: string;
}

export function SessionPanel({ replayId }: SessionPanelProps) {
  const {
    isConnected,
    isLoading,
    error,
    createSession,
    clearError,
  } = useCollab();

  const [nickname, setNickname] = useState('');

  const handleCreateSession = async () => {
    if (!nickname.trim()) return;
    await createSession(replayId, nickname.trim());
    // After creation, isInSession becomes true and CollabOverlay takes over
  };

  // Not connected to server
  if (!isConnected) {
    return (
      <div className="bg-gray-800/90 backdrop-blur rounded-lg p-4 border border-gray-700">
        <div className="flex items-center gap-2 text-yellow-400">
          <Loader2 size={16} className="animate-spin" />
          <span className="text-sm">Connecting to collab server...</span>
        </div>
      </div>
    );
  }

  // Not in session - show create form
  return (
    <div className="bg-gray-800/90 backdrop-blur rounded-lg p-4 border border-gray-700">
      <div className="flex items-center gap-2 mb-3">
        <Users size={18} className="text-violet-400" />
        <span className="text-white font-medium">Watch Together</span>
      </div>

      {error && (
        <div className="mb-3 p-2 bg-red-500/20 border border-red-500/50 rounded text-red-400 text-sm">
          {error}
          <button
            onClick={clearError}
            className="ml-2 text-red-300 hover:text-red-200"
          >
            ×
          </button>
        </div>
      )}

      <div className="space-y-3">
        <div>
          <label className="block text-sm text-gray-400 mb-1">Your Nickname</label>
          <input
            type="text"
            value={nickname}
            onChange={(e) => setNickname(e.target.value)}
            placeholder="Enter a nickname..."
            maxLength={32}
            className="w-full bg-gray-900 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm placeholder-gray-500 focus:border-violet-500 focus:outline-none"
          />
        </div>

        <button
          onClick={handleCreateSession}
          disabled={isLoading || !nickname.trim()}
          className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-violet-600 hover:bg-violet-500 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg text-sm font-medium transition-colors"
        >
          {isLoading ? (
            <>
              <Loader2 size={16} className="animate-spin" />
              Creating...
            </>
          ) : (
            <>
              <Users size={16} />
              Create Session
            </>
          )}
        </button>
      </div>

      <p className="mt-3 text-xs text-gray-500">
        Create a session and share the link with friends to watch this replay together.
      </p>
    </div>
  );
}
