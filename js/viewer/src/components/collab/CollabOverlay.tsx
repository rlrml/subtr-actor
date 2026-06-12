import { useCollab } from '@/collab/useCollab';
import { ChatPanel } from './ChatPanel';
import { HostControls } from './HostControls';
import { useState } from 'react';
import { Copy, Check, LogOut, Users, Link } from 'lucide-react';

interface CollabOverlayProps {
  inDrawer?: boolean;
}

/**
 * Overlay containing all collaborative viewing UI elements
 * Note: Participant list is now handled in GameOverlay for better UX
 */
export function CollabOverlay({
  inDrawer = false,
}: CollabOverlayProps) {
  const {
    isInSession,
    isConnected,
    sessionId,
    participants,
    chatMessages,
    isHost,
    hostParticipant,
    sendChat,
    leaveSession,
    getShareUrl,
  } = useCollab();

  const [copied, setCopied] = useState(false);

  // Don't render if not in a session
  if (!isInSession || !sessionId) {
    return null;
  }

  const shareUrl = getShareUrl(sessionId);

  const handleCopyUrl = async () => {
    try {
      await navigator.clipboard.writeText(shareUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const handleSendChat = async (text: string) => {
    await sendChat(text);
  };

  // Drawer mode - render content without absolute positioning
  if (inDrawer) {
    return (
      <div className="flex flex-col gap-3 h-full">
        {/* Session info bar */}
        <div className="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg">
          <div className="flex items-center gap-2">
            <Users className="w-4 h-4 text-violet-400" />
            <span className="text-sm text-white">
              {Object.keys(participants).length} watching
            </span>
          </div>
          <button
            onClick={leaveSession}
            className="p-2 rounded-lg bg-red-500/20 hover:bg-red-600 text-red-400 hover:text-white transition-colors"
            title="Leave session"
          >
            <LogOut className="w-4 h-4" />
          </button>
        </div>

        {/* Share URL */}
        <div className="p-3 bg-gray-800/50 rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <Link className="w-4 h-4 text-violet-400" />
            <span className="text-sm text-gray-300">Share this link</span>
          </div>
          <div className="flex gap-2">
            <input
              type="text"
              readOnly
              value={shareUrl}
              className="flex-1 px-3 py-2 bg-gray-900 border border-gray-700 rounded text-sm text-gray-300 truncate"
            />
            <button
              onClick={handleCopyUrl}
              className={`px-3 py-2 rounded transition-colors flex items-center gap-1 ${
                copied
                  ? 'bg-green-600 text-white'
                  : 'bg-violet-600 hover:bg-violet-500 text-white'
              }`}
              title="Copy URL"
            >
              {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
            </button>
          </div>
        </div>

        {/* Host controls */}
        <HostControls
          isHost={isHost}
          hostNickname={hostParticipant?.nickname}
        />

        {/* Chat panel - takes remaining space */}
        <div className="flex-1 min-h-0">
          <ChatPanel
            messages={chatMessages}
            onSendMessage={handleSendChat}
            disabled={!isConnected}
          />
        </div>
      </div>
    );
  }

  // Default mode - absolute positioned overlay (legacy, not used with drawer system)
  return (
    <>
      {/* Top right: Session info and share button */}
      <div className="absolute top-4 right-4 z-40 flex items-center gap-2">
        <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-gray-900/90 border border-gray-800">
          <Users className="w-4 h-4 text-violet-400" />
          <span className="text-sm text-white">
            {Object.keys(participants).length} watching
          </span>
        </div>
        <button
          onClick={handleCopyUrl}
          className={`p-2 rounded-lg transition-colors ${
            copied
              ? 'bg-green-600 text-white'
              : 'bg-violet-600 hover:bg-violet-500 text-white'
          }`}
          title="Copy share URL"
        >
          {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
        </button>
        <button
          onClick={leaveSession}
          className="p-2 rounded-lg bg-gray-800 hover:bg-red-600 text-gray-400 hover:text-white transition-colors"
          title="Leave session"
        >
          <LogOut className="w-4 h-4" />
        </button>
      </div>

      {/* Right side: Host controls and chat */}
      <div className="absolute top-20 right-4 z-40 w-72 flex flex-col gap-3 max-h-[calc(100vh-12rem)]">
        {/* Host controls */}
        <HostControls
          isHost={isHost}
          hostNickname={hostParticipant?.nickname}
        />

        {/* Chat panel */}
        <ChatPanel
          messages={chatMessages}
          onSendMessage={handleSendChat}
          disabled={!isConnected}
        />
      </div>
    </>
  );
}
