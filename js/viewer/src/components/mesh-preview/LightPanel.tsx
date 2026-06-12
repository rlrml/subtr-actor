import { useState, useCallback } from 'react';
import * as THREE from 'three';
import { MeshPreviewEngine } from './MeshPreviewEngine';
import { Button } from '@/components/ui/button';
import { Slider } from '@/components/ui/slider';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Plus, Trash2, Sun, Lightbulb, Cone, Circle } from 'lucide-react';

interface LightInfo {
  id: string;
  type: 'ambient' | 'directional' | 'point' | 'spot';
  name: string;
  color: string;
  intensity: number;
  enabled: boolean;
  light: THREE.Light;
  helper: THREE.Object3D | null;
}

interface LightPanelProps {
  engine: MeshPreviewEngine | null;
}

export function LightPanel({ engine }: LightPanelProps) {
  const [lights, setLights] = useState<LightInfo[]>([]);
  const [selectedLightId, setSelectedLightId] = useState<string | null>(null);
  const [newLightType, setNewLightType] = useState<'ambient' | 'directional' | 'point' | 'spot'>('directional');

  const addLight = useCallback(() => {
    if (!engine) return;

    const scene = engine.getScene();
    const id = `light_${Date.now()}`;
    let light: THREE.Light;
    let helper: THREE.Object3D | null = null;

    switch (newLightType) {
      case 'ambient':
        light = new THREE.AmbientLight(0xffffff, 0.5);
        break;
      case 'directional':
        light = new THREE.DirectionalLight(0xffffff, 1);
        (light as THREE.DirectionalLight).position.set(100, 200, 100);
        helper = new THREE.DirectionalLightHelper(light as THREE.DirectionalLight, 20);
        break;
      case 'point':
        light = new THREE.PointLight(0xffffff, 1, 500);
        (light as THREE.PointLight).position.set(0, 100, 0);
        helper = new THREE.PointLightHelper(light as THREE.PointLight, 10);
        break;
      case 'spot':
        light = new THREE.SpotLight(0xffffff, 1);
        (light as THREE.SpotLight).position.set(0, 200, 0);
        (light as THREE.SpotLight).angle = Math.PI / 6;
        helper = new THREE.SpotLightHelper(light as THREE.SpotLight);
        break;
    }

    scene.add(light);
    if (helper) scene.add(helper);

    const newLight: LightInfo = {
      id,
      type: newLightType,
      name: `${newLightType.charAt(0).toUpperCase() + newLightType.slice(1)} ${lights.length + 1}`,
      color: '#ffffff',
      intensity: newLightType === 'ambient' ? 0.5 : 1,
      enabled: true,
      light,
      helper,
    };

    setLights([...lights, newLight]);
    setSelectedLightId(id);
  }, [engine, newLightType, lights]);

  const removeLight = useCallback(
    (id: string) => {
      if (!engine) return;

      const scene = engine.getScene();
      const lightInfo = lights.find((l) => l.id === id);
      if (lightInfo) {
        scene.remove(lightInfo.light);
        if (lightInfo.helper) scene.remove(lightInfo.helper);
      }

      setLights(lights.filter((l) => l.id !== id));
      if (selectedLightId === id) setSelectedLightId(null);
    },
    [engine, lights, selectedLightId]
  );

  const updateLight = useCallback(
    (id: string, property: string, value: unknown) => {
      setLights(
        lights.map((l) => {
          if (l.id !== id) return l;

          switch (property) {
            case 'color':
              l.light.color.set(value as string);
              l.color = value as string;
              break;
            case 'intensity':
              l.light.intensity = value as number;
              l.intensity = value as number;
              break;
            case 'enabled':
              l.light.visible = value as boolean;
              if (l.helper) l.helper.visible = value as boolean;
              l.enabled = value as boolean;
              break;
          }

          if (l.helper && 'update' in l.helper) {
            (l.helper as THREE.DirectionalLightHelper).update();
          }

          return { ...l };
        })
      );
    },
    [lights]
  );

  const selectedLight = lights.find((l) => l.id === selectedLightId);

  const getLightIcon = (type: string) => {
    switch (type) {
      case 'ambient':
        return <Circle className="h-4 w-4" />;
      case 'directional':
        return <Sun className="h-4 w-4" />;
      case 'point':
        return <Lightbulb className="h-4 w-4" />;
      case 'spot':
        return <Cone className="h-4 w-4" />;
      default:
        return <Lightbulb className="h-4 w-4" />;
    }
  };

  return (
    <div className="space-y-4">
      {/* Add Light */}
      <div className="flex gap-2">
        <Select value={newLightType} onValueChange={(v) => setNewLightType(v as typeof newLightType)}>
          <SelectTrigger className="flex-1">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="ambient">Ambient</SelectItem>
            <SelectItem value="directional">Directional</SelectItem>
            <SelectItem value="point">Point</SelectItem>
            <SelectItem value="spot">Spot</SelectItem>
          </SelectContent>
        </Select>
        <Button onClick={addLight} size="sm">
          <Plus className="h-4 w-4 mr-1" />
          Add
        </Button>
      </div>

      {/* Light List */}
      {lights.length === 0 ? (
        <div className="text-center py-8 text-zinc-500">
          <p>No custom lights</p>
          <p className="text-sm mt-1">Add a light to customize scene lighting</p>
        </div>
      ) : (
        <div className="space-y-1">
          {lights.map((light) => (
            <div
              key={light.id}
              className={`flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer ${
                selectedLightId === light.id
                  ? 'bg-violet-600 text-white'
                  : 'hover:bg-zinc-800 text-zinc-300'
              }`}
              onClick={() => setSelectedLightId(light.id)}
            >
              {getLightIcon(light.type)}
              <span className="flex-1 truncate">{light.name}</span>
              <Switch
                checked={light.enabled}
                onCheckedChange={(v) => updateLight(light.id, 'enabled', v)}
                onClick={(e) => e.stopPropagation()}
              />
              <Button
                variant="ghost"
                size="sm"
                className="h-6 w-6 p-0"
                onClick={(e) => {
                  e.stopPropagation();
                  removeLight(light.id);
                }}
              >
                <Trash2 className="h-3 w-3" />
              </Button>
            </div>
          ))}
        </div>
      )}

      {/* Light Editor */}
      {selectedLight && (
        <div className="space-y-4 pt-4 border-t border-zinc-800">
          <Label className="text-zinc-300 font-medium">{selectedLight.name}</Label>

          {/* Color */}
          <div className="space-y-2">
            <Label className="text-zinc-400 text-xs">Color</Label>
            <Input
              type="color"
              value={selectedLight.color}
              onChange={(e) => updateLight(selectedLight.id, 'color', e.target.value)}
              className="h-8 w-full cursor-pointer"
            />
          </div>

          {/* Intensity */}
          <div className="space-y-2">
            <div className="flex justify-between">
              <Label className="text-zinc-400 text-xs">Intensity</Label>
              <span className="text-xs text-zinc-500">{selectedLight.intensity.toFixed(2)}</span>
            </div>
            <Slider
              value={[selectedLight.intensity]}
              min={0}
              max={selectedLight.type === 'ambient' ? 2 : 5}
              step={0.1}
              onValueChange={([v]) => updateLight(selectedLight.id, 'intensity', v)}
            />
          </div>
        </div>
      )}
    </div>
  );
}
