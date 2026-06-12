import { useEffect, useState } from 'react';
import { MeshPreviewEngine } from './MeshPreviewEngine';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Move, RotateCw, Maximize2, Focus, RotateCcw } from 'lucide-react';
import { cn } from '@/lib/utils';

interface TransformPanelProps {
  engine: MeshPreviewEngine | null;
  mode: 'translate' | 'rotate' | 'scale';
  onModeChange: (mode: 'translate' | 'rotate' | 'scale') => void;
  onFocus: () => void;
}

export function TransformPanel({ engine, mode, onModeChange, onFocus }: TransformPanelProps) {
  const [position, setPosition] = useState({ x: 0, y: 0, z: 0 });
  const [rotation, setRotation] = useState({ x: 0, y: 0, z: 0 });
  const [scale, setScale] = useState({ x: 1, y: 1, z: 1 });

  // Update transform values from mesh
  useEffect(() => {
    if (!engine) return;

    const updateTransform = () => {
      const mesh = engine.getLoadedMesh();
      if (mesh) {
        setPosition({
          x: mesh.position.x,
          y: mesh.position.y,
          z: mesh.position.z,
        });
        setRotation({
          x: (mesh.rotation.x * 180) / Math.PI,
          y: (mesh.rotation.y * 180) / Math.PI,
          z: (mesh.rotation.z * 180) / Math.PI,
        });
        setScale({
          x: mesh.scale.x,
          y: mesh.scale.y,
          z: mesh.scale.z,
        });
      }
    };

    const interval = setInterval(updateTransform, 100);
    return () => clearInterval(interval);
  }, [engine]);

  const handleReset = () => {
    const mesh = engine?.getLoadedMesh();
    if (mesh) {
      mesh.position.set(0, 0, 0);
      mesh.rotation.set(0, 0, 0);
      mesh.scale.set(1, 1, 1);
    }
  };

  const mesh = engine?.getLoadedMesh();

  if (!mesh) {
    return (
      <div className="text-center py-8 text-zinc-500">
        <p>No mesh loaded</p>
        <p className="text-sm mt-1">Upload a mesh to transform it</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Mode buttons */}
      <div className="flex gap-2">
        <Button
          variant={mode === 'translate' ? 'default' : 'outline'}
          size="sm"
          onClick={() => onModeChange('translate')}
          className={cn('flex-1', mode === 'translate' && 'bg-violet-600')}
        >
          <Move className="h-4 w-4 mr-1" />
          Move (G)
        </Button>
        <Button
          variant={mode === 'rotate' ? 'default' : 'outline'}
          size="sm"
          onClick={() => onModeChange('rotate')}
          className={cn('flex-1', mode === 'rotate' && 'bg-violet-600')}
        >
          <RotateCw className="h-4 w-4 mr-1" />
          Rotate (R)
        </Button>
        <Button
          variant={mode === 'scale' ? 'default' : 'outline'}
          size="sm"
          onClick={() => onModeChange('scale')}
          className={cn('flex-1', mode === 'scale' && 'bg-violet-600')}
        >
          <Maximize2 className="h-4 w-4 mr-1" />
          Scale (S)
        </Button>
      </div>

      {/* Position */}
      <div className="space-y-2">
        <Label className="text-zinc-400 text-xs">Position</Label>
        <div className="grid grid-cols-3 gap-2 text-sm">
          <div className="bg-zinc-800 rounded px-2 py-1">
            <span className="text-red-400">X:</span> {position.x.toFixed(1)}
          </div>
          <div className="bg-zinc-800 rounded px-2 py-1">
            <span className="text-green-400">Y:</span> {position.y.toFixed(1)}
          </div>
          <div className="bg-zinc-800 rounded px-2 py-1">
            <span className="text-blue-400">Z:</span> {position.z.toFixed(1)}
          </div>
        </div>
      </div>

      {/* Rotation */}
      <div className="space-y-2">
        <Label className="text-zinc-400 text-xs">Rotation (degrees)</Label>
        <div className="grid grid-cols-3 gap-2 text-sm">
          <div className="bg-zinc-800 rounded px-2 py-1">
            <span className="text-red-400">X:</span> {rotation.x.toFixed(1)}°
          </div>
          <div className="bg-zinc-800 rounded px-2 py-1">
            <span className="text-green-400">Y:</span> {rotation.y.toFixed(1)}°
          </div>
          <div className="bg-zinc-800 rounded px-2 py-1">
            <span className="text-blue-400">Z:</span> {rotation.z.toFixed(1)}°
          </div>
        </div>
      </div>

      {/* Scale */}
      <div className="space-y-2">
        <Label className="text-zinc-400 text-xs">Scale</Label>
        <div className="grid grid-cols-3 gap-2 text-sm">
          <div className="bg-zinc-800 rounded px-2 py-1">
            <span className="text-red-400">X:</span> {scale.x.toFixed(2)}
          </div>
          <div className="bg-zinc-800 rounded px-2 py-1">
            <span className="text-green-400">Y:</span> {scale.y.toFixed(2)}
          </div>
          <div className="bg-zinc-800 rounded px-2 py-1">
            <span className="text-blue-400">Z:</span> {scale.z.toFixed(2)}
          </div>
        </div>
      </div>

      {/* Actions */}
      <div className="flex gap-2 pt-2">
        <Button variant="outline" size="sm" onClick={onFocus} className="flex-1">
          <Focus className="h-4 w-4 mr-1" />
          Focus (F)
        </Button>
        <Button variant="outline" size="sm" onClick={handleReset} className="flex-1">
          <RotateCcw className="h-4 w-4 mr-1" />
          Reset
        </Button>
      </div>

      <p className="text-xs text-zinc-500 text-center">
        Tip: Use G/R/S keys to switch modes, F to focus
      </p>
    </div>
  );
}
