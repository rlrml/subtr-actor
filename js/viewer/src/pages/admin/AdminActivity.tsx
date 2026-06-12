import { useState } from 'react';
import { User, Play, MessageSquare, Heart, AlertCircle, CheckCircle, FileText } from 'lucide-react';
import { useActivityFeed, ActivityEvent } from '@/hooks/useAdminApi';
import { formatDistanceToNow } from 'date-fns';

const eventTypes = [
  { value: '', label: 'All events' },
  { value: 'user_registered', label: 'User registrations' },
  { value: 'replay_uploaded', label: 'Replay uploads' },
  { value: 'replay_completed', label: 'Replay completed' },
  { value: 'replay_failed', label: 'Replay failed' },
  { value: 'comment_created', label: 'Comments' },
  { value: 'like_created', label: 'Likes' },
  { value: 'feedback_created', label: 'Feedback' },
];

const eventIcons: Record<string, typeof User> = {
  user_registered: User,
  replay_uploaded: Play,
  replay_completed: CheckCircle,
  replay_failed: AlertCircle,
  comment_created: MessageSquare,
  like_created: Heart,
  feedback_created: FileText,
};

const eventColors: Record<string, string> = {
  user_registered: 'text-cyan-400 bg-cyan-500/20',
  replay_uploaded: 'text-violet-400 bg-violet-500/20',
  replay_completed: 'text-green-400 bg-green-500/20',
  replay_failed: 'text-red-400 bg-red-500/20',
  comment_created: 'text-blue-400 bg-blue-500/20',
  like_created: 'text-pink-400 bg-pink-500/20',
  feedback_created: 'text-yellow-400 bg-yellow-500/20',
};

export default function AdminActivity() {
  const [typeFilter, setTypeFilter] = useState('');
  const { data: events, isLoading, error } = useActivityFeed(50, typeFilter || undefined);

  const renderEvent = (event: ActivityEvent) => {
    const Icon = eventIcons[event.type] || Play;
    const colorClass = eventColors[event.type] || 'text-gray-400 bg-gray-500/20';

    return (
      <div
        key={event.id}
        className="flex items-start gap-4 p-4 hover:bg-gray-800/30 rounded-lg transition-colors"
      >
        <div className={`p-2.5 rounded-lg ${colorClass}`}>
          <Icon className="w-5 h-5" />
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm text-white">{event.title}</p>
          <p className="text-xs text-gray-500 mt-1">
            {formatDistanceToNow(new Date(event.createdAt), { addSuffix: true })}
          </p>
        </div>
        <span className="text-xs text-gray-500 capitalize">
          {event.type.replace(/_/g, ' ')}
        </span>
      </div>
    );
  };

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Activity Feed</h1>
          <p className="text-gray-400 mt-1">Recent platform activity</p>
        </div>
        <select
          value={typeFilter}
          onChange={(e) => setTypeFilter(e.target.value)}
          className="px-4 py-2 bg-gray-900/50 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-violet-500"
        >
          {eventTypes.map((type) => (
            <option key={type.value} value={type.value}>
              {type.label}
            </option>
          ))}
        </select>
      </div>

      {/* Activity List */}
      <div className="bg-gray-900/50 border border-gray-800 rounded-xl overflow-hidden">
        {isLoading ? (
          <div className="p-8 text-center">
            <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-violet-500 mx-auto" />
          </div>
        ) : error ? (
          <div className="p-8 text-center text-red-400">
            Failed to load activity feed
          </div>
        ) : events.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            No activity found
          </div>
        ) : (
          <div className="divide-y divide-gray-800/50">
            {events.map(renderEvent)}
          </div>
        )}
      </div>
    </div>
  );
}
