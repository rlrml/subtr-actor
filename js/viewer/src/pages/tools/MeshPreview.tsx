import { useEffect, useRef, useState, useCallback } from 'react';
import { MeshPreviewEngine, MaterialInfo, MeshNode } from '@/components/mesh-preview/MeshPreviewEngine';
import { MeshUploader } from '@/components/mesh-preview/MeshUploader';
import { MaterialPanel } from '@/components/mesh-preview/MaterialPanel';
import { TransformPanel } from '@/components/mesh-preview/TransformPanel';
import { HierarchyPanel } from '@/components/mesh-preview/HierarchyPanel';
import { LightPanel } from '@/components/mesh-preview/LightPanel';
import { EnvironmentSelector } from '@/components/mesh-preview/EnvironmentSelector';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useToast } from '@/hooks/use-toast';
import { HelpCircle } from 'lucide-react';

export default function MeshPreview() {
  const containerRef = useRef<HTMLDivElement>(null);
  const engineRef = useRef<MeshPreviewEngine | null>(null);
  const { toast } = useToast();

  // State
  const [isLoading, setIsLoading] = useState(false);
  const [loadProgress, setLoadProgress] = useState(0);
  const [loadedFileName, setLoadedFileName] = useState<string | null>(null);
  const [materials, setMaterials] = useState<MaterialInfo[]>([]);
  const [hierarchy, setHierarchy] = useState<MeshNode[]>([]);
  const [selectedMaterialId, setSelectedMaterialId] = useState<string | null>(null);
  const [transformMode, setTransformMode] = useState<'translate' | 'rotate' | 'scale'>('translate');
  const [showHelp, setShowHelp] = useState(false);

  // Initialize engine
  useEffect(() => {
    if (!containerRef.current) return;

    const engine = new MeshPreviewEngine(containerRef.current);
    engineRef.current = engine;

    // Setup callbacks
    engine.onMaterialsUpdate = (mats) => setMaterials(mats);
    engine.onHierarchyUpdate = (nodes) => setHierarchy(nodes);
    engine.onMaterialSelect = (materialId) => setSelectedMaterialId(materialId);

    return () => {
      engine.dispose();
      engineRef.current = null;
    };
  }, []);

  // Handle file upload
  const handleFileUpload = useCallback(async (file: File) => {
    if (!engineRef.current) return;

    setIsLoading(true);
    setLoadProgress(0);

    try {
      await engineRef.current.loadMesh(file, (progress) => {
        setLoadProgress(progress);
      });
      // Attach transform controls to the loaded mesh
      engineRef.current.attachTransformControls();
      setLoadedFileName(file.name);
      toast({
        title: 'Mesh loaded',
        description: `Successfully loaded ${file.name}`,
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to load mesh';
      toast({
        title: 'Error loading mesh',
        description: message,
        variant: 'destructive',
      });
      setLoadedFileName(null);
    } finally {
      setIsLoading(false);
      setLoadProgress(0);
    }
  }, [toast]);

  // Handle material property change
  const handleMaterialChange = useCallback((materialId: string, property: string, value: unknown) => {
    engineRef.current?.updateMaterial(materialId, property, value);
    // Refresh materials list to update UI
    const updatedMaterials = engineRef.current?.getMaterials() || [];
    setMaterials([...updatedMaterials]);
  }, []);

  // Handle material reset
  const handleMaterialReset = useCallback((materialId: string) => {
    engineRef.current?.resetMaterial(materialId);
  }, []);

  // Handle reset all materials
  const handleResetAllMaterials = useCallback(() => {
    engineRef.current?.resetAllMaterials();
  }, []);

  // Handle focus on mesh
  const handleFocus = useCallback(() => {
    const mesh = engineRef.current?.getLoadedMesh();
    if (mesh) {
      engineRef.current?.focusOnObject(mesh);
    }
  }, []);

  // Sync transform mode with engine
  useEffect(() => {
    engineRef.current?.setTransformMode(transformMode);
  }, [transformMode]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore when typing in input fields
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

      switch (e.key.toLowerCase()) {
        case 'g':
          setTransformMode('translate');
          break;
        case 'r':
          setTransformMode('rotate');
          break;
        case 's':
          setTransformMode('scale');
          break;
        case 'f':
          handleFocus();
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleFocus]);

  return (
    <div className="flex h-screen w-full bg-zinc-950 overflow-hidden">
      {/* 3D Canvas */}
      <div className="flex-1 min-w-0 relative">
        <div ref={containerRef} className="w-full h-full" />

        {/* Upload prompt when no mesh loaded or loading */}
        {(!loadedFileName || isLoading) && (
          <div className="absolute inset-0 flex items-center justify-center pointer-events-none z-10">
            <div className="pointer-events-auto">
              <MeshUploader
                onFileSelect={handleFileUpload}
                isLoading={isLoading}
                progress={loadProgress}
              />
            </div>
          </div>
        )}

        {/* Loaded file indicator */}
        {loadedFileName && (
          <div className="absolute top-4 left-4 bg-zinc-800/90 backdrop-blur-sm rounded-lg px-3 py-2 text-sm text-zinc-300 z-10">
            {loadedFileName}
          </div>
        )}

        {/* Help indicator */}
        <div className="absolute bottom-4 left-4 z-10">
          <button
            onClick={() => setShowHelp(!showHelp)}
            className="flex items-center gap-2 bg-zinc-800/90 backdrop-blur-sm rounded-lg px-3 py-2 text-sm text-zinc-300 hover:bg-zinc-700/90 transition-colors"
          >
            <HelpCircle className="w-4 h-4" />
            {showHelp ? 'Hide' : 'Controls'}
          </button>

          {showHelp && (
            <div className="mt-2 bg-zinc-800/95 backdrop-blur-sm rounded-lg p-4 text-xs text-zinc-300 min-w-[200px]">
              <h3 className="font-semibold text-zinc-100 mb-3">Keyboard Shortcuts</h3>
              <div className="space-y-1.5">
                <div className="flex justify-between">
                  <span className="text-zinc-400">Move</span>
                  <kbd className="bg-zinc-700 px-1.5 py-0.5 rounded">G</kbd>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-400">Rotate</span>
                  <kbd className="bg-zinc-700 px-1.5 py-0.5 rounded">R</kbd>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-400">Scale</span>
                  <kbd className="bg-zinc-700 px-1.5 py-0.5 rounded">S</kbd>
                </div>
                <div className="flex justify-between">
                  <span className="text-zinc-400">Focus</span>
                  <kbd className="bg-zinc-700 px-1.5 py-0.5 rounded">F</kbd>
                </div>
              </div>

              <h3 className="font-semibold text-zinc-100 mt-4 mb-3">Mouse Controls</h3>
              <div className="space-y-1.5">
                <div className="flex justify-between gap-4">
                  <span className="text-zinc-400">Orbit</span>
                  <span className="text-zinc-500">Left click + drag</span>
                </div>
                <div className="flex justify-between gap-4">
                  <span className="text-zinc-400">Pan</span>
                  <span className="text-zinc-500">Right click + drag</span>
                </div>
                <div className="flex justify-between gap-4">
                  <span className="text-zinc-400">Zoom</span>
                  <span className="text-zinc-500">Scroll wheel</span>
                </div>
                <div className="flex justify-between gap-4">
                  <span className="text-zinc-400">Select material</span>
                  <span className="text-zinc-500">Click on mesh</span>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Right Sidebar */}
      <div className="w-80 flex-shrink-0 bg-zinc-900 border-l border-zinc-800 flex flex-col">
        {/* Upload section */}
        {loadedFileName && (
          <div className="p-4 border-b border-zinc-800">
            <MeshUploader
              onFileSelect={handleFileUpload}
              compact
              currentFile={loadedFileName}
            />
          </div>
        )}

        {/* Tabs */}
        <Tabs defaultValue="materials" className="flex-1 flex flex-col">
          <TabsList className="flex w-full justify-start rounded-none border-b border-zinc-800 bg-transparent px-1 flex-wrap h-auto py-1 gap-1">
            <TabsTrigger value="materials" className="data-[state=active]:bg-zinc-800 text-xs px-2 py-1">
              Mat
            </TabsTrigger>
            <TabsTrigger value="transform" className="data-[state=active]:bg-zinc-800 text-xs px-2 py-1">
              Trans
            </TabsTrigger>
            <TabsTrigger value="hierarchy" className="data-[state=active]:bg-zinc-800 text-xs px-2 py-1">
              Hier
            </TabsTrigger>
            <TabsTrigger value="lights" className="data-[state=active]:bg-zinc-800 text-xs px-2 py-1">
              Lights
            </TabsTrigger>
            <TabsTrigger value="env" className="data-[state=active]:bg-zinc-800 text-xs px-2 py-1">
              Env
            </TabsTrigger>
          </TabsList>

          <div className="flex-1 overflow-y-auto">
            <TabsContent value="materials" className="m-0 p-4">
              <MaterialPanel
                materials={materials}
                selectedMaterialId={selectedMaterialId}
                onSelectMaterial={setSelectedMaterialId}
                onMaterialChange={handleMaterialChange}
                onResetMaterial={handleMaterialReset}
                onResetAll={handleResetAllMaterials}
              />
            </TabsContent>

            <TabsContent value="transform" className="m-0 p-4">
              <TransformPanel
                engine={engineRef.current}
                mode={transformMode}
                onModeChange={setTransformMode}
                onFocus={handleFocus}
              />
            </TabsContent>

            <TabsContent value="hierarchy" className="m-0 p-4">
              <HierarchyPanel
                hierarchy={hierarchy}
                materials={materials}
                engine={engineRef.current}
              />
            </TabsContent>

            <TabsContent value="lights" className="m-0 p-4">
              <LightPanel engine={engineRef.current} />
            </TabsContent>

            <TabsContent value="env" className="m-0 p-4">
              <EnvironmentSelector engine={engineRef.current} />
            </TabsContent>
          </div>
        </Tabs>
      </div>
    </div>
  );
}
