/**
 * DeviceAuth - Device Authorization Page
 * Allows users to authorize desktop/TV apps using a code
 */

import { useState, useEffect } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { Smartphone, CheckCircle, XCircle, Loader2 } from 'lucide-react';
import { api } from '@/services/api';
import { useAuth } from '@/hooks/useAuth';

type AuthState = 'input' | 'authorizing' | 'success' | 'error';

export default function DeviceAuth() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { user, isLoading: authLoading } = useAuth();

  const [code, setCode] = useState('');
  const [state, setState] = useState<AuthState>('input');
  const [error, setError] = useState<string | null>(null);
  const [deviceName, setDeviceName] = useState<string | null>(null);

  // Pre-fill code from URL if provided
  useEffect(() => {
    const urlCode = searchParams.get('code');
    if (urlCode) {
      setCode(urlCode.toUpperCase());
    }
  }, [searchParams]);

  // Redirect to login if not authenticated
  useEffect(() => {
    if (!authLoading && !user) {
      // Save current URL to redirect back after login
      const returnUrl = `/device${searchParams.toString() ? `?${searchParams.toString()}` : ''}`;
      navigate(`/login?returnUrl=${encodeURIComponent(returnUrl)}`);
    }
  }, [user, authLoading, navigate, searchParams]);

  const formatCode = (value: string) => {
    // Remove non-alphanumeric characters and uppercase
    const cleaned = value.replace(/[^A-Za-z0-9]/g, '').toUpperCase();
    // Add dash after 4 characters
    if (cleaned.length > 4) {
      return `${cleaned.slice(0, 4)}-${cleaned.slice(4, 8)}`;
    }
    return cleaned;
  };

  const handleCodeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const formatted = formatCode(e.target.value);
    setCode(formatted);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (code.replace(/-/g, '').length !== 8) {
      setError('Please enter a valid 8-character code');
      return;
    }

    setState('authorizing');
    setError(null);

    try {
      const response = await api.post<{ success: boolean; deviceName: string | null } | { error: string; error_description: string }>(
        '/auth/device/authorize',
        { user_code: code }
      );

      if ('error' in response) {
        setState('error');
        setError(response.error_description || 'Failed to authorize device');
      } else {
        setState('success');
        setDeviceName(response.deviceName);
      }
    } catch (err) {
      setState('error');
      setError(err instanceof Error ? err.message : 'Failed to authorize device');
    }
  };

  if (authLoading) {
    return (
      <div className="flex items-center justify-center min-h-[50vh]">
        <Loader2 className="w-8 h-8 animate-spin text-violet-500" />
      </div>
    );
  }

  return (
    <div className="max-w-md mx-auto py-12 px-4">
      <div className="bg-gray-900 rounded-xl p-8 border border-gray-800">
        {/* Header */}
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-violet-500/20 mb-4">
            <Smartphone className="w-8 h-8 text-violet-400" />
          </div>
          <h1 className="text-2xl font-bold text-white mb-2">
            Connect a Device
          </h1>
          <p className="text-gray-400">
            Enter the code displayed on your device to link it to your account
          </p>
        </div>

        {/* Input State */}
        {state === 'input' && (
          <form onSubmit={handleSubmit} className="space-y-6">
            <div>
              <label htmlFor="code" className="block text-sm font-medium text-gray-300 mb-2">
                Device Code
              </label>
              <input
                type="text"
                id="code"
                value={code}
                onChange={handleCodeChange}
                placeholder="XXXX-XXXX"
                maxLength={9}
                autoComplete="off"
                autoFocus
                className="w-full px-4 py-3 text-center text-2xl font-mono tracking-widest bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-violet-500 focus:border-transparent"
              />
              {error && (
                <p className="mt-2 text-sm text-red-400">{error}</p>
              )}
            </div>

            <button
              type="submit"
              disabled={code.replace(/-/g, '').length !== 8}
              className="w-full py-3 px-4 bg-violet-600 hover:bg-violet-500 disabled:bg-gray-700 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-colors"
            >
              Authorize Device
            </button>
          </form>
        )}

        {/* Authorizing State */}
        {state === 'authorizing' && (
          <div className="text-center py-8">
            <Loader2 className="w-12 h-12 animate-spin text-violet-500 mx-auto mb-4" />
            <p className="text-gray-400">Authorizing device...</p>
          </div>
        )}

        {/* Success State */}
        {state === 'success' && (
          <div className="text-center py-8">
            <CheckCircle className="w-16 h-16 text-green-500 mx-auto mb-4" />
            <h2 className="text-xl font-semibold text-white mb-2">
              Device Authorized!
            </h2>
            <p className="text-gray-400 mb-6">
              {deviceName ? (
                <>Your device <span className="text-white font-medium">{deviceName}</span> has been connected.</>
              ) : (
                <>Your device has been successfully connected to your account.</>
              )}
            </p>
            <p className="text-sm text-gray-500">
              You can now close this page and return to your device.
            </p>
          </div>
        )}

        {/* Error State */}
        {state === 'error' && (
          <div className="text-center py-8">
            <XCircle className="w-16 h-16 text-red-500 mx-auto mb-4" />
            <h2 className="text-xl font-semibold text-white mb-2">
              Authorization Failed
            </h2>
            <p className="text-gray-400 mb-6">
              {error || 'The code is invalid or has expired. Please try again.'}
            </p>
            <button
              onClick={() => {
                setState('input');
                setCode('');
                setError(null);
              }}
              className="px-6 py-2 bg-gray-800 hover:bg-gray-700 text-white rounded-lg transition-colors"
            >
              Try Again
            </button>
          </div>
        )}

        {/* Help text */}
        {state === 'input' && (
          <div className="mt-8 pt-6 border-t border-gray-800">
            <p className="text-xs text-gray-500 text-center">
              The code is displayed on your desktop app. It expires after 15 minutes.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
