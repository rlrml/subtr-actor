import { Crown, UserCheck } from 'lucide-react';

interface HostControlsProps {
  isHost: boolean;
  hostNickname?: string;
}

/**
 * Host status indicator
 * Note: Host transfer is now done via the participant list
 */
export function HostControls({
  isHost,
  hostNickname,
}: HostControlsProps) {
  if (isHost) {
    return (
      <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-yellow-500/10 border border-yellow-500/20">
        <Crown className="w-4 h-4 text-yellow-500" />
        <span className="text-sm text-yellow-200">
          You are the host
        </span>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-gray-800/50 border border-gray-700">
      <UserCheck className="w-4 h-4 text-gray-400" />
      <span className="text-sm text-gray-400">
        Host: <span className="text-white">{hostNickname || 'Unknown'}</span>
      </span>
    </div>
  );
}
