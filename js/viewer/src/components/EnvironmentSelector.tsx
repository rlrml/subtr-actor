import { useState, useEffect, useCallback } from 'react';
import { ChevronDown, Loader2, Sun, Check } from 'lucide-react';
import { environmentApi } from '../services/environment.api';
import type { EnvironmentListItem } from '../types/environment';

interface EnvironmentSelectorProps {
  currentEnvironmentId: string | null;
  onEnvironmentChange: (environmentId: string) => void;
  disabled?: boolean;
}

/**
 * EnvironmentSelector - Dropdown component for selecting environments
 * Used in the settings panel to allow users to switch environments
 */
export function EnvironmentSelector({
  currentEnvironmentId,
  onEnvironmentChange,
  disabled = false,
}: EnvironmentSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [environments, setEnvironments] = useState<EnvironmentListItem[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load environments list
  const loadEnvironments = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await environmentApi.list();
      setEnvironments(response.environments || []);
    } catch (err) {
      console.error('Failed to load environments:', err);
      setError('Failed to load environments');
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Load environments on mount
  useEffect(() => {
    loadEnvironments();
  }, [loadEnvironments]);

  // Find current environment name
  const currentEnvironment = environments.find(env => env.id === currentEnvironmentId);

  // Handle selection
  const handleSelect = (envId: string) => {
    if (envId !== currentEnvironmentId) {
      onEnvironmentChange(envId);
    }
    setIsOpen(false);
  };

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (!target.closest('.environment-selector')) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('click', handleClickOutside);
      return () => document.removeEventListener('click', handleClickOutside);
    }
  }, [isOpen]);

  if (error) {
    return (
      <div className="text-xs text-red-400 py-2">
        {error}
        <button
          onClick={loadEnvironments}
          className="ml-2 text-blue-400 hover:text-blue-300 underline"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="environment-selector relative">
      <label className="text-xs text-gray-400 font-medium mb-1 block">
        Environment
      </label>

      {/* Dropdown Trigger */}
      <button
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled || isLoading}
        className={`w-full flex items-center justify-between gap-2 px-3 py-2 rounded border transition-colors text-left ${
          disabled
            ? 'bg-gray-800 border-gray-700 text-gray-500 cursor-not-allowed'
            : isOpen
            ? 'bg-gray-700 border-amber-500 text-white'
            : 'bg-gray-800 border-gray-600 text-white hover:border-gray-500'
        }`}
      >
        <div className="flex items-center gap-2 min-w-0">
          {isLoading ? (
            <Loader2 size={14} className="animate-spin text-gray-400" />
          ) : (
            <Sun size={14} className="text-amber-400 flex-shrink-0" />
          )}
          <span className="text-xs truncate">
            {isLoading
              ? 'Loading...'
              : currentEnvironment
              ? currentEnvironment.name
              : environments.length > 0
              ? 'Select environment'
              : 'No environments available'}
          </span>
        </div>
        <ChevronDown
          size={14}
          className={`text-gray-400 flex-shrink-0 transition-transform ${isOpen ? 'rotate-180' : ''}`}
        />
      </button>

      {/* Dropdown Menu */}
      {isOpen && environments.length > 0 && (
        <div className="absolute z-50 w-full mt-1 bg-gray-800 border border-gray-600 rounded shadow-lg max-h-[200px] overflow-y-auto">
          {environments.map((env) => (
            <button
              key={env.id}
              onClick={() => handleSelect(env.id)}
              className={`w-full flex items-center gap-2 px-3 py-2 text-left transition-colors hover:bg-gray-700 ${
                env.id === currentEnvironmentId ? 'bg-amber-900/30' : ''
              }`}
            >
              <Sun size={12} className="text-amber-400 flex-shrink-0" />
              <div className="flex-1 min-w-0">
                <div className="text-xs text-white truncate">
                  {env.name}
                  {env.isDefault && (
                    <span className="ml-1 text-[9px] text-yellow-400">(default)</span>
                  )}
                </div>
                <div className="text-[10px] text-gray-500">
                  {env.meshCount} mesh{env.meshCount !== 1 ? 'es' : ''} · {env.lightCount} light{env.lightCount !== 1 ? 's' : ''}
                </div>
              </div>
              {env.id === currentEnvironmentId && (
                <Check size={12} className="text-amber-400 flex-shrink-0" />
              )}
            </button>
          ))}
        </div>
      )}

      {/* Empty State */}
      {isOpen && environments.length === 0 && !isLoading && (
        <div className="absolute z-50 w-full mt-1 bg-gray-800 border border-gray-600 rounded p-3 text-center">
          <div className="text-xs text-gray-500">No custom environments</div>
          <div className="text-[10px] text-gray-600 mt-1">
            Create one in the DevTools panel
          </div>
        </div>
      )}
    </div>
  );
}

// Simple inline version for compact display
interface EnvironmentSelectorInlineProps {
  currentEnvironmentId: string | null;
  onEnvironmentChange: (environmentId: string) => void;
  isLoading?: boolean;
}

export function EnvironmentSelectorInline({
  currentEnvironmentId,
  onEnvironmentChange,
  isLoading = false,
}: EnvironmentSelectorInlineProps) {
  const [environments, setEnvironments] = useState<EnvironmentListItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function loadEnvs() {
      try {
        const response = await environmentApi.list();
        setEnvironments(response.environments || []);
      } catch (err) {
        console.error('Failed to load environments:', err);
      } finally {
        setLoading(false);
      }
    }
    loadEnvs();
  }, []);

  if (loading || environments.length === 0) {
    return null;
  }

  return (
    <select
      value={currentEnvironmentId || ''}
      onChange={(e) => e.target.value && onEnvironmentChange(e.target.value)}
      disabled={isLoading}
      className="px-2 py-1 bg-gray-800 text-white text-xs rounded border border-gray-600 focus:border-amber-500 focus:outline-none disabled:opacity-50"
    >
      <option value="">Default Environment</option>
      {environments.map((env) => (
        <option key={env.id} value={env.id}>
          {env.name} {env.isDefault ? '(default)' : ''}
        </option>
      ))}
    </select>
  );
}
