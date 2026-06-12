import { ReactNode, useState, useEffect } from 'react';
import { ChevronUp, ChevronDown, ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { Pagination } from '@/hooks/useAdminApi';

// =====================================
// Types
// =====================================

export interface Column<T> {
  key: string;
  header: string;
  sortable?: boolean;
  width?: string;
  render?: (item: T) => ReactNode;
  className?: string;
  /** Hide this column on mobile card view (shown in table view) */
  hideOnMobile?: boolean;
}

export interface DataTableProps<T> {
  data: T[];
  columns: Column<T>[];
  keyExtractor: (item: T) => string;
  pagination?: Pagination | null;
  onPageChange?: (page: number) => void;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  onSort?: (key: string) => void;
  loading?: boolean;
  emptyMessage?: string;
  rowClassName?: (item: T) => string;
  onRowClick?: (item: T) => void;
  /** Custom render function for mobile card view. If not provided, auto-generates from columns */
  mobileCardRender?: (item: T) => ReactNode;
}

// =====================================
// Component
// =====================================

export function DataTable<T>({
  data,
  columns,
  keyExtractor,
  pagination,
  onPageChange,
  sortBy,
  sortOrder,
  onSort,
  loading = false,
  emptyMessage = 'No data found',
  rowClassName,
  onRowClick,
  mobileCardRender,
}: DataTableProps<T>) {
  // Track if we're in mobile view
  const [isMobileView, setIsMobileView] = useState(
    typeof window !== 'undefined' ? window.innerWidth < 768 : false
  );

  useEffect(() => {
    const handleResize = () => {
      setIsMobileView(window.innerWidth < 768);
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  // Render table header
  const renderHeader = () => (
    <thead>
      <tr className="border-b border-gray-800">
        {columns.map((column) => (
          <th
            key={column.key}
            className={cn(
              'px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider',
              column.sortable && 'cursor-pointer hover:text-white transition-colors select-none',
              column.width,
              column.className
            )}
            onClick={() => column.sortable && onSort?.(column.key)}
          >
            <div className="flex items-center gap-1">
              <span>{column.header}</span>
              {column.sortable && sortBy === column.key && (
                sortOrder === 'asc' ? (
                  <ChevronUp className="w-3 h-3" />
                ) : (
                  <ChevronDown className="w-3 h-3" />
                )
              )}
            </div>
          </th>
        ))}
      </tr>
    </thead>
  );

  // Render loading skeleton for table
  const renderLoading = () => (
    <tbody>
      {[...Array(5)].map((_, i) => (
        <tr key={i} className="border-b border-gray-800/50">
          {columns.map((column) => (
            <td key={column.key} className="px-4 py-3">
              <div className="h-4 bg-gray-700/50 rounded animate-pulse" />
            </td>
          ))}
        </tr>
      ))}
    </tbody>
  );

  // Render loading skeleton for mobile cards
  const renderMobileLoading = () => (
    <div className="space-y-3">
      {[...Array(3)].map((_, i) => (
        <div key={i} className="bg-gray-900/50 border border-gray-800 rounded-xl p-4 animate-pulse">
          <div className="h-4 bg-gray-700/50 rounded w-3/4 mb-3" />
          <div className="h-3 bg-gray-700/50 rounded w-1/2 mb-2" />
          <div className="h-3 bg-gray-700/50 rounded w-2/3" />
        </div>
      ))}
    </div>
  );

  // Render empty state
  const renderEmpty = () => (
    <tbody>
      <tr>
        <td
          colSpan={columns.length}
          className="px-4 py-12 text-center text-gray-500"
        >
          {emptyMessage}
        </td>
      </tr>
    </tbody>
  );

  // Render mobile empty state
  const renderMobileEmpty = () => (
    <div className="text-center text-gray-500 py-12 bg-gray-900/50 border border-gray-800 rounded-xl">
      {emptyMessage}
    </div>
  );

  // Render data rows
  const renderData = () => (
    <tbody>
      {data.map((item) => (
        <tr
          key={keyExtractor(item)}
          className={cn(
            'border-b border-gray-800/50 hover:bg-gray-800/30 transition-colors',
            onRowClick && 'cursor-pointer',
            rowClassName?.(item)
          )}
          onClick={() => onRowClick?.(item)}
        >
          {columns.map((column) => (
            <td
              key={column.key}
              className={cn('px-4 py-3 text-sm text-gray-300', column.className)}
            >
              {column.render
                ? column.render(item)
                : String((item as Record<string, unknown>)[column.key] ?? '-')}
            </td>
          ))}
        </tr>
      ))}
    </tbody>
  );

  // Auto-generate mobile card content from columns
  const renderAutoMobileCard = (item: T) => {
    const visibleColumns = columns.filter(col => !col.hideOnMobile);
    const [primaryColumn, ...otherColumns] = visibleColumns;

    return (
      <div className="space-y-2">
        {/* Primary column (first one) as title */}
        {primaryColumn && (
          <div className="font-medium text-white">
            {primaryColumn.render
              ? primaryColumn.render(item)
              : String((item as Record<string, unknown>)[primaryColumn.key] ?? '-')}
          </div>
        )}
        {/* Other columns as key-value pairs */}
        <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
          {otherColumns.map((column) => (
            <div key={column.key} className="contents">
              <span className="text-gray-500">{column.header}</span>
              <span className="text-gray-300 truncate">
                {column.render
                  ? column.render(item)
                  : String((item as Record<string, unknown>)[column.key] ?? '-')}
              </span>
            </div>
          ))}
        </div>
      </div>
    );
  };

  // Render mobile cards
  const renderMobileCards = () => (
    <div className="space-y-3">
      {data.map((item) => (
        <div
          key={keyExtractor(item)}
          className={cn(
            'bg-gray-900/50 border border-gray-800 rounded-xl p-4 transition-colors',
            onRowClick && 'cursor-pointer active:bg-gray-800/50',
            rowClassName?.(item)
          )}
          onClick={() => onRowClick?.(item)}
        >
          {mobileCardRender ? mobileCardRender(item) : renderAutoMobileCard(item)}
        </div>
      ))}
    </div>
  );

  // Render pagination for desktop
  const renderPagination = () => {
    if (!pagination || pagination.totalPages <= 1) return null;

    const { page, totalPages, total } = pagination;
    const canGoPrev = page > 1;
    const canGoNext = page < totalPages;

    return (
      <div className="flex items-center justify-between px-4 py-3 border-t border-gray-800">
        <div className="text-sm text-gray-500">
          Showing page {page} of {totalPages} ({total.toLocaleString()} total)
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => onPageChange?.(1)}
            disabled={!canGoPrev}
            className={cn(
              'p-1.5 rounded-lg transition-colors',
              canGoPrev
                ? 'text-gray-400 hover:text-white hover:bg-gray-800'
                : 'text-gray-600 cursor-not-allowed'
            )}
            title="First page"
          >
            <ChevronsLeft className="w-4 h-4" />
          </button>
          <button
            onClick={() => onPageChange?.(page - 1)}
            disabled={!canGoPrev}
            className={cn(
              'p-1.5 rounded-lg transition-colors',
              canGoPrev
                ? 'text-gray-400 hover:text-white hover:bg-gray-800'
                : 'text-gray-600 cursor-not-allowed'
            )}
            title="Previous page"
          >
            <ChevronLeft className="w-4 h-4" />
          </button>
          <span className="px-3 py-1 text-sm text-gray-400">
            {page} / {totalPages}
          </span>
          <button
            onClick={() => onPageChange?.(page + 1)}
            disabled={!canGoNext}
            className={cn(
              'p-1.5 rounded-lg transition-colors',
              canGoNext
                ? 'text-gray-400 hover:text-white hover:bg-gray-800'
                : 'text-gray-600 cursor-not-allowed'
            )}
            title="Next page"
          >
            <ChevronRight className="w-4 h-4" />
          </button>
          <button
            onClick={() => onPageChange?.(totalPages)}
            disabled={!canGoNext}
            className={cn(
              'p-1.5 rounded-lg transition-colors',
              canGoNext
                ? 'text-gray-400 hover:text-white hover:bg-gray-800'
                : 'text-gray-600 cursor-not-allowed'
            )}
            title="Last page"
          >
            <ChevronsRight className="w-4 h-4" />
          </button>
        </div>
      </div>
    );
  };

  // Render mobile pagination (simplified)
  const renderMobilePagination = () => {
    if (!pagination || pagination.totalPages <= 1) return null;

    const { page, totalPages, total } = pagination;
    const canGoPrev = page > 1;
    const canGoNext = page < totalPages;

    return (
      <div className="flex flex-col items-center gap-3 pt-4">
        <div className="text-xs text-gray-500">
          Page {page} of {totalPages} ({total.toLocaleString()} items)
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => onPageChange?.(page - 1)}
            disabled={!canGoPrev}
            className={cn(
              'p-3 rounded-lg transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center',
              canGoPrev
                ? 'text-gray-400 hover:text-white bg-gray-800 hover:bg-gray-700'
                : 'text-gray-600 bg-gray-800/50 cursor-not-allowed'
            )}
            aria-label="Previous page"
          >
            <ChevronLeft className="w-5 h-5" />
          </button>
          <span className="px-4 py-2 text-sm text-gray-400 bg-gray-800/50 rounded-lg min-w-[80px] text-center">
            {page} / {totalPages}
          </span>
          <button
            onClick={() => onPageChange?.(page + 1)}
            disabled={!canGoNext}
            className={cn(
              'p-3 rounded-lg transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center',
              canGoNext
                ? 'text-gray-400 hover:text-white bg-gray-800 hover:bg-gray-700'
                : 'text-gray-600 bg-gray-800/50 cursor-not-allowed'
            )}
            aria-label="Next page"
          >
            <ChevronRight className="w-5 h-5" />
          </button>
        </div>
      </div>
    );
  };

  // Mobile view - card layout
  if (isMobileView) {
    return (
      <div>
        {loading
          ? renderMobileLoading()
          : data.length === 0
            ? renderMobileEmpty()
            : renderMobileCards()}
        {renderMobilePagination()}
      </div>
    );
  }

  // Desktop view - table layout
  return (
    <div className="bg-gray-900/50 border border-gray-800 rounded-xl overflow-hidden">
      <div className="overflow-x-auto">
        <table className="w-full">
          {renderHeader()}
          {loading ? renderLoading() : data.length === 0 ? renderEmpty() : renderData()}
        </table>
      </div>
      {renderPagination()}
    </div>
  );
}

export default DataTable;
