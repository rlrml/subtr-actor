import { useEffect, useState, useRef } from 'react';
import { Link } from 'react-router-dom';
import { Play, Eye, Users, Target, Shield } from 'lucide-react';
import { api } from '@/services/api';
import { useCheatStats } from '@/api/cheat';

interface GlobalStats {
  totalReplays: number;
  totalViews: number;
  totalUsers: number;
  totalGoals: number;
  cachedAt: number;
}

interface AnimatedCounterProps {
  value: number;
  duration?: number;
  icon: React.ReactNode;
  label: string;
  gradient: string;
}

function AnimatedCounter({ value, duration = 2000, icon, label, gradient }: AnimatedCounterProps) {
  const [displayValue, setDisplayValue] = useState(0);
  const startTimeRef = useRef<number | null>(null);
  const rafRef = useRef<number | null>(null);

  useEffect(() => {
    if (value === 0) {
      setDisplayValue(0);
      return;
    }

    const animate = (timestamp: number) => {
      if (!startTimeRef.current) {
        startTimeRef.current = timestamp;
      }

      const progress = Math.min((timestamp - startTimeRef.current) / duration, 1);
      // Easing function for smooth deceleration
      const easeOutQuart = 1 - Math.pow(1 - progress, 4);
      const currentValue = Math.floor(easeOutQuart * value);

      setDisplayValue(currentValue);

      if (progress < 1) {
        rafRef.current = requestAnimationFrame(animate);
      } else {
        setDisplayValue(value);
      }
    };

    startTimeRef.current = null;
    rafRef.current = requestAnimationFrame(animate);

    return () => {
      if (rafRef.current) {
        cancelAnimationFrame(rafRef.current);
      }
    };
  }, [value, duration]);

  const formatNumber = (num: number): string => {
    if (num >= 1000000) {
      return (num / 1000000).toFixed(1) + 'M';
    }
    if (num >= 1000) {
      return (num / 1000).toFixed(1) + 'K';
    }
    return num.toLocaleString();
  };

  return (
    <div className="flex flex-col items-center gap-1">
      <div className="flex items-center gap-2">
        {icon}
        <span className={`text-2xl font-bold tabular-nums bg-gradient-to-r ${gradient} bg-clip-text text-transparent`}>
          {formatNumber(displayValue)}
        </span>
      </div>
      <div className="text-[10px] text-gray-500 uppercase tracking-wider">
        {label}
      </div>
    </div>
  );
}

export function GlobalStats() {
  const [stats, setStats] = useState<GlobalStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isVisible, setIsVisible] = useState(false);

  // Cheat detection stats (032-cheat-detection)
  const { data: cheatStats } = useCheatStats();

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const data = await api.get<GlobalStats>('/stats');
        setStats(data);
      } catch (error) {
        console.error('Failed to fetch global stats:', error);
      } finally {
        setIsLoading(false);
      }
    };

    fetchStats();
  }, []);

  // Trigger animation when stats are loaded
  useEffect(() => {
    if (stats && !isVisible) {
      // Small delay for smooth entrance
      const timer = setTimeout(() => setIsVisible(true), 150);
      return () => clearTimeout(timer);
    }
  }, [stats, isVisible]);

  if (isLoading) {
    return (
      <div className="flex justify-center py-6">
        <div className="w-6 h-6 border-2 border-violet-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  if (!stats) {
    return null;
  }

  // Don't show if all stats are zero
  const hasStats = stats.totalReplays > 0 || stats.totalViews > 0 || stats.totalUsers > 0 || stats.totalGoals > 0;
  if (!hasStats) {
    return null;
  }

  return (
    <div className="relative py-2">
      {/* Subtle title */}
      <div className="text-center mb-3">
        <span className="text-xs text-gray-600 uppercase tracking-widest">Platform Stats</span>
      </div>

      {/* Stats row - minimal styling */}
      <div className="flex flex-wrap justify-center gap-8 md:gap-12 lg:gap-16">
        <AnimatedCounter
          value={isVisible ? stats.totalReplays : 0}
          icon={<Play className="w-4 h-4 text-violet-400" />}
          label="Replays"
          gradient="from-violet-400 to-violet-300"
        />
        <AnimatedCounter
          value={isVisible ? stats.totalViews : 0}
          icon={<Eye className="w-4 h-4 text-blue-400" />}
          label="Views"
          gradient="from-blue-400 to-blue-300"
        />
        <AnimatedCounter
          value={isVisible ? stats.totalUsers : 0}
          icon={<Users className="w-4 h-4 text-cyan-400" />}
          label="Users"
          gradient="from-cyan-400 to-cyan-300"
        />
        <AnimatedCounter
          value={isVisible ? stats.totalGoals : 0}
          icon={<Target className="w-4 h-4 text-violet-400" />}
          label="Goals"
          gradient="from-violet-400 to-blue-400"
        />
        {/* Cheat Detection Stat (032-cheat-detection) */}
        {cheatStats && cheatStats.totalCheatersDetected > 0 && (
          <Link to="/cheaters" className="group">
            <AnimatedCounter
              value={isVisible ? cheatStats.totalCheatersDetected : 0}
              icon={<Shield className="w-4 h-4 text-red-400 group-hover:text-red-300 transition-colors" />}
              label="Cheaters Caught"
              gradient="from-red-400 to-red-300"
            />
          </Link>
        )}
      </div>
    </div>
  );
}
