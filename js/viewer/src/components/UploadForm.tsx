import { useState, useCallback, useRef, useEffect } from 'react';
import { Upload, File, X, UploadCloud, Globe, EyeOff, Loader2 } from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';
import { cn } from '@/lib/utils';
import { parseReplayHeader } from '@/utils/replayHeaderParser';

export interface UploadOptions {
  title: string;
  visibility: 'public' | 'unlisted';
}

interface UploadFormProps {
  onFileSelect: (file: File, options: UploadOptions) => void;
  disabled?: boolean;
  initialFile?: File | null;
}

export function UploadForm({ onFileSelect, disabled, initialFile }: UploadFormProps) {
  const [file, setFile] = useState<File | null>(initialFile || null);
  const [isDragging, setIsDragging] = useState(false);
  const [showConfigForm, setShowConfigForm] = useState(false);
  const [title, setTitle] = useState('');
  const [visibility, setVisibility] = useState<'public' | 'unlisted'>('public');
  const [isParsing, setIsParsing] = useState(false);
  const [detectedReplayName, setDetectedReplayName] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Set initial file if provided
  useEffect(() => {
    if (initialFile) {
      setFile(initialFile);
      setShowConfigForm(true);
    }
  }, [initialFile]);

  const handleFileChange = useCallback(async (selectedFile: File | null) => {
    if (selectedFile && selectedFile.name.endsWith('.replay')) {
      setFile(selectedFile);
      setShowConfigForm(true);
      // Reset form fields
      setTitle('');
      setVisibility('public');
      setDetectedReplayName(null);

      // Parse replay header to extract replay name
      setIsParsing(true);
      try {
        const headerInfo = await parseReplayHeader(selectedFile);
        if (headerInfo.replayName) {
          setDetectedReplayName(headerInfo.replayName);
          setTitle(headerInfo.replayName);
        }
      } catch (error) {
        console.warn('Failed to parse replay header:', error);
      } finally {
        setIsParsing(false);
      }
    }
  }, []);

  const handleSubmit = useCallback(() => {
    if (file) {
      onFileSelect(file, { title, visibility });
    }
  }, [file, title, visibility, onFileSelect]);

  const handleCancel = useCallback(() => {
    setFile(null);
    setShowConfigForm(false);
    setTitle('');
    setVisibility('public');
    setDetectedReplayName(null);
    setIsParsing(false);
    if (inputRef.current) {
      inputRef.current.value = '';
    }
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const droppedFile = e.dataTransfer.files[0];
    handleFileChange(droppedFile);
  }, [handleFileChange]);

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0] || null;
    handleFileChange(selectedFile);
  }, [handleFileChange]);

  const handleClick = useCallback(() => {
    inputRef.current?.click();
  }, []);

  const handleClear = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    handleCancel();
  }, [handleCancel]);

  // Configuration form (shown after file selection)
  if (showConfigForm && file) {
    return (
      <div className="relative rounded-2xl">
        {/* Animated border */}
        <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-violet-500 to-blue-500 p-[1px] shadow-lg shadow-violet-500/20">
          <div className="w-full h-full rounded-[15px] bg-gray-900" />
        </div>

        <div className="relative rounded-2xl p-6 space-y-5">
          {/* File info header */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="relative">
                <div className="absolute inset-0 bg-gradient-to-r from-violet-600 to-blue-600 rounded-xl blur-lg opacity-50" />
                <div className="relative w-12 h-12 rounded-xl bg-gradient-to-r from-violet-600 to-blue-600 flex items-center justify-center">
                  <File className="w-6 h-6 text-white" />
                </div>
              </div>
              <div>
                <p className="font-semibold text-white">{file.name}</p>
                <p className="text-sm text-gray-400">
                  {(file.size / 1024 / 1024).toFixed(2)} MB
                </p>
              </div>
            </div>
            {!disabled && (
              <button
                onClick={handleClear}
                className="p-2 rounded-lg hover:bg-gray-800 transition-colors border border-gray-700 hover:border-gray-600"
              >
                <X className="w-4 h-4 text-gray-400 hover:text-white" />
              </button>
            )}
          </div>

          {/* Title input */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-300 flex items-center gap-2">
              Title <span className="text-gray-500">(optional)</span>
              {isParsing && (
                <Loader2 className="w-3 h-3 animate-spin text-violet-400" />
              )}
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder={detectedReplayName ? `Detected: ${detectedReplayName}` : 'Leave empty to use replay name'}
              maxLength={100}
              disabled={disabled || isParsing}
              className="w-full px-4 py-2.5 rounded-lg bg-gray-800/50 border border-gray-700 text-white placeholder-gray-500 focus:outline-none focus:border-violet-500 focus:ring-1 focus:ring-violet-500 transition-colors"
            />
            <p className="text-xs text-gray-500">
              {title.length}/100 characters • Uses replay name or map name if empty
            </p>
          </div>

          {/* Visibility selection */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-300">
              Visibility
            </label>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 sm:gap-3">
              <button
                type="button"
                onClick={() => setVisibility('public')}
                disabled={disabled}
                className={cn(
                  'flex items-center gap-3 p-3 rounded-lg border transition-all min-h-[60px]',
                  visibility === 'public'
                    ? 'border-green-500 bg-green-500/10 text-green-400'
                    : 'border-gray-700 bg-gray-800/50 text-gray-400 hover:border-gray-600 hover:text-gray-300'
                )}
              >
                <Globe className="w-5 h-5 shrink-0" />
                <div className="text-left">
                  <div className="font-medium text-sm sm:text-base">Public</div>
                  <div className="text-xs opacity-70">Visible in replay list</div>
                </div>
              </button>
              <button
                type="button"
                onClick={() => setVisibility('unlisted')}
                disabled={disabled}
                className={cn(
                  'flex items-center gap-3 p-3 rounded-lg border transition-all min-h-[60px]',
                  visibility === 'unlisted'
                    ? 'border-gray-400 bg-gray-500/10 text-gray-300'
                    : 'border-gray-700 bg-gray-800/50 text-gray-400 hover:border-gray-600 hover:text-gray-300'
                )}
              >
                <EyeOff className="w-5 h-5 shrink-0" />
                <div className="text-left">
                  <div className="font-medium text-sm sm:text-base">Unlisted</div>
                  <div className="text-xs opacity-70">Only via direct link</div>
                </div>
              </button>
            </div>
          </div>

          {/* Action buttons */}
          <div className="flex gap-3 pt-2">
            <button
              type="button"
              onClick={handleCancel}
              disabled={disabled}
              className="flex-1 px-4 py-2.5 rounded-lg border border-gray-700 text-gray-400 hover:text-white hover:border-gray-600 transition-colors disabled:opacity-50"
            >
              Cancel
            </button>
            <GradientButton
              type="button"
              onClick={handleSubmit}
              disabled={disabled}
              className="flex-1"
            >
              <Upload className="w-4 h-4" />
              Upload Replay
            </GradientButton>
          </div>
        </div>
      </div>
    );
  }

  // Dropzone (shown when no file selected)
  return (
    <div
      onClick={!disabled ? handleClick : undefined}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      className={cn(
        'relative rounded-2xl cursor-pointer transition-all duration-300',
        disabled && 'opacity-50 cursor-not-allowed'
      )}
    >
      {/* Animated border */}
      <div className={cn(
        'absolute inset-0 rounded-2xl transition-all duration-300 shadow-lg shadow-violet-500/20',
        isDragging
          ? 'bg-gradient-to-r from-violet-500 via-blue-500 to-cyan-500 p-[2px]'
          : 'bg-gradient-to-br from-violet-500 to-blue-500 p-[1px]'
      )}>
        <div className="w-full h-full rounded-[15px] bg-gray-900" />
      </div>

      {/* Content */}
      <div className="relative rounded-2xl p-4 sm:p-8">
        <input
          ref={inputRef}
          type="file"
          accept=".replay"
          onChange={handleInputChange}
          disabled={disabled}
          className="hidden"
        />

        <div className="text-center py-6 sm:py-8">
          {/* Animated icon */}
          <div className="relative w-14 h-14 sm:w-20 sm:h-20 mx-auto mb-4 sm:mb-6">
            <div className={cn(
              'absolute inset-0 rounded-2xl bg-gradient-to-r from-violet-600 to-blue-600 blur-xl transition-opacity duration-300',
              isDragging ? 'opacity-60' : 'opacity-30'
            )} />
            <div className={cn(
              'relative w-full h-full rounded-2xl bg-gradient-to-br from-violet-600 to-blue-600 flex items-center justify-center transition-transform duration-300',
              isDragging && 'scale-110'
            )}>
              <UploadCloud className={cn(
                'w-6 h-6 sm:w-10 sm:h-10 text-white transition-transform duration-300',
                isDragging && '-translate-y-1'
              )} />
            </div>
          </div>

          <h3 className={cn(
            'text-lg sm:text-xl font-bold mb-2 transition-colors duration-300',
            isDragging
              ? 'bg-gradient-to-r from-violet-400 to-blue-400 bg-clip-text text-transparent'
              : 'text-white'
          )}>
            {isDragging ? 'Drop your replay here!' : 'Upload your replay'}
          </h3>

          <p className="text-gray-400 text-sm sm:text-base mb-4 sm:mb-6 px-2">
            {isDragging
              ? 'Release to start uploading'
              : 'Drag and drop your .replay file or click to browse'}
          </p>

          <GradientButton type="button" size="lg" disabled={disabled} className="w-full sm:w-auto">
            <Upload className="w-5 h-5" />
            Choose File
          </GradientButton>
        </div>
      </div>

      {/* Drag overlay effect */}
      {isDragging && (
        <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-violet-500/10 to-blue-500/10 pointer-events-none" />
      )}
    </div>
  );
}
