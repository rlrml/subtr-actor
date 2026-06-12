import { MaterialInfo } from './MeshPreviewEngine';
import { Button } from '@/components/ui/button';
import { Slider } from '@/components/ui/slider';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { RotateCcw } from 'lucide-react';
import { cn } from '@/lib/utils';

interface MaterialPanelProps {
  materials: MaterialInfo[];
  selectedMaterialId: string | null;
  onSelectMaterial: (id: string | null) => void;
  onMaterialChange: (materialId: string, property: string, value: unknown) => void;
  onResetMaterial: (materialId: string) => void;
  onResetAll: () => void;
}

export function MaterialPanel({
  materials,
  selectedMaterialId,
  onSelectMaterial,
  onMaterialChange,
  onResetMaterial,
  onResetAll,
}: MaterialPanelProps) {
  const selectedMaterial = materials.find((m) => m.id === selectedMaterialId);

  if (materials.length === 0) {
    return (
      <div className="text-center py-8 text-zinc-500">
        <p>No materials found</p>
        <p className="text-sm mt-1">Upload a mesh to see its materials</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Material List */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <Label className="text-zinc-400">Materials ({materials.length})</Label>
          <Button variant="ghost" size="sm" onClick={onResetAll} className="h-7 text-xs">
            <RotateCcw className="h-3 w-3 mr-1" />
            Reset All
          </Button>
        </div>
        <div className="space-y-1 max-h-48 overflow-y-auto">
          {materials.map((mat) => (
            <button
              key={mat.id}
              onClick={() => onSelectMaterial(mat.id === selectedMaterialId ? null : mat.id)}
              className={cn(
                'w-full flex items-center gap-2 px-2 py-1.5 rounded text-sm text-left transition-colors',
                selectedMaterialId === mat.id
                  ? 'bg-violet-600 text-white'
                  : 'hover:bg-zinc-800 text-zinc-300'
              )}
            >
              <div
                className="w-4 h-4 rounded border border-zinc-600"
                style={{ backgroundColor: mat.original.color }}
              />
              <span className="truncate flex-1">{mat.name}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Material Editor */}
      {selectedMaterial && (
        <div className="space-y-4 pt-4 border-t border-zinc-800">
          <div className="flex items-center justify-between">
            <Label className="text-zinc-300 font-medium">{selectedMaterial.name}</Label>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onResetMaterial(selectedMaterial.id)}
              className="h-7 text-xs"
            >
              <RotateCcw className="h-3 w-3 mr-1" />
              Reset
            </Button>
          </div>

          {/* Color */}
          <div className="space-y-2">
            <Label className="text-zinc-400 text-xs">Base Color</Label>
            <Input
              type="color"
              value={'#' + selectedMaterial.material.color.getHexString()}
              onChange={(e) => onMaterialChange(selectedMaterial.id, 'color', e.target.value)}
              className="h-8 w-full cursor-pointer"
            />
          </div>

          {/* Roughness */}
          <div className="space-y-2">
            <div className="flex justify-between">
              <Label className="text-zinc-400 text-xs">Roughness</Label>
              <span className="text-xs text-zinc-500">
                {selectedMaterial.material.roughness.toFixed(2)}
              </span>
            </div>
            <Slider
              value={[selectedMaterial.material.roughness]}
              min={0}
              max={1}
              step={0.01}
              onValueChange={([v]) => onMaterialChange(selectedMaterial.id, 'roughness', v)}
            />
          </div>

          {/* Metalness */}
          <div className="space-y-2">
            <div className="flex justify-between">
              <Label className="text-zinc-400 text-xs">Metalness</Label>
              <span className="text-xs text-zinc-500">
                {selectedMaterial.material.metalness.toFixed(2)}
              </span>
            </div>
            <Slider
              value={[selectedMaterial.material.metalness]}
              min={0}
              max={1}
              step={0.01}
              onValueChange={([v]) => onMaterialChange(selectedMaterial.id, 'metalness', v)}
            />
          </div>

          {/* Emissive */}
          <div className="space-y-2">
            <Label className="text-zinc-400 text-xs">Emissive Color</Label>
            <Input
              type="color"
              value={'#' + selectedMaterial.material.emissive.getHexString()}
              onChange={(e) => onMaterialChange(selectedMaterial.id, 'emissive', e.target.value)}
              className="h-8 w-full cursor-pointer"
            />
          </div>

          {/* Emissive Intensity */}
          <div className="space-y-2">
            <div className="flex justify-between">
              <Label className="text-zinc-400 text-xs">Emissive Intensity</Label>
              <span className="text-xs text-zinc-500">
                {selectedMaterial.material.emissiveIntensity.toFixed(2)}
              </span>
            </div>
            <Slider
              value={[selectedMaterial.material.emissiveIntensity]}
              min={0}
              max={10}
              step={0.1}
              onValueChange={([v]) => onMaterialChange(selectedMaterial.id, 'emissiveIntensity', v)}
            />
          </div>

          {/* Opacity */}
          <div className="space-y-2">
            <div className="flex justify-between">
              <Label className="text-zinc-400 text-xs">Opacity</Label>
              <span className="text-xs text-zinc-500">
                {selectedMaterial.material.opacity.toFixed(2)}
              </span>
            </div>
            <Slider
              value={[selectedMaterial.material.opacity]}
              min={0}
              max={1}
              step={0.01}
              onValueChange={([v]) => onMaterialChange(selectedMaterial.id, 'opacity', v)}
            />
          </div>
        </div>
      )}
    </div>
  );
}
