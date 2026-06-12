import { useState, useEffect, useRef } from 'react';
import { Link } from 'react-router-dom';
import {
  SEOHead,
  StructuredData,
  createWebSiteStructuredData,
  createOrganizationStructuredData,
} from '@/components/SEO';
import {
  Upload,
  Eye,
  Camera,
  Zap,
  Users,
  BarChart3,
  Sparkles,
  Target,
  Gauge,
  ArrowRight,
  CheckCircle2,
  Check,
  Video,
  MousePointer2,
  MessageCircle,
  Share2,
  Link2,
  PlayCircle,
  UserPlus,
  FileCode,
  Loader2,
  MessageSquare,
  Lightbulb,
  Bug,
  ChevronUp,
  MapPin,
  Pencil,
  Clapperboard,
  Smartphone,
  Rocket,
  Heart,
  Shield,
} from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';
import { GradientCard, GlowCard } from '@/components/ui/GradientCard';
import { OctaneHero } from '@/components/OctaneHero';
import { Logo } from '@/components/ui/Logo';
import { HotReplays } from '@/components/HotReplays';
import { GlobalStats } from '@/components/GlobalStats';
import { StatsShowcase } from '@/components/StatsShowcase';
import { DiscordWidget } from '@/components/DiscordWidget';
import { LatestNews } from '@/components/LatestNews';

const collabFeatures = [
  {
    icon: Link2,
    title: 'One-Click Sharing',
    description: 'Generate a shareable link instantly. Anyone with the link can join your viewing session.'
  },
  {
    icon: PlayCircle,
    title: 'Synchronized Playback',
    description: 'Play, pause, and seek together. Everyone stays in sync with the host\'s controls.'
  },
  {
    icon: MessageCircle,
    title: 'Built-in Chat',
    description: 'Discuss plays in real-time with integrated chat. No need for external apps.'
  },
  {
    icon: Eye,
    title: 'Independent Cameras',
    description: 'Each viewer controls their own camera. Watch from your preferred angle.'
  },
  {
    icon: MapPin,
    title: 'Ping System',
    description: 'Point out specific locations on the field. Pings are visible to all viewers in real-time.'
  },
  {
    icon: Pencil,
    title: 'Collaborative Drawing',
    description: 'Draw on the terrain to illustrate strategies. Perfect for coaching and replay analysis.'
  },
];

const cameraModes = [
  {
    icon: Eye,
    name: 'Free Camera',
    description: 'Full 360° control with keyboard and mouse. Explore the arena from any angle.',
    keys: 'WASD + Mouse'
  },
  {
    icon: Target,
    name: 'Ball Cam',
    description: 'Automatically follows the ball with smooth orbital movement. Scroll to zoom.',
    keys: 'Click on ball'
  },
  {
    icon: Users,
    name: 'Player Cam',
    description: 'Lock onto any player and follow their perspective throughout the match.',
    keys: 'Click on player'
  },
];

const features = [
  {
    icon: Upload,
    title: 'Drag & Drop Upload',
    description: 'Simply drag your .replay files anywhere on the page. No complicated setup required.'
  },
  {
    icon: Video,
    title: 'Full 3D Playback',
    description: 'Watch your replays in a fully rendered 3D environment with realistic physics visualization.'
  },
  {
    icon: Gauge,
    title: 'Boost Tracking',
    description: 'Real-time boost gauges for all players. See who had boost advantage at key moments.'
  },
  {
    icon: Sparkles,
    title: 'Visual Effects',
    description: 'Boost trails, ball trails, demolition explosions, and more for an immersive experience.'
  },
  {
    icon: BarChart3,
    title: 'Match Statistics',
    description: 'Goals, assists, saves, shots, and detailed player statistics at your fingertips.'
  },
  {
    icon: MousePointer2,
    title: 'Interactive Timeline',
    description: 'Scrub through the match, adjust playback speed, and jump to key moments instantly.'
  },
  {
    icon: Shield,
    title: 'Cheat Detection',
    description: 'Automatic analysis powered by whosbotting.com. Identify cheaters in your matches.'
  },
];

const steps = [
  {
    number: '01',
    title: 'Upload',
    description: 'Drop your Rocket League .replay file',
    icon: Upload
  },
  {
    number: '02',
    title: 'Process',
    description: 'We parse and optimize your replay',
    icon: Zap
  },
  {
    number: '03',
    title: 'Watch Together',
    description: 'Invite friends & analyze as a team',
    icon: Users
  },
];

// Compilation demo component
const compilationSteps = [
  { label: 'Parsing replay file...', duration: 800 },
  { label: 'Extracting frame data...', duration: 600 },
  { label: 'Compiling physics...', duration: 700 },
  { label: 'Optimizing binary...', duration: 500 },
];

const totalCompilationDuration = compilationSteps.reduce((sum, s) => sum + s.duration, 0);

function CompilationDemo() {
  const [progress, setProgress] = useState(0);
  const [currentStep, setCurrentStep] = useState(0);
  const [isComplete, setIsComplete] = useState(false);
  const startTimeRef = useRef(Date.now());

  useEffect(() => {
    const pauseAfterComplete = 2500;
    const cycleDuration = totalCompilationDuration + pauseAfterComplete;

    const interval = setInterval(() => {
      const elapsed = (Date.now() - startTimeRef.current) % cycleDuration;

      if (elapsed < totalCompilationDuration) {
        // Compiling phase
        setIsComplete(false);
        const progressPercent = (elapsed / totalCompilationDuration) * 100;
        setProgress(progressPercent);

        // Determine current step
        let accumulated = 0;
        for (let i = 0; i < compilationSteps.length; i++) {
          accumulated += compilationSteps[i].duration;
          if (elapsed < accumulated) {
            setCurrentStep(i);
            break;
          }
        }
      } else {
        // Complete phase
        setProgress(100);
        setIsComplete(true);
        setCurrentStep(compilationSteps.length);
      }
    }, 50);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="relative">
      <div className="absolute inset-0 bg-gradient-to-r from-violet-600 to-blue-600 rounded-2xl blur-2xl opacity-30" />
      <div className="relative bg-gray-900/80 backdrop-blur rounded-2xl p-6 border border-gray-700 min-w-[320px]">
        {/* Header */}
        <div className="flex items-center gap-3 mb-4">
          <div className={`p-2 rounded-lg ${isComplete ? 'bg-green-500/20' : 'bg-violet-500/20'}`}>
            {isComplete ? (
              <Check className="w-5 h-5 text-green-400" />
            ) : (
              <FileCode className="w-5 h-5 text-violet-400" />
            )}
          </div>
          <div>
            <div className="text-sm font-medium text-white">
              {isComplete ? 'Replay Compiled!' : 'Compiling Replay'}
            </div>
            <div className="text-xs text-gray-500">match_2024_finals.replay</div>
          </div>
        </div>

        {/* Progress bar */}
        <div className="relative h-2 bg-gray-800 rounded-full overflow-hidden mb-3">
          <div
            className={`absolute inset-y-0 left-0 rounded-full transition-all duration-100 ${
              isComplete
                ? 'bg-green-500'
                : 'bg-gradient-to-r from-violet-500 to-blue-500'
            }`}
            style={{ width: `${progress}%` }}
          />
          {!isComplete && (
            <div
              className="absolute inset-y-0 w-20 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-shimmer"
              style={{ left: `${progress - 10}%` }}
            />
          )}
        </div>

        {/* Status */}
        <div className="flex items-center justify-between text-xs">
          <div className="flex items-center gap-2">
            {isComplete ? (
              <span className="text-green-400 font-medium">✓ Ready to play</span>
            ) : (
              <>
                <Loader2 className="w-3 h-3 text-violet-400 animate-spin" />
                <span className="text-gray-400">
                  {compilationSteps[currentStep]?.label || 'Processing...'}
                </span>
              </>
            )}
          </div>
          <span className={isComplete ? 'text-green-400' : 'text-gray-500'}>
            {Math.round(progress)}%
          </span>
        </div>

        {/* Stats after completion */}
        {isComplete && (
          <div className="mt-4 pt-4 border-t border-gray-700/50 grid grid-cols-2 gap-3 text-xs">
            <div>
              <div className="text-gray-500">Original</div>
              <div className="text-gray-300 font-mono">14.2 MB</div>
            </div>
            <div>
              <div className="text-gray-500">Compiled</div>
              <div className="text-green-400 font-mono">0.78 MB</div>
            </div>
            <div>
              <div className="text-gray-500">Compression</div>
              <div className="text-cyan-400 font-mono">94.5%</div>
            </div>
            <div>
              <div className="text-gray-500">Time</div>
              <div className="text-gray-300 font-mono">2.6s</div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// Quadratic Bezier evaluation
function quadBezier(t: number, p0: number, p1: number, p2: number): number {
  const mt = 1 - t;
  return mt * mt * p0 + 2 * mt * t * p1 + t * t * p2;
}

// Animated clip editor visualization with camera tracking ball
function ClipEditorAnimation() {
  const [time, setTime] = useState(0);

  useEffect(() => {
    let animationId: number;
    const startTime = Date.now();

    const animate = () => {
      const elapsed = (Date.now() - startTime) / 1000;
      setTime(elapsed);
      animationId = requestAnimationFrame(animate);
    };

    animationId = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(animationId);
  }, []);

  // Ball position: straight line back and forth (6s cycle)
  const ballCycle = 6;
  const ballT = (time % ballCycle) / ballCycle;
  const ballProgress = ballT < 0.5 ? ballT * 2 : 2 - ballT * 2; // 0->1->0
  const ballX = 60 + (260 - 60) * ballProgress;
  const ballY = 130 + (50 - 130) * ballProgress;

  // Camera position: follows the actual Bezier path (6s cycle)
  // Path: M 40 140 Q 100 60 180 80 Q 260 100 280 100
  // Two quadratic Bezier segments
  const camCycle = 6;
  const camT = (time % camCycle) / camCycle;

  let camX: number, camY: number;
  if (camT < 0.5) {
    // First segment: (40,140) -> (100,60) -> (180,80)
    const t = camT * 2;
    camX = quadBezier(t, 40, 100, 180);
    camY = quadBezier(t, 140, 60, 80);
  } else {
    // Second segment: (180,80) -> (260,100) -> (280,100)
    const t = (camT - 0.5) * 2;
    camX = quadBezier(t, 180, 260, 280);
    camY = quadBezier(t, 80, 100, 100);
  }

  // Calculate angle from camera to ball
  const dx = ballX - camX;
  const dy = ballY - camY;
  const angle = Math.atan2(dy, dx) * (180 / Math.PI);

  // Car positions (closed loops)
  const carPositions = [
    // Blue cars
    { path: [[50,100], [120,80], [150,110], [80,120]], dur: 4, color: 'rgb(59, 130, 246)' },
    { path: [[70,130], [110,110], [140,125], [100,140]], dur: 5, color: 'rgb(59, 130, 246)' },
    // Orange cars
    { path: [[250,70], [200,55], [180,80], [220,95]], dur: 4.5, color: 'rgb(249, 115, 22)' },
    { path: [[240,110], [200,95], [190,115], [225,130]], dur: 5.5, color: 'rgb(249, 115, 22)' },
  ].map(car => {
    const t = (time % car.dur) / car.dur;
    const totalSegments = car.path.length;
    const segmentIndex = Math.floor(t * totalSegments) % totalSegments;
    const segmentT = (t * totalSegments) % 1;
    const p1 = car.path[segmentIndex];
    const p2 = car.path[(segmentIndex + 1) % totalSegments];
    return {
      x: p1[0] + (p2[0] - p1[0]) * segmentT,
      y: p1[1] + (p2[1] - p1[1]) * segmentT,
      color: car.color,
    };
  });

  return (
    <svg className="absolute inset-0 w-full h-full" style={{ overflow: 'visible' }}>
      <defs>
        <linearGradient id="pathGradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="rgb(59, 130, 246)" stopOpacity="0.3" />
          <stop offset="50%" stopColor="rgb(139, 92, 246)" stopOpacity="0.6" />
          <stop offset="100%" stopColor="rgb(59, 130, 246)" stopOpacity="0.3" />
        </linearGradient>
        <linearGradient id="cameraViewGradient" x1="0%" y1="50%" x2="100%" y2="50%">
          <stop offset="0%" stopColor="rgb(59, 130, 246)" stopOpacity="0.35" />
          <stop offset="100%" stopColor="rgb(59, 130, 246)" stopOpacity="0" />
        </linearGradient>
      </defs>

      {/* Camera path background (dashed line) */}
      <path
        d="M 40 140 Q 100 60 180 80 Q 260 100 280 100"
        stroke="url(#pathGradient)"
        strokeWidth="2"
        strokeDasharray="4 4"
        fill="none"
      />
      {/* Keyframe dots on camera path */}
      <circle cx="40" cy="140" r="4" fill="rgb(59, 130, 246)" opacity="0.6" />
      <circle cx="140" cy="70" r="4" fill="rgb(139, 92, 246)" opacity="0.6" />
      <circle cx="280" cy="100" r="4" fill="rgb(59, 130, 246)" opacity="0.6" />

      {/* Cars */}
      {carPositions.map((car, i) => (
        <g key={i}>
          <circle cx={car.x} cy={car.y} r="6" fill={car.color} />
          <circle cx={car.x} cy={car.y} r="9" fill={car.color} opacity="0.3" />
        </g>
      ))}

      {/* Ball */}
      <circle cx={ballX} cy={ballY} r="7" fill="white" opacity="0.95" />
      <circle cx={ballX} cy={ballY} r="10" fill="white" opacity="0.3" />

      {/* Camera with view cone oriented toward ball */}
      <g transform={`translate(${camX}, ${camY}) rotate(${angle})`}>
        {/* View cone triangle pointing toward ball */}
        <polygon
          points="12,0 70,-30 70,30"
          fill="url(#cameraViewGradient)"
        />
        {/* Camera glow */}
        <circle r="12" fill="rgb(59, 130, 246)" opacity="0.2" />
        <circle r="8" fill="rgb(59, 130, 246)" opacity="0.9" />
        {/* Camera icon */}
        <rect x="-4" y="-3" width="8" height="6" rx="1" fill="white" />
        <circle r="2" fill="rgb(59, 130, 246)" />
      </g>
    </svg>
  );
}

export default function Home() {
  return (
    <div className="relative space-y-24">
      <SEOHead
        title="BallCam - Watch Rocket League Replays in 3D"
        description="Experience Rocket League replays from any angle with BallCam's immersive 3D viewer. Upload your replays and analyze them with your team."
      />
      <StructuredData data={[createWebSiteStructuredData(), createOrganizationStructuredData()]} />
      {/* Animated background effects */}
      <div className="fixed inset-0 -z-10 overflow-hidden pointer-events-none">
        {/* Base gradient */}
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-violet-900/30 via-gray-950 to-gray-950" />
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_bottom_right,_var(--tw-gradient-stops))] from-blue-900/20 via-transparent to-transparent" />

        {/* Animated floating orbs */}
        <div className="absolute top-[10%] left-[15%] w-[500px] h-[500px] bg-violet-600/15 rounded-full blur-[120px] animate-pulse" />
        <div className="absolute top-[40%] right-[10%] w-[400px] h-[400px] bg-blue-600/15 rounded-full blur-[100px] animate-pulse" style={{ animationDelay: '1s', animationDuration: '3s' }} />
        <div className="absolute bottom-[20%] left-[30%] w-[350px] h-[350px] bg-cyan-600/10 rounded-full blur-[80px] animate-pulse" style={{ animationDelay: '0.5s', animationDuration: '4s' }} />
        <div className="absolute top-[60%] right-[35%] w-[300px] h-[300px] bg-violet-500/10 rounded-full blur-[100px] animate-pulse" style={{ animationDelay: '2s', animationDuration: '5s' }} />

        {/* Subtle grid pattern overlay */}
        <div className="absolute inset-0 bg-[linear-gradient(rgba(255,255,255,0.02)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.02)_1px,transparent_1px)] bg-[size:64px_64px]" />
      </div>

      {/* Hero Section */}
      <section className="grid lg:grid-cols-2 gap-12 items-center py-8">
        <div className="text-center lg:text-left space-y-6">
          <div className="flex flex-wrap gap-2 justify-center lg:justify-start">
            <span className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-cyan-400 text-sm">
              <Users className="w-4 h-4" />
              Collaborative Viewing
            </span>
          </div>

          <h1>
            <Logo size="xl" />
          </h1>

          <p className="text-2xl text-gray-300 font-light">
            Watch Rocket League <span className="text-cyan-400">Replays</span> in <span className="text-violet-400">3D</span>
          </p>

          <p className="text-lg text-gray-400 max-w-xl mx-auto lg:mx-0">
            Experience Rocket League replays from any angle with BallCam's immersive 3D viewer.
            Upload, analyze, and share the experience with your team.
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center lg:justify-start pt-4">
            <Link to="/upload">
              <GradientButton size="lg" className="w-full sm:w-auto">
                <Upload className="w-5 h-5" />
                Upload a Replay
                <ArrowRight className="w-4 h-4 ml-1" />
              </GradientButton>
            </Link>
            <Link to="/replays">
              <GradientButton size="lg" variant="outline" className="w-full sm:w-auto">
                <Eye className="w-5 h-5" />
                Browse Replays
              </GradientButton>
            </Link>
          </div>

          {/* Discord Join Widget */}
          <DiscordWidget variant="hero" className="pt-2" />
        </div>

        <div className="hidden lg:block h-[550px] -mr-8 -my-12 overflow-visible">
          <OctaneHero className="w-full h-full scale-110 origin-center" />
        </div>
      </section>

      {/* Global Stats Banner */}
      <GlobalStats />

      {/* Latest News (only renders if there are published announcements) */}
      <LatestNews />

      {/* ═══════════════════════════════════════════════════════════════ */}
      {/* REPLAY ANALYSIS SECTION */}
      {/* ═══════════════════════════════════════════════════════════════ */}
      <div className="relative space-y-8">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-3">
            <div className="p-3 bg-cyan-500/20 rounded-xl">
              <Eye className="w-8 h-8 text-cyan-400" />
            </div>
            <h2 className="text-3xl lg:text-4xl font-bold text-white">Replay Analysis</h2>
          </div>
          <div className="flex-1 h-px bg-gradient-to-r from-cyan-500/30 to-transparent" />
        </div>

        {/* Watch Together - Main Feature Section */}
        <section className="relative">
        <div className="absolute inset-0 bg-gradient-to-r from-cyan-600/5 via-violet-600/5 to-blue-600/5 rounded-3xl" />
        <div className="relative rounded-3xl border border-cyan-500/20 p-8 lg:p-12">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            {/* Left: Content */}
            <div className="space-y-6">
              <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-cyan-400 text-sm">
                <Users className="w-4 h-4" />
                Replay Analysis
              </div>

              <h2 className="text-4xl font-bold">
                <span className="text-white">Watch Replays </span>
                <span className="bg-gradient-to-r from-cyan-400 to-blue-400 bg-clip-text text-transparent">Together</span>
              </h2>

              <p className="text-lg text-gray-400">
                Analyze your replays as a team. Create a session, share the link,
                and watch synchronized playback with up to 30 teammates.
                Everyone can chat and discuss plays in real-time.
              </p>

              <ul className="space-y-3">
                {[
                  'Synchronized playback across all viewers',
                  'Real-time chat to discuss plays',
                  'Each viewer controls their own camera',
                  'No account required to join',
                ].map((item) => (
                  <li key={item} className="flex items-center gap-3">
                    <CheckCircle2 className="w-5 h-5 text-cyan-400 flex-shrink-0" />
                    <span className="text-gray-300">{item}</span>
                  </li>
                ))}
              </ul>

              <div className="pt-4">
                <Link to="/upload">
                  <GradientButton size="lg">
                    <UserPlus className="w-5 h-5" />
                    Start a Session
                    <ArrowRight className="w-4 h-4 ml-1" />
                  </GradientButton>
                </Link>
              </div>
            </div>

            {/* Right: Visual - Mock Viewer Interface */}
            <div className="relative">
              {/* Glow effect */}
              <div className="absolute inset-0 bg-gradient-to-r from-cyan-600/20 to-blue-600/20 rounded-2xl blur-3xl" />

              {/* Browser-style window */}
              <div className="relative bg-gray-900 rounded-xl border border-gray-700 shadow-2xl shadow-cyan-500/10 overflow-hidden">
                {/* Title bar */}
                <div className="flex items-center justify-between px-4 py-2.5 bg-gray-800/80 border-b border-gray-700">
                  <div className="flex items-center gap-2">
                    <div className="flex gap-1.5">
                      <div className="w-3 h-3 rounded-full bg-red-500" />
                      <div className="w-3 h-3 rounded-full bg-yellow-500" />
                      <div className="w-3 h-3 rounded-full bg-green-500" />
                    </div>
                    <div className="ml-3 px-3 py-1 bg-gray-900/50 rounded text-xs text-gray-400 flex items-center gap-2">
                      <span className="text-cyan-400">⬢</span>
                      ballcam.tv/watch/abc123
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="flex items-center gap-1 text-xs text-gray-400">
                      <Users className="w-3.5 h-3.5" />
                      <span>4</span>
                    </div>
                  </div>
                </div>

                {/* Main content area */}
                <div className="flex">
                  {/* Viewer area */}
                  <div className="flex-1 relative h-[280px] bg-gradient-to-br from-gray-800 to-gray-900">
                    {/* Stylized field */}
                    <div className="absolute inset-4 rounded-lg bg-gradient-to-br from-violet-900/40 to-gray-900/60 border border-violet-500/20 overflow-hidden">
                      {/* Center circle */}
                      <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-16 h-16 rounded-full border border-white/10" />
                      {/* Center line */}
                      <div className="absolute left-1/2 top-0 bottom-0 w-px bg-white/10" />
                      {/* Goal areas */}
                      <div className="absolute left-0 top-1/2 -translate-y-1/2 w-8 h-20 border-r border-t border-b border-blue-500/20 bg-blue-500/5" />
                      <div className="absolute right-0 top-1/2 -translate-y-1/2 w-8 h-20 border-l border-t border-b border-orange-500/20 bg-orange-500/5" />

                      {/* Players - Blue team */}
                      <div className="absolute left-[15%] top-[30%] w-3 h-3 rounded-full bg-blue-500 shadow-lg shadow-blue-500/50" />
                      <div className="absolute left-[20%] top-[55%] w-3 h-3 rounded-full bg-blue-500 shadow-lg shadow-blue-500/50" />
                      <div className="absolute left-[25%] top-[70%] w-3 h-3 rounded-full bg-blue-500 shadow-lg shadow-blue-500/50" />

                      {/* Players - Orange team */}
                      <div className="absolute left-[75%] top-[25%] w-3 h-3 rounded-full bg-orange-500 shadow-lg shadow-orange-500/50" />
                      <div className="absolute left-[80%] top-[50%] w-3 h-3 rounded-full bg-orange-500 shadow-lg shadow-orange-500/50" />
                      <div className="absolute left-[70%] top-[75%] w-3 h-3 rounded-full bg-orange-500 shadow-lg shadow-orange-500/50" />

                      {/* Ball */}
                      <div className="absolute left-[55%] top-[40%] w-4 h-4 rounded-full bg-white shadow-lg shadow-white/50" />

                      {/* Ping marker with ripple */}
                      <div className="absolute left-[35%] top-[55%]">
                        <div className="w-6 h-6 rounded-full bg-cyan-500/30 animate-ping" />
                        <div className="absolute inset-0 flex items-center justify-center">
                          <div className="w-3 h-3 rounded-full bg-cyan-400 border-2 border-white" />
                        </div>
                      </div>

                      {/* Drawing line */}
                      <svg className="absolute inset-0 w-full h-full" style={{ overflow: 'visible' }}>
                        <path
                          d="M 80 120 Q 120 80 160 100 T 220 90"
                          stroke="rgb(249, 115, 22)"
                          strokeWidth="2"
                          strokeDasharray="4 2"
                          fill="none"
                          className="opacity-60"
                        />
                        <circle cx="220" cy="90" r="4" fill="rgb(249, 115, 22)" />
                      </svg>
                    </div>

                    {/* Cursors */}
                    <div className="absolute left-[30%] top-[45%] transition-all duration-1000" style={{ animation: 'float 4s ease-in-out infinite' }}>
                      <MousePointer2 className="w-4 h-4 text-cyan-400 -rotate-12 drop-shadow-lg" />
                      <span className="absolute left-4 -top-1 text-[10px] text-cyan-400 font-medium bg-gray-900/80 px-1.5 py-0.5 rounded">Sarah</span>
                    </div>
                    <div className="absolute left-[55%] top-[60%] transition-all duration-1000" style={{ animation: 'float 5s ease-in-out infinite 1s' }}>
                      <MousePointer2 className="w-4 h-4 text-violet-400 rotate-12 drop-shadow-lg" />
                      <span className="absolute left-4 -top-1 text-[10px] text-violet-400 font-medium bg-gray-900/80 px-1.5 py-0.5 rounded">Mike</span>
                    </div>

                    {/* Playback controls */}
                    <div className="absolute bottom-4 left-4 right-4 flex items-center gap-3 px-3 py-2 bg-gray-900/90 backdrop-blur rounded-lg border border-gray-700">
                      <PlayCircle className="w-5 h-5 text-white" />
                      <div className="flex-1 h-1 bg-gray-700 rounded-full overflow-hidden">
                        <div className="w-[65%] h-full bg-gradient-to-r from-cyan-500 to-blue-500 rounded-full relative">
                          <div className="absolute right-0 top-1/2 -translate-y-1/2 w-3 h-3 bg-white rounded-full shadow-lg" />
                        </div>
                      </div>
                      <span className="text-xs text-gray-400 font-mono">2:34 / 5:00</span>
                    </div>
                  </div>

                  {/* Chat sidebar */}
                  <div className="w-48 border-l border-gray-700 bg-gray-800/50 flex flex-col">
                    <div className="px-3 py-2 border-b border-gray-700 text-xs font-medium text-gray-400 flex items-center gap-2">
                      <MessageCircle className="w-3.5 h-3.5" />
                      Session Chat
                    </div>
                    <div className="flex-1 p-2 space-y-2 text-xs overflow-hidden">
                      <div className="animate-fade-in" style={{ animationDelay: '0s' }}>
                        <span className="text-cyan-400 font-medium">Sarah:</span>
                        <span className="text-gray-300 ml-1">Look at this rotation!</span>
                      </div>
                      <div className="animate-fade-in" style={{ animationDelay: '0.5s' }}>
                        <span className="text-violet-400 font-medium">Mike:</span>
                        <span className="text-gray-300 ml-1">Great pass 🔥</span>
                      </div>
                      <div className="animate-fade-in" style={{ animationDelay: '1s' }}>
                        <span className="text-blue-400 font-medium">Jake:</span>
                        <span className="text-gray-300 ml-1">2:34 nice save</span>
                      </div>
                      <div className="animate-fade-in" style={{ animationDelay: '1.5s' }}>
                        <span className="text-green-400 font-medium">Emma:</span>
                        <span className="text-gray-300 ml-1">I see the mistake</span>
                      </div>
                    </div>
                    <div className="p-2 border-t border-gray-700">
                      <div className="flex items-center gap-2 px-2 py-1.5 bg-gray-900/50 rounded text-gray-500 text-xs">
                        <span>Type a message...</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Floating connected viewers indicator */}
              <div className="absolute -bottom-6 left-1/2 -translate-x-1/2 flex items-center gap-2 px-4 py-2 bg-gray-800/90 backdrop-blur rounded-full border border-gray-700 shadow-lg">
                <div className="flex -space-x-2">
                  <div className="w-6 h-6 rounded-full bg-gradient-to-br from-cyan-400 to-cyan-600 flex items-center justify-center text-white text-xs font-bold border-2 border-gray-800">S</div>
                  <div className="w-6 h-6 rounded-full bg-gradient-to-br from-violet-400 to-violet-600 flex items-center justify-center text-white text-xs font-bold border-2 border-gray-800">M</div>
                  <div className="w-6 h-6 rounded-full bg-gradient-to-br from-blue-400 to-blue-600 flex items-center justify-center text-white text-xs font-bold border-2 border-gray-800">J</div>
                  <div className="w-6 h-6 rounded-full bg-gradient-to-br from-green-400 to-green-600 flex items-center justify-center text-white text-xs font-bold border-2 border-gray-800">E</div>
                </div>
                <span className="text-xs text-gray-400">All synced at <span className="text-cyan-400 font-mono">2:34</span></span>
              </div>
            </div>
          </div>
        </div>
        </section>
      </div>

      {/* Collab Features Grid */}
      <section>
        <div className="text-center mb-12">
          <h2 className="text-3xl font-bold mb-4">
            <Share2 className="inline-block w-8 h-8 mr-3 text-cyan-400" />
            Collaborative Features
          </h2>
          <p className="text-gray-400 max-w-2xl mx-auto">
            Everything you need to review replays as a team
          </p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
          {collabFeatures.map((feature) => (
            <GradientCard key={feature.title} hover>
              <div className="text-center">
                <div className="w-12 h-12 mx-auto rounded-xl bg-gradient-to-br from-cyan-600/20 to-blue-600/20 flex items-center justify-center mb-4">
                  <feature.icon className="w-6 h-6 text-cyan-400" />
                </div>
                <h3 className="font-semibold mb-2">{feature.title}</h3>
                <p className="text-gray-400 text-sm">{feature.description}</p>
              </div>
            </GradientCard>
          ))}
        </div>
      </section>

      {/* How It Works */}
      <section>
        <div className="text-center mb-12">
          <h2 className="text-3xl font-bold mb-4">How It Works</h2>
          <p className="text-gray-400 max-w-2xl mx-auto">
            From replay file to team analysis in seconds
          </p>
        </div>

        <div className="grid md:grid-cols-3 gap-8">
          {steps.map((step, index) => (
            <div key={step.number} className="relative">
              {index < steps.length - 1 && (
                <div className="hidden md:block absolute top-12 left-[60%] w-[80%] h-[2px] bg-gradient-to-r from-violet-500/50 to-transparent" />
              )}
              <div className="flex flex-col items-center text-center">
                <div className="relative">
                  <div className={`w-24 h-24 rounded-2xl ${index === 2 ? 'bg-gradient-to-br from-cyan-600/20 to-blue-600/20 border-cyan-500/30' : 'bg-gradient-to-br from-violet-600/20 to-blue-600/20 border-violet-500/30'} border flex items-center justify-center mb-4`}>
                    <step.icon className={`w-10 h-10 ${index === 2 ? 'text-cyan-400' : 'text-violet-400'}`} />
                  </div>
                  <span className={`absolute -top-2 -right-2 text-xs font-bold ${index === 2 ? 'text-cyan-400' : 'text-violet-400'} bg-gray-900 px-2 py-1 rounded-full border ${index === 2 ? 'border-cyan-500/30' : 'border-violet-500/30'}`}>
                    {step.number}
                  </span>
                </div>
                <h3 className="text-xl font-semibold mb-2">{step.title}</h3>
                <p className="text-gray-400">{step.description}</p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* Replay Library Section */}
      <section>
        <div className="text-center mb-8">
          <p className="text-gray-500 text-sm uppercase tracking-wider mb-2">
            Also on BallCam
          </p>
          <h2 className="text-3xl font-bold mb-4">
            <Eye className="inline-block w-8 h-8 mr-3 text-violet-400" />
            Replay Library
          </h2>
          <p className="text-gray-400 max-w-2xl mx-auto">
            Browse and analyze recorded matches from the community
          </p>
        </div>
        <HotReplays />
      </section>

      {/* Stats Showcase */}
      <StatsShowcase />

      {/* Clips Section */}
      <section className="relative">
        <div className="absolute inset-0 bg-gradient-to-r from-blue-600/5 via-violet-600/5 to-cyan-600/5 rounded-3xl" />
        <div className="relative rounded-3xl border border-blue-500/20 p-8 lg:p-12">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            {/* Left: Content */}
            <div className="space-y-6">
              <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-amber-500/10 border border-amber-500/20 text-amber-400 text-sm">
                <Clapperboard className="w-4 h-4" />
                Alpha
              </div>

              <h2 className="text-4xl font-bold">
                <span className="text-white">Create & Share </span>
                <span className="bg-gradient-to-r from-blue-400 to-violet-400 bg-clip-text text-transparent">Clips</span>
              </h2>

              <p className="text-lg text-gray-400">
                Capture your best moments and share them with the community.
                Record camera movements, set start and end points, and create
                cinematic clips from any replay.
              </p>

              <ul className="space-y-3">
                {[
                  'Record your camera movements in real-time',
                  'Create cinematic clips with keyframe editing',
                  'Share clips with a single link',
                  'Browse community clips for inspiration',
                ].map((item) => (
                  <li key={item} className="flex items-center gap-3">
                    <CheckCircle2 className="w-5 h-5 text-blue-400 flex-shrink-0" />
                    <span className="text-gray-300">{item}</span>
                  </li>
                ))}
              </ul>

              <div className="flex flex-wrap gap-4 pt-4">
                <Link to="/clips">
                  <GradientButton size="lg">
                    <Clapperboard className="w-5 h-5" />
                    Browse Clips
                    <ArrowRight className="w-4 h-4 ml-1" />
                  </GradientButton>
                </Link>
                <Link to="/replays">
                  <GradientButton size="lg" variant="outline">
                    <Video className="w-5 h-5" />
                    Create a Clip
                  </GradientButton>
                </Link>
              </div>
            </div>

            {/* Right: Visual - Mock Clip Interface */}
            <div className="relative">
              {/* Glow effect */}
              <div className="absolute inset-0 bg-gradient-to-r from-blue-600/20 to-violet-600/20 rounded-2xl blur-3xl" />

              {/* Browser-style window */}
              <div className="relative bg-gray-900 rounded-xl border border-gray-700 shadow-2xl shadow-blue-500/10 overflow-hidden">
                {/* Title bar */}
                <div className="flex items-center justify-between px-4 py-2.5 bg-gray-800/80 border-b border-gray-700">
                  <div className="flex items-center gap-2">
                    <div className="flex gap-1.5">
                      <div className="w-3 h-3 rounded-full bg-red-500" />
                      <div className="w-3 h-3 rounded-full bg-yellow-500" />
                      <div className="w-3 h-3 rounded-full bg-green-500" />
                    </div>
                    <div className="ml-3 px-3 py-1 bg-gray-900/50 rounded text-xs text-gray-400 flex items-center gap-2">
                      <Clapperboard className="w-3 h-3 text-blue-400" />
                      Clip Editor
                    </div>
                  </div>
                  <div className="flex items-center gap-2 text-xs text-gray-400">
                    <span className="px-2 py-0.5 rounded bg-blue-500/20 text-blue-400">Capture Mode</span>
                  </div>
                </div>

                {/* Main content */}
                <div className="aspect-video relative bg-gradient-to-br from-gray-800 to-gray-900">
                  {/* Stylized 3D view */}
                  <div className="absolute inset-4 rounded-lg bg-gradient-to-br from-violet-900/40 to-gray-900/60 border border-violet-500/20 overflow-hidden">
                    {/* Field markings */}
                    <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-16 h-16 rounded-full border border-white/10" />
                    <div className="absolute left-1/2 top-0 bottom-0 w-px bg-white/10" />

                    {/* Animated clip editor visualization */}
                    <ClipEditorAnimation />

                    {/* Recording indicator */}
                    <div className="absolute top-3 left-3 flex items-center gap-2 px-2 py-1 bg-red-500/20 border border-red-500/30 rounded-full">
                      <div className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
                      <span className="text-[10px] text-red-400 font-medium">REC</span>
                    </div>
                  </div>

                  {/* Timeline at bottom */}
                  <div className="absolute bottom-3 left-3 right-3 bg-gray-900/90 backdrop-blur rounded-lg p-2 border border-gray-700">
                    <div className="flex items-center gap-2 mb-2">
                      <PlayCircle className="w-4 h-4 text-white" />
                      <div className="flex-1 h-1.5 bg-gray-700 rounded-full overflow-hidden relative">
                        <div className="absolute inset-y-0 left-0 bg-gradient-to-r from-blue-500 to-violet-500 rounded-full animate-playback-progress" />
                      </div>
                      <span className="text-[10px] text-gray-400 font-mono">0:12 / 0:28</span>
                    </div>
                    {/* Keyframes visualization */}
                    <div className="h-4 bg-gray-800 rounded relative">
                      <div className="absolute left-[10%] top-1/2 -translate-y-1/2 w-2 h-2 rounded-full bg-blue-500" />
                      <div className="absolute left-[35%] top-1/2 -translate-y-1/2 w-2 h-2 rounded-full bg-blue-500" />
                      <div className="absolute left-[70%] top-1/2 -translate-y-1/2 w-2 h-2 rounded-full bg-blue-500" />
                      <div className="absolute left-[10%] right-[30%] top-1/2 -translate-y-1/2 h-0.5 bg-blue-500/50" />
                    </div>
                  </div>

                  {/* CSS for playback animation */}
                  <style>{`
                    @keyframes playback-progress {
                      0% { width: 0%; }
                      100% { width: 100%; }
                    }
                    .animate-playback-progress {
                      animation: playback-progress 6s linear infinite;
                    }
                  `}</style>
                </div>
              </div>

              {/* Stats badge */}
              <div className="absolute -bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-3 px-4 py-2 bg-gray-800/90 backdrop-blur rounded-full border border-gray-700 shadow-lg">
                <div className="flex items-center gap-1.5 text-xs">
                  <Eye className="w-3.5 h-3.5 text-gray-400" />
                  <span className="text-gray-300">1.2k views</span>
                </div>
                <div className="w-px h-4 bg-gray-700" />
                <div className="flex items-center gap-1.5 text-xs">
                  <Heart className="w-3.5 h-3.5 text-red-400" />
                  <span className="text-gray-300">89 likes</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Camera Modes */}
      <section>
        <div className="text-center mb-12">
          <h2 className="text-3xl font-bold mb-4">
            <Camera className="inline-block w-8 h-8 mr-3 text-violet-400" />
            Camera Modes
          </h2>
          <p className="text-gray-400 max-w-2xl mx-auto">
            Multiple ways to experience your replays
          </p>
        </div>

        <div className="grid md:grid-cols-3 gap-6">
          {cameraModes.map((mode) => (
            <GlowCard key={mode.name}>
              <div className="flex flex-col h-full">
                <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-violet-600 to-blue-600 flex items-center justify-center mb-4">
                  <mode.icon className="w-6 h-6 text-white" />
                </div>
                <h3 className="text-lg font-semibold mb-2">{mode.name}</h3>
                <p className="text-gray-400 text-sm flex-grow">{mode.description}</p>
                <div className="mt-4 pt-4 border-t border-gray-800">
                  <span className="text-xs text-violet-400 font-mono bg-violet-500/10 px-2 py-1 rounded">
                    {mode.keys}
                  </span>
                </div>
              </div>
            </GlowCard>
          ))}
        </div>
      </section>

      {/* Features Grid */}
      <section>
        <div className="text-center mb-12">
          <h2 className="text-3xl font-bold mb-4">Packed with Features</h2>
          <p className="text-gray-400 max-w-2xl mx-auto">
            Everything you need to analyze and enjoy your Rocket League replays
          </p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature) => (
            <GradientCard key={feature.title} hover>
              <div className="flex gap-4">
                <div className="flex-shrink-0">
                  <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-violet-600/20 to-blue-600/20 flex items-center justify-center">
                    <feature.icon className="w-5 h-5 text-violet-400" />
                  </div>
                </div>
                <div>
                  <h3 className="font-semibold mb-1">{feature.title}</h3>
                  <p className="text-gray-400 text-sm">{feature.description}</p>
                </div>
              </div>
            </GradientCard>
          ))}
        </div>
      </section>

      {/* Tech Highlights */}
      <section className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-violet-900/20 to-blue-900/20 border border-violet-500/20 p-8 lg:p-12">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,_var(--tw-gradient-stops))] from-violet-600/10 via-transparent to-transparent" />

        <div className="relative grid lg:grid-cols-2 gap-8 items-center">
          <div>
            <h2 className="text-3xl font-bold mb-6">Built for Performance</h2>
            <ul className="space-y-4">
              {[
                'Binary format with 94.5% compression ratio',
                'Smooth 60 FPS playback on modern browsers',
                'WebGL-powered 3D rendering with Three.js',
                'Real-time sync with WebSocket technology',
                'Client-side processing for privacy'
              ].map((item) => (
                <li key={item} className="flex items-start gap-3">
                  <CheckCircle2 className="w-5 h-5 text-green-400 flex-shrink-0 mt-0.5" />
                  <span className="text-gray-300">{item}</span>
                </li>
              ))}
            </ul>
          </div>

          <div className="flex justify-center">
            <CompilationDemo />
          </div>
        </div>
      </section>

      {/* Feedback Hub Section */}
      <section className="relative">
        <div className="absolute inset-0 bg-gradient-to-r from-violet-600/5 via-transparent to-violet-600/5 rounded-3xl" />
        <div className="relative rounded-3xl border border-violet-500/20 p-8 lg:p-12">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div className="space-y-6">
              <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-violet-500/10 border border-violet-500/20 text-violet-400 text-sm">
                <MessageSquare className="w-4 h-4" />
                Community
              </div>

              <h2 className="text-4xl font-bold">
                <span className="text-white">Shape the </span>
                <span className="bg-gradient-to-r from-violet-400 to-blue-400 bg-clip-text text-transparent">Future</span>
              </h2>

              <p className="text-lg text-gray-400">
                Your feedback drives our development. Share ideas, report bugs,
                and vote on features you want to see in RLView.
              </p>

              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 rounded-xl bg-gray-800/50 border border-gray-700/50">
                  <Lightbulb className="w-8 h-8 text-blue-400 mb-2" />
                  <div className="font-medium text-white">Feature Requests</div>
                  <div className="text-sm text-gray-500">Suggest new features</div>
                </div>
                <div className="p-4 rounded-xl bg-gray-800/50 border border-gray-700/50">
                  <Bug className="w-8 h-8 text-red-400 mb-2" />
                  <div className="font-medium text-white">Bug Reports</div>
                  <div className="text-sm text-gray-500">Help us fix issues</div>
                </div>
              </div>

              <div className="pt-4">
                <Link to="/feedback">
                  <GradientButton size="lg">
                    <MessageSquare className="w-5 h-5" />
                    Visit Feedback Hub
                    <ArrowRight className="w-4 h-4 ml-1" />
                  </GradientButton>
                </Link>
              </div>
            </div>

            {/* Right: Mock Feedback Card */}
            <div className="relative">
              <div className="absolute inset-0 bg-gradient-to-r from-violet-600 to-blue-600 rounded-2xl blur-3xl opacity-20" />
              <div className="relative bg-gray-900/80 backdrop-blur rounded-2xl border border-gray-700 overflow-hidden">
                <div className="p-5 space-y-4">
                  {/* Mock feedback item */}
                  <div className="flex gap-4">
                    <div className="flex flex-col items-center">
                      <div className="p-2 rounded-lg bg-violet-600/30 text-violet-300">
                        <ChevronUp className="w-5 h-5" />
                      </div>
                      <span className="text-lg font-bold text-violet-300 mt-1">42</span>
                    </div>
                    <div className="flex-1">
                      <div className="flex gap-2 mb-2">
                        <span className="px-2 py-0.5 text-xs rounded-full bg-blue-500/20 text-blue-400">Feature</span>
                        <span className="px-2 py-0.5 text-xs rounded-full bg-green-500/20 text-green-400">Planned</span>
                      </div>
                      <h3 className="font-bold text-white mb-1">Add heatmap visualization</h3>
                      <p className="text-sm text-gray-400 line-clamp-2">
                        Would love to see player positioning heatmaps to analyze rotations...
                      </p>
                      <div className="flex items-center gap-4 mt-3 text-sm text-gray-500">
                        <span>by <span className="text-gray-300">RocketFan123</span></span>
                        <span className="flex items-center gap-1">
                          <MessageSquare className="w-3.5 h-3.5" /> 12
                        </span>
                      </div>
                    </div>
                  </div>

                  <div className="border-t border-gray-700/50 pt-4">
                    <div className="flex gap-4">
                      <div className="flex flex-col items-center">
                        <div className="p-2 rounded-lg bg-gray-800/50 text-gray-400">
                          <ChevronUp className="w-5 h-5" />
                        </div>
                        <span className="text-lg font-bold text-gray-300 mt-1">28</span>
                      </div>
                      <div className="flex-1">
                        <div className="flex gap-2 mb-2">
                          <span className="px-2 py-0.5 text-xs rounded-full bg-violet-500/20 text-violet-400">Improvement</span>
                          <span className="px-2 py-0.5 text-xs rounded-full bg-yellow-500/20 text-yellow-400">Under Review</span>
                        </div>
                        <h3 className="font-bold text-white mb-1">Mobile-friendly viewer</h3>
                        <p className="text-sm text-gray-400 line-clamp-2">
                          Touch controls for viewing replays on tablets...
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Roadmap Section */}
      <section className="relative">
        <div className="text-center mb-10">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-violet-500/10 border border-violet-500/20 text-violet-400 text-sm mb-4">
            <Rocket className="w-4 h-4" />
            Coming Soon
          </div>
          <h2 className="text-3xl font-bold mb-4">Roadmap</h2>
          <p className="text-gray-400 max-w-2xl mx-auto">
            Exciting features we're working on to make BallCam even better
          </p>
        </div>

        <div className="max-w-lg mx-auto">
          {/* Mobile Viewer */}
          <div className="group relative">
            <div className="absolute inset-0 bg-gradient-to-br from-green-600/20 to-emerald-600/20 rounded-2xl blur-xl opacity-0 group-hover:opacity-100 transition-opacity duration-500" />
            <div className="relative h-full p-6 rounded-2xl bg-gray-900/50 border border-gray-800 hover:border-green-500/30 transition-colors">
              <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-green-600/20 to-green-500/10 flex items-center justify-center mb-4 border border-green-500/20">
                <Smartphone className="w-6 h-6 text-green-400" />
              </div>
              <h3 className="text-lg font-semibold text-white mb-2">Mobile Viewer</h3>
              <p className="text-gray-400 text-sm mb-4">
                Watch replays on your phone or tablet with an optimized mobile experience.
                Touch controls, simplified UI, and performance optimizations for mobile devices.
              </p>
              <div className="flex flex-wrap gap-2">
                <span className="px-2 py-1 text-xs rounded-full bg-gray-800 text-gray-400 border border-gray-700">Touch controls</span>
                <span className="px-2 py-1 text-xs rounded-full bg-gray-800 text-gray-400 border border-gray-700">Optimized UI</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="text-center py-12">
        <h2 className="text-3xl font-bold mb-4">Ready to Experience Rocket League in 3D?</h2>
        <p className="text-gray-400 mb-8 max-w-xl mx-auto">
          Upload a replay or browse the community library to get started.
        </p>
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <Link to="/upload">
            <GradientButton size="lg">
              <Upload className="w-5 h-5" />
              Upload a Replay
              <ArrowRight className="w-4 h-4 ml-1" />
            </GradientButton>
          </Link>
          <Link to="/replays">
            <GradientButton size="lg" variant="outline">
              <Eye className="w-5 h-5" />
              Browse Replays
            </GradientButton>
          </Link>
        </div>

        <p className="text-gray-500 text-sm mt-6">
          <span className="text-violet-400">Tip:</span> You can also drag and drop .replay files anywhere on this page
        </p>
      </section>
    </div>
  );
}
