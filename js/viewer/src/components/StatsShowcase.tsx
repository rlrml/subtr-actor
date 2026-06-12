/**
 * StatsShowcase - Animated stats showcase for homepage
 * Displays animated statistics to promote the stats feature
 */

import { useState, useEffect, useRef } from 'react';
import { Link } from 'react-router-dom';
import {
  BarChart3,
  Gauge,
  Flame,
  Clock,
  Target,
  TrendingUp,
  ArrowRight,
  Zap,
  Users,
} from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';

// Animated counter hook
function useAnimatedCounter(target: number, duration: number = 2000, decimals: number = 0): number {
  const [value, setValue] = useState(0);
  const startTimeRef = useRef<number | null>(null);
  const animationRef = useRef<number>();

  useEffect(() => {
    startTimeRef.current = null;

    const animate = (timestamp: number) => {
      if (startTimeRef.current === null) {
        startTimeRef.current = timestamp;
      }

      const elapsed = timestamp - startTimeRef.current;
      const progress = Math.min(elapsed / duration, 1);

      // Easing function (ease-out cubic)
      const eased = 1 - Math.pow(1 - progress, 3);
      setValue(eased * target);

      if (progress < 1) {
        animationRef.current = requestAnimationFrame(animate);
      }
    };

    animationRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [target, duration]);

  return Number(value.toFixed(decimals));
}

// Animated possession bar
function AnimatedPossessionBar() {
  const [possession, setPossession] = useState({ blue: 50, orange: 50 });

  useEffect(() => {

    // Cycle through different possession states
    const possessionStates = [
      { blue: 48, orange: 52 },
      { blue: 55, orange: 45 },
      { blue: 42, orange: 58 },
      { blue: 61, orange: 39 },
      { blue: 47, orange: 53 },
      { blue: 53, orange: 47 },
    ];

    let index = 0;
    const interval = setInterval(() => {
      index = (index + 1) % possessionStates.length;
      setPossession(possessionStates[index]);
    }, 3000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="space-y-2">
      <div className="flex justify-between text-sm">
        <span className="text-blue-400 font-mono font-bold">{possession.blue}%</span>
        <span className="text-gray-500 text-xs uppercase tracking-wider">Ball Possession</span>
        <span className="text-orange-400 font-mono font-bold">{possession.orange}%</span>
      </div>

      <div className="relative h-4 rounded-full bg-gray-800 overflow-hidden">
        {/* Blue team side */}
        <div
          className="absolute inset-y-0 left-0 bg-gradient-to-r from-blue-600 to-blue-400 transition-all duration-1000 ease-out"
          style={{ width: `${possession.blue}%` }}
        >
          <div className="absolute inset-0 bg-gradient-to-r from-white/0 via-white/20 to-white/0 animate-shimmer" />
        </div>

        {/* Orange team side */}
        <div
          className="absolute inset-y-0 right-0 bg-gradient-to-l from-orange-600 to-orange-400 transition-all duration-1000 ease-out"
          style={{ width: `${possession.orange}%` }}
        >
          <div className="absolute inset-0 bg-gradient-to-r from-white/0 via-white/20 to-white/0 animate-shimmer" />
        </div>

        {/* Glowing divider */}
        <div
          className="absolute top-0 bottom-0 w-1 bg-white/80 shadow-[0_0_10px_rgba(255,255,255,0.8)] z-10 transition-all duration-1000"
          style={{ left: `${possession.blue}%`, transform: 'translateX(-50%)' }}
        />
      </div>
    </div>
  );
}

// Animated stat card with counting number
function StatCard({
  icon: Icon,
  label,
  value,
  unit,
  color,
  delay = 0,
}: {
  icon: React.ElementType;
  label: string;
  value: number;
  unit: string;
  color: 'blue' | 'orange' | 'cyan' | 'violet' | 'green';
  delay?: number;
}) {
  const [visible, setVisible] = useState(false);
  const animatedValue = useAnimatedCounter(visible ? value : 0, 2000, value % 1 !== 0 ? 1 : 0);

  useEffect(() => {
    const timer = setTimeout(() => setVisible(true), delay);
    return () => clearTimeout(timer);
  }, [delay]);

  const colorClasses = {
    blue: 'from-blue-600/20 to-blue-500/10 border-blue-500/30 text-blue-400',
    orange: 'from-orange-600/20 to-orange-500/10 border-orange-500/30 text-orange-400',
    cyan: 'from-cyan-600/20 to-cyan-500/10 border-cyan-500/30 text-cyan-400',
    violet: 'from-violet-600/20 to-violet-500/10 border-violet-500/30 text-violet-400',
    green: 'from-green-600/20 to-green-500/10 border-green-500/30 text-green-400',
  };

  const textColors = {
    blue: 'text-blue-400',
    orange: 'text-orange-400',
    cyan: 'text-cyan-400',
    violet: 'text-violet-400',
    green: 'text-green-400',
  };

  return (
    <div
      className={`p-4 rounded-xl bg-gradient-to-br ${colorClasses[color]} border transition-all duration-500 ${
        visible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'
      }`}
    >
      <div className="flex items-center gap-3">
        <div className={`p-2 rounded-lg bg-gray-900/50 ${textColors[color]}`}>
          <Icon className="w-5 h-5" />
        </div>
        <div>
          <div className="text-xs text-gray-500 uppercase tracking-wider">{label}</div>
          <div className={`text-xl font-bold font-mono ${textColors[color]}`}>
            {animatedValue}{unit}
          </div>
        </div>
      </div>
    </div>
  );
}

// Player positioning bar animation
function PositioningBar({
  offensive,
  midfield,
  defensive,
  playerName,
  color,
}: {
  offensive: number;
  midfield: number;
  defensive: number;
  playerName: string;
  color: 'blue' | 'orange';
}) {
  const colorClasses = {
    blue: {
      text: 'text-blue-400',
      offensive: 'bg-blue-500',
      midfield: 'bg-blue-400',
      defensive: 'bg-blue-300',
    },
    orange: {
      text: 'text-orange-400',
      offensive: 'bg-orange-500',
      midfield: 'bg-orange-400',
      defensive: 'bg-orange-300',
    },
  };

  return (
    <div className="space-y-1">
      <div className="flex justify-between items-center">
        <span className={`text-sm font-medium ${colorClasses[color].text}`}>{playerName}</span>
        <div className="flex gap-3 text-xs text-gray-500">
          <span>DEF {defensive}%</span>
          <span>MID {midfield}%</span>
          <span>OFF {offensive}%</span>
        </div>
      </div>
      <div className="flex h-2 rounded-full overflow-hidden bg-gray-800">
        <div
          className={`${colorClasses[color].defensive} transition-all duration-1000`}
          style={{ width: `${defensive}%` }}
        />
        <div
          className={`${colorClasses[color].midfield} transition-all duration-1000`}
          style={{ width: `${midfield}%` }}
        />
        <div
          className={`${colorClasses[color].offensive} transition-all duration-1000`}
          style={{ width: `${offensive}%` }}
        />
      </div>
    </div>
  );
}

// Stats ticker animation
function StatsTicker() {
  const [currentStat, setCurrentStat] = useState(0);
  const stats = [
    { label: 'Supersonic Speed', value: '2.4s', icon: Zap },
    { label: 'Boost Efficiency', value: '78%', icon: Flame },
    { label: 'Air Time', value: '12.5s', icon: TrendingUp },
    { label: 'Ball Touches', value: '23', icon: Target },
  ];

  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentStat((prev) => (prev + 1) % stats.length);
    }, 2500);
    return () => clearInterval(interval);
  }, []);

  const CurrentIcon = stats[currentStat].icon;

  return (
    <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-gray-800/50 border border-gray-700/50">
      <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
      <CurrentIcon className="w-4 h-4 text-violet-400" />
      <span className="text-sm text-gray-400">{stats[currentStat].label}:</span>
      <span className="text-sm font-mono font-bold text-white">{stats[currentStat].value}</span>
    </div>
  );
}

export function StatsShowcase() {
  return (
    <section className="relative">
      {/* Background gradient */}
      <div className="absolute inset-0 bg-gradient-to-r from-emerald-600/5 via-cyan-600/5 to-violet-600/5 rounded-3xl" />

      <div className="relative rounded-3xl border border-emerald-500/20 p-8 lg:p-12">
        <div className="grid lg:grid-cols-2 gap-12 items-center">
          {/* Left: Content */}
          <div className="space-y-6">
            <div className="flex flex-wrap gap-2">
              <span className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-sm">
                <BarChart3 className="w-4 h-4" />
                New Feature
              </span>
              <StatsTicker />
            </div>

            <h2 className="text-4xl font-bold">
              <span className="text-white">Advanced </span>
              <span className="bg-gradient-to-r from-emerald-400 to-cyan-400 bg-clip-text text-transparent">Statistics</span>
            </h2>

            <p className="text-lg text-gray-400">
              Dive deep into your gameplay with comprehensive analytics.
              Track speed, boost efficiency, positioning, and more for every player.
            </p>

            <div className="grid grid-cols-2 gap-3">
              <StatCard icon={Gauge} label="Max Speed" value={138.5} unit=" km/h" color="cyan" delay={0} />
              <StatCard icon={Flame} label="Boost Used" value={847} unit="" color="orange" delay={200} />
              <StatCard icon={Clock} label="Air Time" value={18.3} unit="s" color="violet" delay={400} />
              <StatCard icon={Target} label="Ball Touches" value={42} unit="" color="green" delay={600} />
            </div>

            <div className="pt-4">
              <Link to="/replays">
                <GradientButton size="lg" className="bg-gradient-to-r from-emerald-600 to-cyan-600 hover:from-emerald-500 hover:to-cyan-500">
                  <BarChart3 className="w-5 h-5" />
                  Explore Stats
                  <ArrowRight className="w-4 h-4 ml-1" />
                </GradientButton>
              </Link>
            </div>
          </div>

          {/* Right: Visual Demo */}
          <div className="relative">
            <div className="absolute inset-0 bg-gradient-to-r from-emerald-600 to-cyan-600 rounded-2xl blur-3xl opacity-20" />
            <div className="relative bg-gray-900/80 backdrop-blur rounded-2xl border border-gray-700 overflow-hidden">
              {/* Header */}
              <div className="p-4 border-b border-gray-700 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <BarChart3 className="w-5 h-5 text-emerald-400" />
                  <span className="font-medium text-white">Match Analysis</span>
                </div>
                <div className="flex items-center gap-2 text-gray-400 text-sm">
                  <Users className="w-4 h-4" />
                  <span>3v3 Ranked</span>
                </div>
              </div>

              {/* Stats Content */}
              <div className="p-5 space-y-6">
                {/* Possession Bar */}
                <AnimatedPossessionBar />

                {/* Team comparison */}
                <div className="grid grid-cols-3 gap-4 text-center py-4 border-y border-gray-700/50">
                  <div>
                    <div className="text-2xl font-bold text-blue-400 font-mono">82.4</div>
                    <div className="text-xs text-gray-500">Avg Speed</div>
                  </div>
                  <div>
                    <div className="text-xs text-gray-400 uppercase tracking-wider mb-1">km/h</div>
                    <div className="text-2xl font-bold text-gray-600">vs</div>
                  </div>
                  <div>
                    <div className="text-2xl font-bold text-orange-400 font-mono">78.9</div>
                    <div className="text-xs text-gray-500">Avg Speed</div>
                  </div>
                </div>

                {/* Player positioning */}
                <div className="space-y-3">
                  <div className="text-xs text-gray-500 uppercase tracking-wider">Player Positioning</div>
                  <PositioningBar
                    playerName="SquishyMuffinz"
                    offensive={45}
                    midfield={32}
                    defensive={23}
                    color="blue"
                  />
                  <PositioningBar
                    playerName="JSTN"
                    offensive={38}
                    midfield={35}
                    defensive={27}
                    color="blue"
                  />
                  <PositioningBar
                    playerName="GarrettG"
                    offensive={25}
                    midfield={30}
                    defensive={45}
                    color="blue"
                  />
                </div>

                {/* Boost efficiency mini chart */}
                <div className="flex items-end gap-1 h-12">
                  {[65, 82, 45, 91, 73, 88, 56, 79, 68, 85, 72, 94].map((height, i) => (
                    <div
                      key={i}
                      className="flex-1 bg-gradient-to-t from-emerald-600 to-cyan-400 rounded-t opacity-80 hover:opacity-100 transition-opacity"
                      style={{
                        height: `${height}%`,
                        animationDelay: `${i * 100}ms`,
                      }}
                    />
                  ))}
                </div>
                <div className="flex justify-between text-xs text-gray-500">
                  <span>0:00</span>
                  <span className="text-gray-400">Boost Usage Over Time</span>
                  <span>5:00</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
