import { useState } from 'react';
import { Trash2, Loader2, AlertTriangle, X } from 'lucide-react';
import { api } from '@/services/api';

interface DeleteReplayButtonProps {
  replayId: string;
  replayName?: string;
  onDelete?: (id: string) => void;
  variant?: 'icon' | 'button';
  className?: string;
}

export function DeleteReplayButton({
  replayId,
  replayName,
  onDelete,
  variant = 'icon',
  className = '',
}: DeleteReplayButtonProps) {
  const [isDeleting, setIsDeleting] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleDelete = async () => {
    setIsDeleting(true);
    setError(null);

    try {
      await api.delete(`/replays/${replayId}`);
      setShowConfirm(false);
      onDelete?.(replayId);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete replay');
    } finally {
      setIsDeleting(false);
    }
  };

  if (variant === 'icon') {
    return (
      <>
        <button
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            setShowConfirm(true);
          }}
          className={`p-2 rounded-lg bg-red-500/10 text-red-400 hover:bg-red-500/20 hover:text-red-300 transition-all ${className}`}
          title="Delete replay"
        >
          <Trash2 className="w-4 h-4" />
        </button>

        {showConfirm && (
          <ConfirmModal
            replayName={replayName}
            isDeleting={isDeleting}
            error={error}
            onConfirm={handleDelete}
            onCancel={() => {
              setShowConfirm(false);
              setError(null);
            }}
          />
        )}
      </>
    );
  }

  return (
    <>
      <button
        onClick={() => setShowConfirm(true)}
        className={`flex items-center gap-2 px-4 py-2 rounded-lg bg-red-500/10 text-red-400 hover:bg-red-500/20 hover:text-red-300 transition-all border border-red-500/20 ${className}`}
      >
        <Trash2 className="w-4 h-4" />
        Delete Replay
      </button>

      {showConfirm && (
        <ConfirmModal
          replayName={replayName}
          isDeleting={isDeleting}
          error={error}
          onConfirm={handleDelete}
          onCancel={() => {
            setShowConfirm(false);
            setError(null);
          }}
        />
      )}
    </>
  );
}

interface ConfirmModalProps {
  replayName?: string;
  isDeleting: boolean;
  error: string | null;
  onConfirm: () => void;
  onCancel: () => void;
}

function ConfirmModal({
  replayName,
  isDeleting,
  error,
  onConfirm,
  onCancel,
}: ConfirmModalProps) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm"
      onClick={(e) => {
        e.stopPropagation();
        e.preventDefault();
        if (e.target === e.currentTarget) onCancel();
      }}
    >
      <div className="bg-gray-900 border border-gray-700 rounded-xl max-w-md w-full p-6 shadow-2xl">
        <div className="flex items-start gap-4">
          <div className="w-12 h-12 rounded-full bg-red-500/20 flex items-center justify-center flex-shrink-0">
            <AlertTriangle className="w-6 h-6 text-red-400" />
          </div>
          <div className="flex-1">
            <h3 className="text-lg font-semibold text-white mb-2">Delete Replay?</h3>
            <p className="text-gray-400 text-sm">
              Are you sure you want to delete{' '}
              {replayName ? (
                <span className="text-gray-200 font-medium">"{replayName}"</span>
              ) : (
                'this replay'
              )}
              ? This action cannot be undone.
            </p>
          </div>
          <button
            onClick={(e) => {
              e.stopPropagation();
              e.preventDefault();
              onCancel();
            }}
            className="text-gray-500 hover:text-gray-300 transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {error && (
          <div className="mt-4 p-3 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
            {error}
          </div>
        )}

        <div className="flex gap-3 mt-6">
          <button
            onClick={(e) => {
              e.stopPropagation();
              e.preventDefault();
              onCancel();
            }}
            disabled={isDeleting}
            className="flex-1 px-4 py-2 rounded-lg bg-gray-800 text-gray-300 hover:bg-gray-700 hover:text-white transition-colors disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              e.preventDefault();
              onConfirm();
            }}
            disabled={isDeleting}
            className="flex-1 px-4 py-2 rounded-lg bg-red-600 text-white hover:bg-red-500 transition-colors disabled:opacity-50 flex items-center justify-center gap-2"
          >
            {isDeleting ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                Deleting...
              </>
            ) : (
              <>
                <Trash2 className="w-4 h-4" />
                Delete
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
