import { Link } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import {
  Activity,
  AlertTriangle,
  HelpCircle,
  ArrowLeft,
  Zap,
  Clock,
  Database,
  Gamepad2,
} from 'lucide-react';
import { AuthCard } from '@/components/ui/GradientCard';

export default function ReplayQualityFaq() {
  return (
    <div className="max-w-3xl mx-auto px-4 py-8">
      <SEOHead
        title="Replay Quality FAQ"
        description="Understand why some Rocket League replays have quality issues and what causes stuttering or teleporting objects in the viewer."
      />

      {/* Back link */}
      <Link
        to="/replays"
        className="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors mb-6"
      >
        <ArrowLeft className="w-4 h-4" />
        Back to replays
      </Link>

      {/* Header */}
      <div className="flex items-center gap-3 mb-8">
        <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-amber-500 to-orange-500 flex items-center justify-center">
          <HelpCircle className="w-6 h-6 text-white" />
        </div>
        <div>
          <h1 className="text-2xl font-bold text-white">Replay Quality</h1>
          <p className="text-gray-400 text-sm">
            Why some replays have quality issues
          </p>
        </div>
      </div>

      {/* Main explanation */}
      <AuthCard className="mb-6">
        <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
          <Activity className="w-5 h-5 text-amber-400" />
          Why does my replay have bad quality?
        </h2>

        <div className="space-y-4 text-gray-300">
          <p>
            Rocket League computes physics at <strong className="text-white">120 Hz</strong> (120 times per second),
            but replays are recorded at approximately <strong className="text-white">30 Hz</strong>.
          </p>
          <p>
            Normally, the recorded velocities allow us to reconstruct smooth movement between frames.
            However, in some replays, the <strong className="text-amber-400">velocities don't match the positions</strong>.
          </p>
          <p>
            For example, the ball might have a high velocity but barely moves between two frames.
            This desynchronization causes the stuttering and "teleporting" effect you see in playback.
          </p>
        </div>
      </AuthCard>

      {/* Technical details */}
      <AuthCard className="mb-6">
        <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
          <Clock className="w-5 h-5 text-blue-400" />
          The 120Hz vs 30Hz Problem
        </h2>

        <div className="space-y-4">
          <div className="grid grid-cols-3 gap-3">
            <div className="p-3 rounded-lg bg-gray-800/50 border border-gray-700/50 text-center">
              <div className="text-2xl font-bold text-violet-400">120 Hz</div>
              <div className="text-xs text-gray-400 mt-1">Physics engine</div>
            </div>
            <div className="p-3 rounded-lg bg-gray-800/50 border border-gray-700/50 text-center">
              <div className="text-2xl font-bold text-blue-400">60 Hz</div>
              <div className="text-xs text-gray-400 mt-1">Network sync</div>
            </div>
            <div className="p-3 rounded-lg bg-gray-800/50 border border-gray-700/50 text-center">
              <div className="text-2xl font-bold text-amber-400">30 Hz</div>
              <div className="text-xs text-gray-400 mt-1">Replay recording</div>
            </div>
          </div>

          <p className="text-gray-400 text-sm">
            When a collision happens mid-frame (between two recording points),
            the velocity is captured at the physics tick but the position is only recorded later.
            This creates the mismatch.
          </p>
        </div>
      </AuthCard>

      {/* Why some replays are affected */}
      <AuthCard className="mb-6">
        <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
          <AlertTriangle className="w-5 h-5 text-orange-400" />
          Why are some replays affected and not others?
        </h2>

        <div className="space-y-4 text-gray-300">
          <p>
            <strong className="text-white">We don't know exactly.</strong> The issue seems related to network
            conditions during the original match, but even private matches or LAN games can be affected.
          </p>
          <p className="text-gray-400 text-sm">
            There's no clear pattern — some online matches have perfect quality while some offline matches don't.
            The quality depends on how the game recorded the data, not on how we process it.
          </p>
        </div>
      </AuthCard>

      {/* Why smooth in game */}
      <AuthCard className="mb-6">
        <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
          <Gamepad2 className="w-5 h-5 text-green-400" />
          Why are replays smooth in Rocket League?
        </h2>

        <div className="space-y-4 text-gray-300">
          <p>
            Rocket League has a major advantage: it has access to the <strong className="text-white">full physics engine</strong>.
          </p>
          <p>
            When you watch a replay in-game, Rocket League doesn't just play back the recorded positions.
            It <strong className="text-cyan-400">re-simulates the entire physics</strong> at 120 Hz using the recorded
            inputs and key events as reference points.
          </p>
          <p className="text-gray-400 text-sm">
            We can only work with the exported data — positions and velocities sampled at 30 Hz.
            We don't have access to the physics engine, so we cannot re-simulate the match.
            We can only interpolate between the recorded data points.
          </p>
        </div>
      </AuthCard>

      {/* Can it be fixed */}
      <AuthCard className="mb-6">
        <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
          <Zap className="w-5 h-5 text-yellow-400" />
          Can the quality be improved?
        </h2>

        <div className="space-y-4">
          <p className="text-gray-300">
            <strong className="text-green-400">Yes!</strong> Thanks to an algorithm discovered by{' '}
            <strong className="text-violet-400">TitaniteChuck</strong>, we can now significantly improve
            replay quality by filtering out inconsistent frames.
          </p>

          <div className="p-4 rounded-lg bg-violet-500/10 border border-violet-500/30">
            <h3 className="font-semibold text-violet-400 mb-2">The TitaniteChuck Algorithm</h3>
            <p className="text-gray-300 text-sm mb-3">
              The key insight is that bad frames have <strong className="text-white">stable velocities but incorrect positions</strong>.
              When velocity between two frames is consistent (less than 10% variation), but the actual position
              doesn't match what the velocity predicts (more than 15% error), we skip that frame.
            </p>
            <p className="text-gray-400 text-sm">
              By comparing each frame to the last <em>accepted</em> frame (not just the previous one),
              we can filter out the corrupted data while preserving the valid motion data.
            </p>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-center">
              <div className="text-2xl font-bold text-red-400">~30%</div>
              <div className="text-xs text-gray-400 mt-1">Before filtering</div>
            </div>
            <div className="p-3 rounded-lg bg-green-500/10 border border-green-500/30 text-center">
              <div className="text-2xl font-bold text-green-400">~65%</div>
              <div className="text-xs text-gray-400 mt-1">After filtering</div>
            </div>
          </div>

          <p className="text-gray-400 text-sm">
            We also apply advanced interpolation (Hermite splines with velocity-based tangents)
            to smooth the remaining data for the best possible playback experience.
          </p>
        </div>
      </AuthCard>

      {/* Credits */}
      <AuthCard className="mb-6">
        <div className="p-4 rounded-lg bg-gradient-to-br from-violet-500/10 to-purple-500/10 border border-violet-500/30">
          <h2 className="text-lg font-semibold text-violet-400 mb-2">
            Special Thanks
          </h2>
          <p className="text-gray-300 mb-3">
            A huge thank you to <strong className="text-violet-400">TitaniteChuck</strong> for discovering
            and sharing the frame filtering algorithm that dramatically improves replay quality.
          </p>
          <p className="text-gray-400 text-sm">
            If you have ideas for further improvements or notice any issues, we'd love to hear from you!
          </p>
          <Link
            to="/feedback/new"
            className="inline-flex items-center gap-2 mt-3 px-4 py-2 rounded-lg bg-violet-500/20 text-violet-400 hover:bg-violet-500/30 transition-colors"
          >
            Share your feedback
          </Link>
        </div>
      </AuthCard>

      {/* Quality scores explained */}
      <AuthCard className="mb-6">
        <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
          <Database className="w-5 h-5 text-green-400" />
          What do the quality scores mean?
        </h2>

        <div className="space-y-4">
          <div className="space-y-3">
            <div className="flex items-center gap-3 p-3 rounded-lg bg-green-500/10 border border-green-500/30">
              <div className="w-10 h-10 rounded-lg bg-green-500/20 flex items-center justify-center">
                <Activity className="w-5 h-5 text-green-400" />
              </div>
              <div>
                <div className="font-medium text-green-400">Good Quality (70%+)</div>
                <div className="text-sm text-gray-400">Smooth playback, velocities match positions well</div>
              </div>
            </div>

            <div className="flex items-center gap-3 p-3 rounded-lg bg-amber-500/10 border border-amber-500/30">
              <div className="w-10 h-10 rounded-lg bg-amber-500/20 flex items-center justify-center">
                <Activity className="w-5 h-5 text-amber-400" />
              </div>
              <div>
                <div className="font-medium text-amber-400">Acceptable Quality (45-69%)</div>
                <div className="text-sm text-gray-400">Occasional stuttering, generally watchable</div>
              </div>
            </div>

            <div className="flex items-center gap-3 p-3 rounded-lg bg-red-500/10 border border-red-500/30">
              <div className="w-10 h-10 rounded-lg bg-red-500/20 flex items-center justify-center">
                <Activity className="w-5 h-5 text-red-400" />
              </div>
              <div>
                <div className="font-medium text-red-400">Low Quality (&lt;45%)</div>
                <div className="text-sm text-gray-400">Noticeable stuttering, objects may appear to teleport</div>
              </div>
            </div>
          </div>

          <p className="text-gray-500 text-xs">
            The score is calculated by analyzing velocity consistency across all frames.
            It measures how well the recorded velocities predict actual position changes.
          </p>
        </div>
      </AuthCard>

      {/* Footer note */}
      <div className="text-center text-gray-500 text-sm">
        <p>
          Questions? Visit our{' '}
          <Link to="/feedback" className="text-violet-400 hover:underline">
            feedback hub
          </Link>{' '}
          or join the discussion.
        </p>
      </div>
    </div>
  );
}
