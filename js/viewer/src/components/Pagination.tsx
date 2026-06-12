import { ChevronLeft, ChevronRight } from 'lucide-react';

interface PaginationProps {
  page: number;
  totalPages: number;
  onPageChange: (page: number) => void;
}

export function Pagination({ page, totalPages, onPageChange }: PaginationProps) {
  if (totalPages <= 1) return null;

  const pages: (number | 'ellipsis')[] = [];

  // Always show first page
  pages.push(1);

  // Show ellipsis if needed before current range
  if (page > 3) {
    pages.push('ellipsis');
  }

  // Show pages around current
  for (let i = Math.max(2, page - 1); i <= Math.min(totalPages - 1, page + 1); i++) {
    if (!pages.includes(i)) {
      pages.push(i);
    }
  }

  // Show ellipsis if needed after current range
  if (page < totalPages - 2) {
    pages.push('ellipsis');
  }

  // Always show last page
  if (totalPages > 1 && !pages.includes(totalPages)) {
    pages.push(totalPages);
  }

  return (
    <div className="flex items-center justify-center gap-1 sm:gap-2">
      <button
        onClick={() => onPageChange(page - 1)}
        disabled={page === 1}
        className="p-2.5 sm:p-2 rounded-lg bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-white disabled:opacity-50 disabled:cursor-not-allowed transition-colors min-h-[44px] min-w-[44px] sm:min-h-0 sm:min-w-0 flex items-center justify-center"
        aria-label="Previous page"
      >
        <ChevronLeft className="w-5 h-5 sm:w-4 sm:h-4" />
      </button>

      {/* On mobile, show simplified page indicator */}
      <div className="flex sm:hidden items-center">
        <span className="px-4 py-2 text-sm text-gray-400 bg-gray-800 rounded-lg min-w-[80px] text-center">
          {page} / {totalPages}
        </span>
      </div>

      {/* On desktop, show full pagination */}
      <div className="hidden sm:flex items-center gap-1">
        {pages.map((p, idx) => (
          p === 'ellipsis' ? (
            <span key={`ellipsis-${idx}`} className="px-2 text-gray-600">...</span>
          ) : (
            <button
              key={p}
              onClick={() => onPageChange(p)}
              className={`min-w-[40px] h-10 rounded-lg font-medium transition-all ${
                p === page
                  ? 'bg-gradient-to-r from-violet-600 to-blue-600 text-white'
                  : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-white'
              }`}
            >
              {p}
            </button>
          )
        ))}
      </div>

      <button
        onClick={() => onPageChange(page + 1)}
        disabled={page === totalPages}
        className="p-2.5 sm:p-2 rounded-lg bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-white disabled:opacity-50 disabled:cursor-not-allowed transition-colors min-h-[44px] min-w-[44px] sm:min-h-0 sm:min-w-0 flex items-center justify-center"
        aria-label="Next page"
      >
        <ChevronRight className="w-5 h-5 sm:w-4 sm:h-4" />
      </button>
    </div>
  );
}
