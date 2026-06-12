import { useState, useEffect } from 'react';
import { Search, MapPin, SortAsc, X, ChevronDown } from 'lucide-react';
import { api } from '@/services/api';

export interface FilterState {
  search: string;
  map: string;
  sortBy: 'createdAt' | 'playedAt' | 'likeCount' | 'viewCount';
  sortOrder: 'asc' | 'desc';
}

interface SearchFiltersProps {
  filters: FilterState;
  onFiltersChange: (filters: FilterState) => void;
  onReset: () => void;
}

const SORT_OPTIONS = [
  { value: 'createdAt', label: 'Date uploaded' },
  { value: 'playedAt', label: 'Date played' },
  { value: 'viewCount', label: 'Most viewed' },
  { value: 'likeCount', label: 'Most liked' },
] as const;

function getMapDisplayName(mapName: string): string {
  return mapName.replace(/_P$/, '').replace(/_Standard$/, '').replace(/_/g, ' ');
}

export function SearchFilters({ filters, onFiltersChange, onReset }: SearchFiltersProps) {
  const [maps, setMaps] = useState<string[]>([]);
  const [loadingMaps, setLoadingMaps] = useState(true);
  const [mapDropdownOpen, setMapDropdownOpen] = useState(false);
  const [sortDropdownOpen, setSortDropdownOpen] = useState(false);

  // Fetch available maps on mount
  useEffect(() => {
    const fetchMaps = async () => {
      try {
        const data = await api.get<{ maps: string[] }>('/replays/filters');
        setMaps(data.maps);
      } catch (err) {
        console.error('Failed to fetch filter options:', err);
      } finally {
        setLoadingMaps(false);
      }
    };
    fetchMaps();
  }, []);

  const hasActiveFilters = filters.search || filters.map || filters.sortBy !== 'createdAt';

  const handleSearchChange = (value: string) => {
    onFiltersChange({ ...filters, search: value });
  };

  const handleMapChange = (value: string) => {
    onFiltersChange({ ...filters, map: value });
    setMapDropdownOpen(false);
  };

  const handleSortChange = (value: typeof filters.sortBy) => {
    // Default to desc for most sort options (newest first, most views, most likes)
    const sortOrder = value === 'createdAt' || value === 'playedAt' ? 'desc' : 'desc';
    onFiltersChange({ ...filters, sortBy: value, sortOrder });
    setSortDropdownOpen(false);
  };

  const currentSortLabel = SORT_OPTIONS.find(o => o.value === filters.sortBy)?.label || 'Date uploaded';

  return (
    <div className="flex flex-col sm:flex-row sm:flex-wrap items-stretch sm:items-center gap-2 sm:gap-3">
      {/* Search input */}
      <div className="relative w-full sm:flex-1 sm:min-w-[200px] sm:max-w-md">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input
          type="text"
          placeholder="Search replays, players..."
          value={filters.search}
          onChange={(e) => handleSearchChange(e.target.value)}
          className="w-full pl-10 pr-10 py-2.5 sm:py-2 rounded-lg bg-gray-800/50 border border-gray-700 text-white text-base sm:text-sm placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent transition-all"
        />
        {filters.search && (
          <button
            onClick={() => handleSearchChange('')}
            className="absolute right-3 top-1/2 -translate-y-1/2 p-1 text-gray-400 hover:text-white transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        )}
      </div>

      {/* Dropdown filters row */}
      <div className="flex flex-col xs:flex-row items-stretch xs:items-center gap-2 sm:gap-3 w-full sm:w-auto">
        {/* Map filter dropdown */}
        <div className="relative w-full xs:w-auto xs:flex-1 sm:flex-none">
          <button
            onClick={() => {
              setMapDropdownOpen(!mapDropdownOpen);
              setSortDropdownOpen(false);
            }}
            className={`w-full flex items-center justify-between gap-2 px-3 py-2.5 sm:py-2 rounded-lg border transition-all min-h-[44px] sm:min-h-0 ${
              filters.map
                ? 'bg-violet-500/20 border-violet-500/50 text-violet-300'
                : 'bg-gray-800/50 border-gray-700 text-gray-300 hover:border-gray-600'
            }`}
          >
            <MapPin className="w-4 h-4 shrink-0" />
            <span className="flex-1 text-left truncate text-sm">
              {filters.map ? getMapDisplayName(filters.map) : 'All maps'}
            </span>
            <ChevronDown className={`w-4 h-4 shrink-0 transition-transform ${mapDropdownOpen ? 'rotate-180' : ''}`} />
          </button>

          {mapDropdownOpen && (
            <>
              <div
                className="fixed inset-0 z-40"
                onClick={() => setMapDropdownOpen(false)}
              />
              <div className="absolute top-full left-0 right-0 sm:left-0 sm:right-auto mt-1 w-full sm:w-56 max-h-64 overflow-y-auto rounded-lg bg-gray-800 border border-gray-700 shadow-xl z-50">
                <button
                  onClick={() => handleMapChange('')}
                  className={`w-full px-3 py-2.5 sm:py-2 text-left text-sm transition-colors ${
                    !filters.map ? 'bg-violet-500/20 text-violet-300' : 'text-gray-300 hover:bg-gray-700'
                  }`}
                >
                  All maps
                </button>
                {loadingMaps ? (
                  <div className="px-3 py-2 text-sm text-gray-500">Loading...</div>
                ) : maps.length === 0 ? (
                  <div className="px-3 py-2 text-sm text-gray-500">No maps found</div>
                ) : (
                  maps.map((map) => (
                    <button
                      key={map}
                      onClick={() => handleMapChange(map)}
                      className={`w-full px-3 py-2.5 sm:py-2 text-left text-sm transition-colors ${
                        filters.map === map ? 'bg-violet-500/20 text-violet-300' : 'text-gray-300 hover:bg-gray-700'
                      }`}
                    >
                      {getMapDisplayName(map)}
                    </button>
                  ))
                )}
              </div>
            </>
          )}
        </div>

        {/* Sort dropdown */}
        <div className="relative w-full xs:w-auto xs:flex-1 sm:flex-none">
          <button
            onClick={() => {
              setSortDropdownOpen(!sortDropdownOpen);
              setMapDropdownOpen(false);
            }}
            className={`w-full flex items-center justify-between gap-2 px-3 py-2.5 sm:py-2 rounded-lg border transition-all min-h-[44px] sm:min-h-0 ${
              filters.sortBy !== 'createdAt'
                ? 'bg-violet-500/20 border-violet-500/50 text-violet-300'
                : 'bg-gray-800/50 border-gray-700 text-gray-300 hover:border-gray-600'
            }`}
          >
            <SortAsc className="w-4 h-4 shrink-0" />
            <span className="flex-1 text-left truncate text-sm">{currentSortLabel}</span>
            <ChevronDown className={`w-4 h-4 shrink-0 transition-transform ${sortDropdownOpen ? 'rotate-180' : ''}`} />
          </button>

          {sortDropdownOpen && (
            <>
              <div
                className="fixed inset-0 z-40"
                onClick={() => setSortDropdownOpen(false)}
              />
              <div className="absolute top-full left-0 right-0 sm:left-auto sm:right-0 mt-1 w-full sm:w-44 rounded-lg bg-gray-800 border border-gray-700 shadow-xl z-50">
                {SORT_OPTIONS.map((option) => (
                  <button
                    key={option.value}
                    onClick={() => handleSortChange(option.value)}
                    className={`w-full px-3 py-2.5 sm:py-2 text-left text-sm transition-colors ${
                      filters.sortBy === option.value ? 'bg-violet-500/20 text-violet-300' : 'text-gray-300 hover:bg-gray-700'
                    }`}
                  >
                    {option.label}
                  </button>
                ))}
              </div>
            </>
          )}
        </div>

        {/* Reset button */}
        {hasActiveFilters && (
          <button
            onClick={onReset}
            className="w-full xs:w-auto flex items-center justify-center gap-1.5 px-3 py-2.5 sm:py-2 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-gray-800/50 border border-gray-700 xs:border-transparent transition-all min-h-[44px] sm:min-h-0"
          >
            <X className="w-4 h-4" />
            <span>Reset</span>
          </button>
        )}
      </div>
    </div>
  );
}

export const DEFAULT_FILTERS: FilterState = {
  search: '',
  map: '',
  sortBy: 'createdAt',
  sortOrder: 'desc',
};
