/**
 * NotFound - 404 Page with proper SEO
 * (022-seo-optimization)
 */

import { Link } from 'react-router-dom';
import { SEOHead } from '@/components/SEO';
import { Home, Search, ArrowLeft } from 'lucide-react';
import { GradientButton } from '@/components/ui/GradientButton';

export default function NotFound() {
  return (
    <div className="min-h-[70vh] flex items-center justify-center px-4">
      <SEOHead
        title="Page Not Found"
        description="The page you're looking for doesn't exist. Browse our replay collection or return to the homepage."
        noIndex
      />

      <div className="text-center max-w-md">
        {/* 404 Visual */}
        <div className="relative mb-8">
          <div className="text-[150px] font-black text-gray-800/50 leading-none select-none">
            404
          </div>
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="w-24 h-24 rounded-full bg-gradient-to-br from-violet-600/20 to-blue-600/20 border border-violet-500/30 flex items-center justify-center">
              <Search className="w-10 h-10 text-violet-400" />
            </div>
          </div>
        </div>

        <h1 className="text-2xl font-bold text-white mb-3">
          Page Not Found
        </h1>

        <p className="text-gray-400 mb-8">
          The page you're looking for doesn't exist or has been moved.
          Let's get you back on track.
        </p>

        <div className="flex flex-col sm:flex-row gap-3 justify-center">
          <Link to="/">
            <GradientButton size="lg" className="w-full sm:w-auto">
              <Home className="w-5 h-5" />
              Go to Homepage
            </GradientButton>
          </Link>

          <Link to="/replays">
            <GradientButton size="lg" variant="outline" className="w-full sm:w-auto">
              <Search className="w-5 h-5" />
              Browse Replays
            </GradientButton>
          </Link>
        </div>

        <button
          onClick={() => window.history.back()}
          className="mt-6 inline-flex items-center gap-2 text-sm text-gray-500 hover:text-violet-400 transition-colors"
        >
          <ArrowLeft className="w-4 h-4" />
          Go back to previous page
        </button>
      </div>
    </div>
  );
}
