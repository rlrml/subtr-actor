import { useState, useEffect } from 'react';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Loader2 } from 'lucide-react';
import { MeshPreviewEngine } from './MeshPreviewEngine';

const API_URL = import.meta.env.VITE_API_URL || '/api';

interface Environment {
  id: string;
  name: string;
}

interface EnvironmentSelectorProps {
  engine: MeshPreviewEngine | null;
}

export function EnvironmentSelector({ engine }: EnvironmentSelectorProps) {
  const [environments, setEnvironments] = useState<Environment[]>([]);
  const [selectedId, setSelectedId] = useState<string>('default');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchEnvironments = async () => {
      try {
        setLoading(true);
        const response = await fetch(`${API_URL}/environments`);
        if (!response.ok) {
          throw new Error('Failed to fetch environments');
        }
        const data = await response.json();
        // Handle both array response and object with environments property
        const envList = Array.isArray(data) ? data : (data.environments || data.data || []);
        setEnvironments(envList);
        setError(null);
      } catch (err) {
        console.warn('Could not load environments:', err);
        setError('Unable to load environments');
        setEnvironments([]);
      } finally {
        setLoading(false);
      }
    };

    fetchEnvironments();
  }, []);

  const handleChange = async (value: string) => {
    if (!engine) return;

    setSelectedId(value);

    if (value === 'default') {
      engine.loadDefaultEnvironment();
    } else {
      try {
        await engine.loadEnvironment(value);
      } catch (err) {
        console.error('Failed to load environment:', err);
        // Fallback to default
        engine.loadDefaultEnvironment();
        setSelectedId('default');
      }
    }
  };

  if (loading) {
    return (
      <div className="space-y-2">
        <Label className="text-zinc-300">Environment</Label>
        <div className="flex items-center gap-2 text-zinc-500 text-sm">
          <Loader2 className="h-4 w-4 animate-spin" />
          Loading environments...
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <Label className="text-zinc-300">Environment</Label>
      <Select value={selectedId} onValueChange={handleChange}>
        <SelectTrigger className="w-full">
          <SelectValue placeholder="Select environment" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="default">Default (Neutral Studio)</SelectItem>
          {environments.map((env) => (
            <SelectItem key={env.id} value={env.id}>
              {env.name}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {error && (
        <p className="text-xs text-amber-500">{error}</p>
      )}
      <p className="text-xs text-zinc-500">
        Loads skybox and IBL lighting only (no arena)
      </p>
    </div>
  );
}
