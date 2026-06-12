import { useCallback, useState } from 'react';
import { Upload, FileBox, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { cn } from '@/lib/utils';

interface MeshUploaderProps {
  onFileSelect: (file: File) => void;
  onCancel?: () => void;
  compact?: boolean;
  currentFile?: string;
  isLoading?: boolean;
  progress?: number;
}

export function MeshUploader({
  onFileSelect,
  onCancel,
  compact = false,
  currentFile,
  isLoading = false,
  progress = 0,
}: MeshUploaderProps) {
  const [isDragging, setIsDragging] = useState(false);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const validateFile = (file: File): string | null => {
    const ext = file.name.toLowerCase();
    if (!ext.endsWith('.glb') && !ext.endsWith('.gltf')) {
      return 'Invalid file format. Only GLB and GLTF files are accepted.';
    }

    const maxSize = 50 * 1024 * 1024; // 50MB
    if (file.size > maxSize) {
      return `File too large (${(file.size / 1024 / 1024).toFixed(1)}MB). Maximum size is 50MB.`;
    }

    return null;
  };

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setIsDragging(false);

      const file = e.dataTransfer.files[0];
      if (file) {
        const error = validateFile(file);
        if (!error) {
          onFileSelect(file);
        }
      }
    },
    [onFileSelect]
  );

  const handleFileInput = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) {
        const error = validateFile(file);
        if (!error) {
          onFileSelect(file);
        }
      }
      // Reset input so same file can be selected again
      e.target.value = '';
    },
    [onFileSelect]
  );

  // Loading state with progress
  if (isLoading) {
    return (
      <div className="flex flex-col gap-3 p-6 rounded-lg border-2 border-violet-500/50 bg-violet-500/10">
        <div className="flex items-center justify-between">
          <span className="text-sm text-zinc-300">Loading mesh...</span>
          {onCancel && (
            <Button
              variant="ghost"
              size="sm"
              className="h-6 w-6 p-0"
              onClick={onCancel}
            >
              <X className="h-4 w-4" />
            </Button>
          )}
        </div>
        <Progress value={progress * 100} className="h-2" />
        <span className="text-xs text-zinc-500">{Math.round(progress * 100)}%</span>
      </div>
    );
  }

  if (compact) {
    return (
      <div className="flex items-center gap-2">
        <FileBox className="h-4 w-4 text-zinc-400" />
        <span className="text-sm text-zinc-300 truncate flex-1">{currentFile}</span>
        <label className="cursor-pointer">
          <input
            type="file"
            accept=".glb,.gltf"
            onChange={handleFileInput}
            className="hidden"
          />
          <Button variant="outline" size="sm" asChild>
            <span>Replace</span>
          </Button>
        </label>
      </div>
    );
  }

  return (
    <div
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      className={cn(
        'flex flex-col items-center justify-center gap-4 p-8 rounded-lg border-2 border-dashed transition-colors',
        isDragging
          ? 'border-violet-500 bg-violet-500/10'
          : 'border-zinc-700 bg-zinc-800/50 hover:border-zinc-600'
      )}
    >
      <Upload className={cn('h-12 w-12', isDragging ? 'text-violet-400' : 'text-zinc-500')} />
      <div className="text-center">
        <p className="text-zinc-300 font-medium">Drop your mesh here</p>
        <p className="text-zinc-500 text-sm mt-1">GLB or GLTF format, max 50MB</p>
      </div>
      <label className="cursor-pointer">
        <input
          type="file"
          accept=".glb,.gltf"
          onChange={handleFileInput}
          className="hidden"
        />
        <Button variant="secondary" asChild>
          <span>Browse files</span>
        </Button>
      </label>
    </div>
  );
}
