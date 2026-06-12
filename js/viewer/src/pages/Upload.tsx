import { useState, useCallback, useEffect } from 'react';
import { useLocation, Link } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import { Play, FileUp, Info, Camera, Users, Gauge, Layers, Zap, Eye } from 'lucide-react';
import { UploadForm, type UploadOptions } from '@/components/UploadForm';
import { UploadProgress, type UploadStatus } from '@/components/UploadProgress';
import { QualityWarning } from '@/components/QualityWarning';
import { AuthRequiredMessage } from '@/components/AuthRequiredMessage';
import { EmailVerificationRequired } from '@/components/EmailVerificationRequired';
import { useAuth } from '@/hooks/useAuth';
import { DuplicateAlert, type DuplicateInfo } from '@/components/upload/DuplicateAlert';
import { UploadSuccess, type RelatedReplayInfo } from '@/components/upload/UploadSuccess';
import { api } from '@/services/api';
import type { QualityMetrics } from '@/types/quality';
import { shouldShowWarning } from '@/types/quality';

interface UploadResponse {
  success: boolean;
  replay: {
    id: string;
    status: string;
    title?: string | null;
    qualityScore?: number | null;
    qualityMetrics?: QualityMetrics | null;
  };
  relatedReplays?: RelatedReplayInfo[];
}

const features = [
  {
    icon: Camera,
    title: 'Multiple Camera Modes',
    description: 'Free cam, ball cam, player cam - switch perspectives instantly',
    color: 'violet',
  },
  {
    icon: Users,
    title: 'Collaborative Viewing',
    description: 'Watch replays together with friends in real-time',
    color: 'blue',
  },
  {
    icon: Gauge,
    title: 'Speed Controls',
    description: 'Slow motion, fast forward, frame-by-frame analysis',
    color: 'cyan',
  },
  {
    icon: Layers,
    title: 'Detailed Stats',
    description: 'Player statistics, boost usage, positioning data',
    color: 'violet',
  },
  {
    icon: Zap,
    title: 'Fast Processing',
    description: 'Optimized binary format for instant playback',
    color: 'blue',
  },
  {
    icon: Eye,
    title: '3D Visualization',
    description: 'Full arena with cars, ball trails, and effects',
    color: 'cyan',
  },
];

export default function Upload() {
  const location = useLocation();
  const { isAuthenticated, isLoading: authLoading, user } = useAuth();

  const [file, setFile] = useState<File | null>(null);
  const [status, setStatus] = useState<UploadStatus>('idle');
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string>();
  const [qualityScore, setQualityScore] = useState<number | null>(null);
  const [qualityMetrics, setQualityMetrics] = useState<QualityMetrics | null>(null);
  const [duplicate, setDuplicate] = useState<DuplicateInfo | null>(null);
  // 033-replay-duplicate-detection / US2: Track successful upload for navigation
  const [successReplay, setSuccessReplay] = useState<{
    id: string;
    title?: string | null;
    relatedReplays?: RelatedReplayInfo[];
  } | null>(null);

  // Handle file passed from GlobalDropZone - just set the file, don't auto-upload
  useEffect(() => {
    const state = location.state as { file?: File } | null;
    if (state?.file && isAuthenticated) {
      setFile(state.file);
      // Clear the state - the UploadForm will handle showing the config dialog
      window.history.replaceState({}, document.title);
    }
  }, [location.state, isAuthenticated]);

  // Show loading state while checking auth
  if (authLoading) {
    return (
      <div className="flex items-center justify-center min-h-[50vh]">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-violet-500" />
      </div>
    );
  }

  // Show auth required message if not logged in
  if (!isAuthenticated) {
    return (
      <AuthRequiredMessage
        title="Sign In to Upload"
        message="You need to be signed in to upload replays. Create a free account to get started!"
        returnTo="/upload"
      />
    );
  }

  // Show email verification message if not verified
  if (user && !user.emailVerified) {
    return (
      <EmailVerificationRequired
        title="Verify Your Email"
        message="Please verify your email address to upload replays."
      />
    );
  }

  const handleUpload = useCallback(async (uploadFile: File, options: UploadOptions) => {
    setStatus('uploading');
    setProgress(0);
    setError(undefined);
    setQualityScore(null);
    setQualityMetrics(null);
    setDuplicate(null);
    setSuccessReplay(null);

    try {
      const formData = new FormData();
      formData.append('file', uploadFile);
      if (options.title) {
        formData.append('title', options.title);
      }
      formData.append('visibility', options.visibility);

      // Simulate progress (real progress would need XHR)
      const progressInterval = setInterval(() => {
        setProgress((prev) => Math.min(prev + 10, 90));
      }, 200);

      setStatus('processing');
      const response = await api.postForm<UploadResponse>('/replays', formData);

      clearInterval(progressInterval);
      setProgress(100);
      setStatus('success');

      // Store quality info for display
      const replayId = response.replay.id;
      const replayTitle = response.replay.title ?? null;
      const score = response.replay.qualityScore ?? null;
      const metrics = response.replay.qualityMetrics ?? null;

      setQualityScore(score);
      setQualityMetrics(metrics);

      // 033-replay-duplicate-detection / US2: Store replay info and stay on page
      // (removed automatic redirect to let user choose navigation)
      setSuccessReplay({
        id: replayId,
        title: replayTitle,
        relatedReplays: response.relatedReplays,
      });
    } catch (err) {
      // 033-replay-duplicate-detection: Handle duplicate replay response (409 Conflict)
      const apiError = err as { error?: string; duplicate?: DuplicateInfo; message?: string };
      if (apiError.error === 'DUPLICATE_REPLAY' && apiError.duplicate) {
        setStatus('idle');
        setDuplicate(apiError.duplicate);
        return;
      }

      setStatus('error');
      setError(err instanceof Error ? err.message : apiError.message || 'Upload failed');
    }
  }, []);

  const handleFileSelect = useCallback((selectedFile: File, options: UploadOptions) => {
    setFile(selectedFile);
    handleUpload(selectedFile, options);
  }, [handleUpload]);

  const isUploading = status === 'uploading' || status === 'processing';

  return (
    <div className="space-y-8">
      <SEOHead
        title="Upload Replay"
        description="Upload your Rocket League .replay file to watch it in stunning 3D with full playback controls, camera modes, and player statistics."
        noIndex
      />

      {/* Hero Header */}
      <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-violet-900/20 to-blue-900/20 border border-violet-500/20 p-8">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,_var(--tw-gradient-stops))] from-violet-600/10 via-transparent to-transparent" />

        <div className="relative flex flex-col sm:flex-row sm:items-center sm:justify-between gap-6">
          <div className="flex items-center gap-4">
            <div className="w-14 h-14 rounded-xl bg-gradient-to-br from-violet-600 to-blue-600 flex items-center justify-center">
              <FileUp className="w-7 h-7 text-white" />
            </div>
            <div>
              <h1 className="text-3xl font-bold bg-gradient-to-r from-violet-400 via-blue-400 to-cyan-400 bg-clip-text text-transparent">
                Upload Replay
              </h1>
              <p className="text-gray-400 mt-1">
                Upload your .replay file to start watching
              </p>
            </div>
          </div>

          <Link to="/replays">
            <button className="flex items-center gap-2 px-4 py-2.5 rounded-xl bg-gray-800/80 text-gray-300 hover:bg-gray-700 hover:text-white transition-all border border-gray-700">
              <Play className="w-4 h-4" />
              View Replays
            </button>
          </Link>
        </div>
      </div>

      {/* Main Content Grid */}
      <div className="grid lg:grid-cols-2 gap-8">
        {/* Left Column - Upload */}
        <div className="space-y-6">
          <UploadForm
            onFileSelect={handleFileSelect}
            disabled={isUploading}
            initialFile={file}
          />

          <UploadProgress
            status={status}
            progress={progress}
            error={error}
          />

          {/* Duplicate Alert - shown when upload is blocked due to duplicate (033-replay-duplicate-detection) */}
          {duplicate && (
            <DuplicateAlert
              duplicate={duplicate}
              onRetry={() => {
                setDuplicate(null);
                setFile(null);
              }}
              className="animate-in fade-in slide-in-from-bottom-2 duration-300"
            />
          )}

          {/* Success Panel - shown after successful upload with navigation options (US2) */}
          {status === 'success' && successReplay && (
            <UploadSuccess
              replayId={successReplay.id}
              title={successReplay.title}
              relatedReplays={successReplay.relatedReplays}
              onUploadAnother={() => {
                setStatus('idle');
                setFile(null);
                setSuccessReplay(null);
                setQualityScore(null);
                setQualityMetrics(null);
              }}
              className="animate-in fade-in slide-in-from-bottom-2 duration-300"
            />
          )}

          {/* Quality Warning - shown after successful upload if score is low */}
          {status === 'success' && qualityScore !== null && shouldShowWarning(qualityScore) && (
            <QualityWarning
              score={qualityScore}
              metrics={qualityMetrics}
              className="animate-in fade-in slide-in-from-bottom-2 duration-300 delay-100"
            />
          )}

          {status === 'error' && (
            <div className="text-center">
              <button
                onClick={() => {
                  setStatus('idle');
                  setFile(null);
                  setError(undefined);
                }}
                className="text-violet-400 hover:text-violet-300 underline font-medium"
              >
                Try again
              </button>
            </div>
          )}

          {/* How it works */}
          {status === 'idle' && (
            <div className="relative rounded-xl overflow-hidden">
              <div className="absolute inset-0 bg-gradient-to-r from-blue-500/5 to-cyan-500/5" />
              <div className="relative p-5 border border-blue-500/20 rounded-xl">
                <div className="flex items-start gap-3">
                  <div className="w-10 h-10 rounded-lg bg-blue-500/20 flex items-center justify-center flex-shrink-0">
                    <Info className="w-5 h-5 text-blue-400" />
                  </div>
                  <div className="space-y-2">
                    <h3 className="font-semibold text-white">How it works</h3>
                    <ul className="text-sm text-gray-400 space-y-1.5">
                      <li className="flex items-center gap-2">
                        <span className="w-1.5 h-1.5 rounded-full bg-violet-400" />
                        Upload your Rocket League .replay file
                      </li>
                      <li className="flex items-center gap-2">
                        <span className="w-1.5 h-1.5 rounded-full bg-blue-400" />
                        We process and compile it for 3D playback
                      </li>
                      <li className="flex items-center gap-2">
                        <span className="w-1.5 h-1.5 rounded-full bg-cyan-400" />
                        Watch your replay with full camera controls
                      </li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Supported formats */}
          {status === 'idle' && (
            <div className="flex items-center justify-center gap-4 text-xs text-gray-500">
              <span className="flex items-center gap-1.5">
                <span className="w-2 h-2 rounded-full bg-green-500/50" />
                .replay files supported
              </span>
              <span className="text-gray-700">•</span>
              <span>Max size: 10 MB</span>
            </div>
          )}
        </div>

        {/* Right Column - Features */}
        <div className="space-y-4">
          <h2 className="text-lg font-semibold text-white flex items-center gap-2">
            <span className="w-8 h-8 rounded-lg bg-gradient-to-br from-violet-600/20 to-blue-600/20 flex items-center justify-center">
              <Zap className="w-4 h-4 text-violet-400" />
            </span>
            What you'll get
          </h2>

          <div className="grid gap-3">
            {features.map((feature, index) => {
              const Icon = feature.icon;
              const colorClasses = {
                violet: 'bg-violet-500/10 border-violet-500/20 text-violet-400',
                blue: 'bg-blue-500/10 border-blue-500/20 text-blue-400',
                cyan: 'bg-cyan-500/10 border-cyan-500/20 text-cyan-400',
              };
              const iconBgClasses = {
                violet: 'bg-violet-500/20',
                blue: 'bg-blue-500/20',
                cyan: 'bg-cyan-500/20',
              };

              return (
                <div
                  key={index}
                  className={`p-4 rounded-xl border ${colorClasses[feature.color as keyof typeof colorClasses]} transition-all hover:scale-[1.02]`}
                >
                  <div className="flex items-start gap-3">
                    <div className={`w-9 h-9 rounded-lg ${iconBgClasses[feature.color as keyof typeof iconBgClasses]} flex items-center justify-center flex-shrink-0`}>
                      <Icon className="w-4.5 h-4.5" />
                    </div>
                    <div>
                      <h3 className="font-medium text-white text-sm">{feature.title}</h3>
                      <p className="text-xs text-gray-500 mt-0.5">{feature.description}</p>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>

          {/* Find replays hint */}
          <div className="mt-6 p-4 rounded-xl bg-gray-800/30 border border-gray-700/50 text-center">
            <p className="text-sm text-gray-400">
              Looking for your replay files?
            </p>
            <p className="text-xs text-gray-500 mt-1">
              Windows: <code className="text-violet-400 bg-violet-500/10 px-1.5 py-0.5 rounded">Documents\My Games\Rocket League\TAGame\Demos</code>
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
