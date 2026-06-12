import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Activity, X, HelpCircle } from 'lucide-react';

const STORAGE_KEY = 'quality-info-banner-dismissed';

export function QualityInfoBanner() {
  const [isDismissed, setIsDismissed] = useState(true); // Start hidden to avoid flash

  useEffect(() => {
    const dismissed = localStorage.getItem(STORAGE_KEY);
    setIsDismissed(dismissed === 'true');
  }, []);

  const handleDismiss = () => {
    localStorage.setItem(STORAGE_KEY, 'true');
    setIsDismissed(true);
  };

  if (isDismissed) {
    return null;
  }

  return (
    <div className="relative overflow-hidden rounded-xl bg-gradient-to-r from-amber-500/10 via-orange-500/10 to-amber-500/10 border border-amber-500/20 p-4">
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_left,_var(--tw-gradient-stops))] from-amber-500/5 via-transparent to-transparent" />

      {/* Dismiss button - absolute positioned */}
      <button
        onClick={handleDismiss}
        className="absolute top-2 right-2 p-1.5 rounded-lg text-gray-500 hover:text-gray-300 hover:bg-gray-800/50 transition-colors z-10"
        title="Dismiss"
      >
        <X className="w-4 h-4" />
      </button>

      <div className="relative flex flex-col sm:flex-row sm:items-center gap-3 pr-8 sm:pr-0">
        <div className="flex items-start sm:items-center gap-3 min-w-0 flex-1">
          <div className="w-9 h-9 sm:w-10 sm:h-10 rounded-lg bg-amber-500/20 flex items-center justify-center flex-shrink-0">
            <Activity className="w-4 h-4 sm:w-5 sm:h-5 text-amber-400" />
          </div>

          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium text-amber-400 mb-0.5">
              About replay quality
            </p>
            <p className="text-xs text-gray-400">
              Some replays may show warnings. We apply filtering to improve quality.
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2 flex-shrink-0 sm:ml-auto">
          <Link
            to="/faq/replay-quality"
            className="flex items-center justify-center gap-1.5 px-3 py-2 rounded-lg bg-amber-500/20 text-amber-400 hover:bg-amber-500/30 transition-colors text-sm font-medium min-h-[40px] w-full sm:w-auto"
          >
            <HelpCircle className="w-4 h-4" />
            Learn more
          </Link>
        </div>
      </div>
    </div>
  );
}
