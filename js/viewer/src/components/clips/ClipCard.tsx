/**
 * ClipCard
 *
 * Display a clip in a card format with thumbnail, title, stats, and actions.
 *
 * Feature: 024-clip-system (US3 - Management)
 */

import { Link } from 'react-router-dom';
import { Play, Heart, Eye, MessageCircle, Trash2, Edit, Film, Video, Clock } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { ClipListItem } from '@/api/clips';
import { formatClipDuration } from '@/api/clips';

interface ClipCardProps {
  clip: ClipListItem;
  onDelete?: (id: string) => void;
  onEdit?: (id: string) => void;
  showActions?: boolean;
  className?: string;
}

export function ClipCard({
  clip,
  onDelete,
  onEdit,
  showActions = false,
  className,
}: ClipCardProps) {
  const duration = formatClipDuration(clip.startTime, clip.endTime);

  const handleDelete = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    onDelete?.(clip.id);
  };

  const handleEdit = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    onEdit?.(clip.id);
  };

  return (
    <Link
      to={`/clips/${clip.id}`}
      className={cn(
        "group block bg-zinc-800/80 rounded-xl overflow-hidden border border-zinc-700/50 hover:border-zinc-600 transition-all hover:shadow-lg",
        className
      )}
    >
      {/* Thumbnail / Preview */}
      <div className="relative aspect-video bg-zinc-900">
        {/* Thumbnail or fallback gradient */}
        {clip.thumbnailUrl ? (
          <img
            src={clip.thumbnailUrl}
            alt={clip.title}
            className="absolute inset-0 w-full h-full object-cover"
          />
        ) : (
          <div className="absolute inset-0 bg-gradient-to-br from-blue-600/20 to-purple-600/20" />
        )}

        {/* Camera mode indicator */}
        <div className="absolute top-2 left-2">
          <div className={cn(
            "flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium",
            clip.cameraMode === 'capture'
              ? "bg-blue-500/80 text-white"
              : "bg-purple-500/80 text-white"
          )}>
            {clip.cameraMode === 'capture' ? (
              <Video className="w-3 h-3" />
            ) : (
              <Film className="w-3 h-3" />
            )}
            <span className="capitalize">{clip.cameraMode}</span>
          </div>
        </div>

        {/* Creator avatar */}
        <div className="absolute top-2 right-2">
          {clip.creator.avatarUrl ? (
            <img
              src={clip.creator.avatarUrl}
              alt={clip.creator.username}
              className="w-8 h-8 rounded-full border-2 border-white/20 shadow-lg"
            />
          ) : (
            <div className="w-8 h-8 rounded-full bg-gradient-to-br from-violet-600 to-blue-600 border-2 border-white/20 flex items-center justify-center shadow-lg">
              <span className="text-sm font-medium text-white">
                {clip.creator.username.charAt(0).toUpperCase()}
              </span>
            </div>
          )}
        </div>

        {/* Duration */}
        <div className="absolute bottom-2 right-2 px-2 py-1 bg-black/70 rounded text-xs text-white font-medium flex items-center gap-1">
          <Clock className="w-3 h-3" />
          {duration}
        </div>

        {/* Play overlay */}
        <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity bg-black/40">
          <div className="w-12 h-12 rounded-full bg-white/90 flex items-center justify-center">
            <Play className="w-6 h-6 text-zinc-900 ml-1" />
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="p-4 space-y-3">
        {/* Title */}
        <h3 className="font-semibold text-white line-clamp-2 group-hover:text-blue-400 transition-colors">
          {clip.title}
        </h3>

        {/* Creator */}
        <p className="text-sm text-zinc-400">{clip.creator.username}</p>

        {/* Stats */}
        <div className="flex items-center gap-4 text-sm text-zinc-500">
          <div className="flex items-center gap-1">
            <Eye className="w-4 h-4" />
            <span>{formatCount(clip.viewCount)}</span>
          </div>
          <div className="flex items-center gap-1">
            <Heart className="w-4 h-4" />
            <span>{formatCount(clip.likeCount)}</span>
          </div>
          <div className="flex items-center gap-1">
            <MessageCircle className="w-4 h-4" />
            <span>{formatCount(clip.commentCount)}</span>
          </div>
        </div>

        {/* Actions (owner only) */}
        {showActions && (onDelete || onEdit) && (
          <div className="flex items-center gap-2 pt-2 border-t border-zinc-700/50">
            {onEdit && (
              <button
                onClick={handleEdit}
                className="flex items-center gap-1 px-3 py-1.5 text-sm text-zinc-400 hover:text-white hover:bg-zinc-700 rounded-lg transition-colors"
              >
                <Edit className="w-4 h-4" />
                <span>Edit</span>
              </button>
            )}
            {onDelete && (
              <button
                onClick={handleDelete}
                className="flex items-center gap-1 px-3 py-1.5 text-sm text-red-400 hover:text-red-300 hover:bg-red-500/20 rounded-lg transition-colors"
              >
                <Trash2 className="w-4 h-4" />
                <span>Delete</span>
              </button>
            )}
          </div>
        )}
      </div>
    </Link>
  );
}

// Format large numbers with K/M suffix
function formatCount(count: number): string {
  if (count >= 1000000) {
    return `${(count / 1000000).toFixed(1)}M`;
  }
  if (count >= 1000) {
    return `${(count / 1000).toFixed(1)}K`;
  }
  return count.toString();
}
