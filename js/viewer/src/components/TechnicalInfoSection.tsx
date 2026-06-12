/**
 * TechnicalInfoSection - Displays technical version info for a replay
 *
 * Shows game version, build info, and file header details.
 * Collapsible by default to keep the UI clean.
 */

import { useState } from 'react';
import { ChevronDown, ChevronRight, Code2, Server, FileCode } from 'lucide-react';

interface TechnicalInfoSectionProps {
  /** Game version number */
  gameVersion?: number | null;
  /** Build ID from Rocket League */
  buildId?: number | null;
  /** Build version string */
  buildVersion?: string | null;
  /** Header file size in bytes */
  headerSize?: number | null;
  /** Header CRC checksum */
  headerCrc?: number | null;
  /** Major replay format version */
  majorVersion?: number | null;
  /** Minor replay format version */
  minorVersion?: number | null;
  /** Network version */
  netVersion?: number | null;
  /** Rocket League game type string */
  rlGameType?: string | null;
  /** Compact mode for sidebar display */
  compact?: boolean;
}

/**
 * Format a number with thousand separators
 */
function formatNumber(value: number | null | undefined): string {
  if (value === null || value === undefined) return '---';
  return value.toLocaleString();
}

/**
 * Format a hex value
 */
function formatHex(value: number | null | undefined): string {
  if (value === null || value === undefined) return '---';
  return `0x${value.toString(16).toUpperCase()}`;
}

export function TechnicalInfoSection({
  gameVersion,
  buildId,
  buildVersion,
  headerSize,
  headerCrc,
  majorVersion,
  minorVersion,
  netVersion,
  rlGameType,
  compact = false,
}: TechnicalInfoSectionProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  // Check if we have any data to display
  const hasData = gameVersion || buildId || buildVersion || headerSize ||
    headerCrc || majorVersion || minorVersion || netVersion || rlGameType;

  if (!hasData) {
    return null;
  }

  // Compact mode for sidebar - simplified display
  if (compact) {
    return (
      <div className="space-y-2">
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="w-full flex items-center justify-between text-xs text-gray-500 hover:text-gray-400 transition-colors"
        >
          <span>More details</span>
          {isExpanded ? (
            <ChevronDown className="w-3 h-3" />
          ) : (
            <ChevronRight className="w-3 h-3" />
          )}
        </button>

        {isExpanded && (
          <div className="space-y-1.5 text-xs">
            {gameVersion && (
              <div className="flex justify-between">
                <span className="text-gray-500">Game Version</span>
                <span className="text-gray-400 font-mono">{formatNumber(gameVersion)}</span>
              </div>
            )}
            {buildId && (
              <div className="flex justify-between">
                <span className="text-gray-500">Build ID</span>
                <span className="text-gray-400 font-mono">{formatNumber(buildId)}</span>
              </div>
            )}
            {(majorVersion !== null || minorVersion !== null) && (
              <div className="flex justify-between">
                <span className="text-gray-500">Format</span>
                <span className="text-gray-400 font-mono">{majorVersion ?? '?'}.{minorVersion ?? '?'}</span>
              </div>
            )}
            {netVersion !== null && netVersion !== undefined && (
              <div className="flex justify-between">
                <span className="text-gray-500">Net Version</span>
                <span className="text-gray-400 font-mono">{formatNumber(netVersion)}</span>
              </div>
            )}
            {headerCrc && (
              <div className="flex justify-between">
                <span className="text-gray-500">CRC</span>
                <span className="text-gray-400 font-mono">{formatHex(headerCrc)}</span>
              </div>
            )}
          </div>
        )}
      </div>
    );
  }

  // Full mode - original display
  return (
    <div className="border border-gray-700 rounded-lg overflow-hidden">
      {/* Header - clickable to toggle */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between p-3 bg-gray-800/50 hover:bg-gray-800/70 transition-colors"
      >
        <div className="flex items-center gap-2 text-gray-400">
          <Code2 className="w-4 h-4" />
          <span className="text-sm font-medium">Technical Information</span>
        </div>
        {isExpanded ? (
          <ChevronDown className="w-4 h-4 text-gray-500" />
        ) : (
          <ChevronRight className="w-4 h-4 text-gray-500" />
        )}
      </button>

      {/* Content - collapsible */}
      {isExpanded && (
        <div className="p-4 space-y-4 bg-gray-900/50">
          {/* Game Version Info */}
          {(gameVersion || buildId || buildVersion) && (
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-xs text-gray-500 uppercase tracking-wider">
                <Server className="w-3 h-3" />
                Game Version
              </div>
              <div className="flex flex-wrap gap-x-6 gap-y-1 text-sm">
                {gameVersion && (
                  <div className="flex items-center gap-2">
                    <span className="text-gray-500">Version</span>
                    <span className="text-gray-300 font-mono">{formatNumber(gameVersion)}</span>
                  </div>
                )}
                {buildId && (
                  <div className="flex items-center gap-2">
                    <span className="text-gray-500">Build ID</span>
                    <span className="text-gray-300 font-mono">{formatNumber(buildId)}</span>
                  </div>
                )}
                {buildVersion && (
                  <div className="flex items-center gap-2">
                    <span className="text-gray-500">Build Version</span>
                    <span className="text-gray-300 font-mono">{buildVersion}</span>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Replay Format Info */}
          {(majorVersion !== null || minorVersion !== null || netVersion !== null || rlGameType) && (
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-xs text-gray-500 uppercase tracking-wider">
                <FileCode className="w-3 h-3" />
                Replay Format
              </div>
              <div className="flex flex-wrap gap-x-6 gap-y-1 text-sm">
                {(majorVersion !== null || minorVersion !== null) && (
                  <div className="flex items-center gap-2">
                    <span className="text-gray-500">Format Version</span>
                    <span className="text-gray-300 font-mono">
                      {majorVersion ?? '?'}.{minorVersion ?? '?'}
                    </span>
                  </div>
                )}
                {netVersion !== null && netVersion !== undefined && (
                  <div className="flex items-center gap-2">
                    <span className="text-gray-500">Net Version</span>
                    <span className="text-gray-300 font-mono">{formatNumber(netVersion)}</span>
                  </div>
                )}
                {rlGameType && (
                  <div className="flex items-center gap-2">
                    <span className="text-gray-500">Game Type</span>
                    <span className="text-gray-300">{rlGameType}</span>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* File Header Info */}
          {(headerSize || headerCrc) && (
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-xs text-gray-500 uppercase tracking-wider">
                <FileCode className="w-3 h-3" />
                File Header
              </div>
              <div className="flex flex-wrap gap-x-6 gap-y-1 text-sm">
                {headerSize && (
                  <div className="flex items-center gap-2">
                    <span className="text-gray-500">Header Size</span>
                    <span className="text-gray-300 font-mono">{formatNumber(headerSize)} bytes</span>
                  </div>
                )}
                {headerCrc && (
                  <div className="flex items-center gap-2">
                    <span className="text-gray-500">Header CRC</span>
                    <span className="text-gray-300 font-mono">{formatHex(headerCrc)}</span>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
