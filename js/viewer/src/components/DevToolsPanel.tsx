import { useState, useEffect, useCallback, useRef, forwardRef } from 'react';
import { Slider } from '@/components/ui/slider';
import {
  Wrench,
  X,
  ChevronDown,
  ChevronRight,
  Lightbulb,
  Palette,
  Plus,
  Trash2,
  Move,
  MousePointer,
  RotateCcw,
  Eye,
  EyeOff,
  Copy,
  Check,
  Target,
  Cloud,
  Upload,
  Box,
  Save,
  FolderOpen,
  Loader2,
  Scale,
  RotateCw,
  FilePlus,
  Search,
  Pencil,
  Star,
  GripVertical,
  Mountain,
} from 'lucide-react';

// ============================================================================
// PANEL STORAGE
// ============================================================================

const PANEL_STORAGE_KEY = 'devtools-panel-state';

interface PanelState {
  x: number;
  y: number;
  width: number;
  height: number;
}

const DEFAULT_PANEL_STATE: PanelState = {
  x: window.innerWidth - 520, // Right side with margin
  y: 80,
  width: 500,
  height: 600,
};

const MIN_WIDTH = 380;
const MAX_WIDTH = 800;
const MIN_HEIGHT = 400;
const MAX_HEIGHT = window.innerHeight - 100;

function loadPanelState(): PanelState {
  try {
    const saved = localStorage.getItem(PANEL_STORAGE_KEY);
    if (saved) {
      const parsed = JSON.parse(saved);
      // Validate and clamp values
      return {
        x: Math.max(0, Math.min(parsed.x ?? DEFAULT_PANEL_STATE.x, window.innerWidth - 100)),
        y: Math.max(0, Math.min(parsed.y ?? DEFAULT_PANEL_STATE.y, window.innerHeight - 100)),
        width: Math.max(MIN_WIDTH, Math.min(parsed.width ?? DEFAULT_PANEL_STATE.width, MAX_WIDTH)),
        height: Math.max(MIN_HEIGHT, Math.min(parsed.height ?? DEFAULT_PANEL_STATE.height, MAX_HEIGHT)),
      };
    }
  } catch {
    // Ignore parse errors
  }
  return { ...DEFAULT_PANEL_STATE };
}

function savePanelState(state: PanelState): void {
  try {
    localStorage.setItem(PANEL_STORAGE_KEY, JSON.stringify(state));
  } catch {
    // Ignore storage errors
  }
}
import { assetApi } from '@/services/asset.api';
import { environmentApi } from '@/services/environment.api';
import type { Asset, AssetType, EnvironmentListItem, Environment, CreateMeshRequest, CreateLightRequest } from '@/types/environment';

// ============================================================================
// TYPES & INTERFACES
// ============================================================================

interface MaterialInfo {
  id: string;
  name: string;              // Mesh name
  meshPath: string | null;   // Full hierarchy path (e.g., "Parent > Child > Mesh")
  materialName: string | null; // Material name if set
  materialIndex: number;
  roughness: number;
  metalness: number;
  color: string;
  emissive: string;
  emissiveIntensity: number;
  opacity: number;
  transparent: boolean;
  envMapIntensity: number;
}

interface ShadowInfo {
  enabled: boolean;
  mapSize: number;
  bias: number;
  normalBias: number;
  cameraNear: number;
  cameraFar: number;
  cameraLeft: number | null;
  cameraRight: number | null;
  cameraTop: number | null;
  cameraBottom: number | null;
}

interface LightInfo {
  id: string;
  type: string;
  intensity: number;
  color: string;
  position: { x: number; y: number; z: number } | null;
  distance: number | null;
  angle: number | null;
  penumbra: number | null;
  groundColor: string | null;
  target: { x: number; y: number; z: number } | null;
  hasTarget: boolean;
  shadow: ShadowInfo | null;
}

interface PlacedMeshInfo {
  id: string;
  name: string;
  assetId: string;
  displayName?: string; // Custom name for this mesh instance
  visible: boolean;
  position: { x: number; y: number; z: number };
  rotation: { x: number; y: number; z: number };
  scale: { x: number; y: number; z: number };
}

interface DevToolsManagerInterface {
  // Materials
  scanMaterials: () => MaterialInfo[];
  getMaterialsList: () => MaterialInfo[];
  updateMaterial: (id: string, property: string, value: number | string | boolean) => void;
  resetMaterial: (id: string) => void;
  resetAllMaterials: () => void;
  // Lights
  addLight: (type: string, options?: Record<string, unknown>) => string;
  removeLight: (id: string) => void;
  updateLight: (id: string, property: string, value: unknown) => void;
  getLightsList: () => LightInfo[];
  selectLight: (id: string, mode?: string) => void;
  deselectLight: () => void;
  setLightTransformMode: (mode: string) => void;
  getLightTransformMode: () => string;
  enableLightPositioning: (id: string) => void;
  setLightHelpersVisible: (visible: boolean) => void;
  // Mesh selection for materials
  enableMeshSelection: () => void;
  disableMeshSelection: () => void;
  // Placed meshes
  addMeshFromAsset: (assetId: string, assetName: string) => Promise<string>;
  getPlacedMeshesList: () => PlacedMeshInfo[];
  selectPlacedMesh: (meshId: string) => void;
  deselectPlacedMesh: () => void;
  removePlacedMesh: (meshId: string) => void;
  duplicatePlacedMesh: (meshId: string) => void;
  updatePlacedMeshName: (meshId: string, name: string) => void;
  updatePlacedMeshDisplayName: (meshId: string, displayName: string) => void;
  setMeshTransformMode: (mode: string) => void;
  getMeshTransformMode: () => string;
  // Environment
  collectEnvironmentData: () => { meshes: unknown[]; lights: unknown[] };
  loadEnvironmentInEditor: (data: Environment) => Promise<void>;
  clearAllPlacedMeshes: () => void;
  // Dev mode
  enterDevMode: () => void;
  exitDevMode: (restoreCamera?: boolean) => void;
  isDevModeActive: () => boolean;
  // Renderer access for exposure
  renderer?: { toneMappingExposure: number };
  // Environment Manager (Dev Mode Lot 3)
  environmentManager?: {
    scene: { remove: (obj: unknown) => void };
    groundMesh: unknown;
    clearEnvironment: (clearSkybox?: boolean) => Promise<void>;
    setSkyboxRotation: (degrees: number, axis?: 'x' | 'y' | 'z') => void;
    getSkyboxRotation: () => number;
    startSkyboxAnimation: (speed?: number) => void;
    stopSkyboxAnimation: () => void;
    setSkyboxAnimationSpeed: (speed: number) => void;
    getSkyboxAnimationSpeed: () => number;
    isSkyboxAnimationEnabled: () => boolean;
    setGroundHeight: (height: number) => void;
    getGroundHeight: () => number;
    loadGround: (textureId: string, repeatX: number, repeatY: number, height: number, heightmap: number[] | null) => Promise<void>;
    removeGround: () => void;
    flattenTerrain: (height?: number) => void;
    applyBrush: (x: number, z: number, radius: number, strength: number, falloff?: string) => void;
    serializeHeightmap: () => number[] | null;
    deserializeHeightmap: (heightmap: number[] | null) => void;
    recalculateGroundNormals: () => void;
    getGroundMesh: () => unknown;
  };
  // Terraforming mode (Dev Mode Lot 3 - US4)
  enableTerraformingMode?: () => void;
  disableTerraformingMode?: () => void;
  isTerraformingModeActive?: () => boolean;
  setTerraformBrushSize?: (size: number) => void;
  getTerraformBrushSize?: () => number;
  setTerraformBrushStrength?: (strength: number) => void;
  getTerraformBrushStrength?: () => number;
  flattenTerrain?: () => void;
  // Arena Manager (for arena decoration control)
  arenaManager?: {
    setArenaDecorVisible: (visible: boolean) => void;
    isArenaDecorVisible: () => boolean;
  };
  // Callbacks
  onMaterialsUpdate: ((materials: MaterialInfo[]) => void) | null;
  onLightsUpdate: ((lights: LightInfo[]) => void) | null;
  onPlacedMeshesUpdate: ((meshes: PlacedMeshInfo[]) => void) | null;
  onSelectionChange: ((selection: { type: string | null; id?: string; materialEntry?: MaterialInfo | null; mode?: string }) => void) | null;
  onDevModeChange: ((active: boolean) => void) | null;
}

interface DevToolsPanelProps {
  devToolsManager: DevToolsManagerInterface | null;
  onSkyboxChange?: (assetId: string) => void;
}

type TabId = 'environment' | 'assets' | 'meshes' | 'skybox' | 'terrain' | 'lights' | 'materials';

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export function DevToolsPanel({
  devToolsManager,
  onSkyboxChange,
}: DevToolsPanelProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<TabId>('environment');

  // Panel position and size state
  const [panelState, setPanelState] = useState<PanelState>(loadPanelState);
  const [isDragging, setIsDragging] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  const dragStartRef = useRef<{ x: number; y: number; panelX: number; panelY: number } | null>(null);
  const resizeStartRef = useRef<{ x: number; y: number; width: number; height: number } | null>(null);
  const panelRef = useRef<HTMLDivElement>(null);

  // Environment state
  const [environments, setEnvironments] = useState<EnvironmentListItem[]>([]);
  const [currentEnvironmentId, setCurrentEnvironmentId] = useState<string | null>(null);
  const [currentEnvironmentName, setCurrentEnvironmentName] = useState('');
  const [environmentsLoading, setEnvironmentsLoading] = useState(false);
  const [saving, setSaving] = useState(false);

  // Assets state
  const [assets, setAssets] = useState<Asset[]>([]);
  const [assetsLoading, setAssetsLoading] = useState(false);
  const [assetFilter, setAssetFilter] = useState<AssetType | 'all'>('all');
  const [uploading, setUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0); // 0-100 percentage
  const [uploadFileName, setUploadFileName] = useState('');
  const [isDragOver, setIsDragOver] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const dropZoneRef = useRef<HTMLDivElement>(null);

  // Skybox assets state
  const [skyboxAssets, setSkyboxAssets] = useState<Asset[]>([]);
  const [skyboxAssetsLoading, setSkyboxAssetsLoading] = useState(false);
  const [selectedSkyboxId, setSelectedSkyboxId] = useState<string | null>(null);
  const [exposure, setExposure] = useState(1.0);

  // Skybox rotation state (Dev Mode Lot 3 - US1/US2)
  const [skyboxRotationX, setSkyboxRotationX] = useState(0);
  const [skyboxRotationY, setSkyboxRotationY] = useState(0);
  const [skyboxRotationZ, setSkyboxRotationZ] = useState(0);
  const [skyboxAnimationEnabled, setSkyboxAnimationEnabled] = useState(false);
  const [skyboxAnimationSpeed, setSkyboxAnimationSpeed] = useState(1.0);

  // Ground state (Dev Mode Lot 3 - US3/US4)
  const [groundEnabled, setGroundEnabled] = useState(true);
  const [groundTextureId, setGroundTextureId] = useState<string | null>(null);
  const [groundRepeatX, setGroundRepeatX] = useState(10);
  const [groundRepeatY, setGroundRepeatY] = useState(10);
  const [groundHeight, setGroundHeight] = useState(0);

  // Arena decoration state (stands, stadium surroundings)
  const [showArenaDecor, setShowArenaDecor] = useState(true);

  // Ground texture assets
  const [groundTextureAssets, setGroundTextureAssets] = useState<Asset[]>([]);
  const [groundTextureLoading, setGroundTextureLoading] = useState(false);

  // Terraforming state (Dev Mode Lot 3 - US4)
  const [terraformingEnabled, setTerraformingEnabled] = useState(false);
  const [brushSize, setBrushSize] = useState(500);
  const [brushStrength, setBrushStrength] = useState(10);

  // Mesh search state
  const [meshSearchQuery, setMeshSearchQuery] = useState('');
  const [showMeshDropdown, setShowMeshDropdown] = useState(false);

  // Placed meshes state
  const [placedMeshes, setPlacedMeshes] = useState<PlacedMeshInfo[]>([]);
  const [selectedMeshId, setSelectedMeshId] = useState<string | null>(null);
  const [meshTransformMode, setMeshTransformMode] = useState<'translate' | 'rotate' | 'scale'>('translate');
  const [addingMesh, setAddingMesh] = useState(false);

  // Materials state
  const [materials, setMaterials] = useState<MaterialInfo[]>([]);
  const [expandedMaterials, setExpandedMaterials] = useState<Set<string>>(new Set());
  const [isSelectingMaterial, setIsSelectingMaterial] = useState(false);
  const [selectedMaterialId, setSelectedMaterialId] = useState<string | null>(null);
  const [materialSearchFilter, setMaterialSearchFilter] = useState('');

  // Lights state
  const [lights, setLights] = useState<LightInfo[]>([]);
  const [selectedLightId, setSelectedLightId] = useState<string | null>(null);
  const [showHelpers, setShowHelpers] = useState(true);
  const [isPositioning, setIsPositioning] = useState(false);
  const [lightTransformMode, setLightTransformMode] = useState<'translate' | 'rotate'>('translate');

  // Refs
  const materialsListRef = useRef<HTMLDivElement>(null);
  const materialItemRefs = useRef<Map<string, HTMLDivElement>>(new Map());

  // ============================================================================
  // DATA LOADING
  // ============================================================================

  const loadEnvironments = useCallback(async () => {
    setEnvironmentsLoading(true);
    try {
      const response = await environmentApi.list();
      setEnvironments(response.environments);
    } catch (error) {
      console.error('[DevTools] Failed to load environments:', error);
    } finally {
      setEnvironmentsLoading(false);
    }
  }, []);

  const loadAssets = useCallback(async () => {
    setAssetsLoading(true);
    try {
      const response = await assetApi.list({ limit: 100 });
      setAssets(response.assets);
    } catch (error) {
      console.error('[DevTools] Failed to load assets:', error);
    } finally {
      setAssetsLoading(false);
    }
  }, []);

  const loadSkyboxAssets = useCallback(async () => {
    setSkyboxAssetsLoading(true);
    try {
      const response = await assetApi.list({ type: 'skybox' });
      setSkyboxAssets(response.assets);
    } catch (error) {
      console.error('[DevTools] Failed to load skybox assets:', error);
    } finally {
      setSkyboxAssetsLoading(false);
    }
  }, []);

  const loadGroundTextureAssets = useCallback(async () => {
    setGroundTextureLoading(true);
    try {
      const response = await assetApi.list({ type: 'ground_texture' });
      setGroundTextureAssets(response.assets);
    } catch (error) {
      console.error('[DevTools] Failed to load ground texture assets:', error);
    } finally {
      setGroundTextureLoading(false);
    }
  }, []);

  // ============================================================================
  // PANEL OPEN/CLOSE
  // ============================================================================

  const handleOpen = useCallback(() => {
    setIsOpen(true);
    devToolsManager?.enterDevMode();
    devToolsManager?.setLightHelpersVisible(false);
    loadEnvironments();
    loadAssets();
    loadSkyboxAssets();
    loadGroundTextureAssets();
    // Get initial placed meshes
    if (devToolsManager) {
      setPlacedMeshes(devToolsManager.getPlacedMeshesList());
    }
  }, [devToolsManager, loadEnvironments, loadAssets, loadSkyboxAssets, loadGroundTextureAssets]);

  const handleClose = useCallback(() => {
    setIsOpen(false);
    devToolsManager?.exitDevMode(true);
    setIsSelectingMaterial(false);
    setIsPositioning(false);
  }, [devToolsManager]);

  // ============================================================================
  // CALLBACKS SETUP
  // ============================================================================

  useEffect(() => {
    if (!devToolsManager) return;

    devToolsManager.onMaterialsUpdate = (newMaterials) => {
      setMaterials(newMaterials);
    };

    devToolsManager.onLightsUpdate = (newLights) => {
      setLights(newLights);
    };

    devToolsManager.onPlacedMeshesUpdate = (newMeshes) => {
      setPlacedMeshes(newMeshes);
    };

    devToolsManager.onSelectionChange = (selection) => {
      if (selection.type === 'mesh' && selection.materialEntry) {
        const selectedId = selection.materialEntry.id;
        setSelectedMaterialId(selectedId);
        setExpandedMaterials(new Set([selectedId]));
        setIsSelectingMaterial(false);
        devToolsManager.disableMeshSelection();
        setTimeout(() => {
          const itemRef = materialItemRefs.current.get(selectedId);
          if (itemRef && materialsListRef.current) {
            itemRef.scrollIntoView({ behavior: 'smooth', block: 'center' });
          }
        }, 50);
      } else if (selection.type === 'light') {
        setSelectedLightId(selection.id || null);
        if (selection.mode) {
          setLightTransformMode(selection.mode as 'translate' | 'rotate');
        }
      } else if (selection.type === 'placed-mesh') {
        setSelectedMeshId(selection.id || null);
        if (selection.mode) {
          setMeshTransformMode(selection.mode as 'translate' | 'rotate' | 'scale');
        }
      }
    };

    return () => {
      devToolsManager.onMaterialsUpdate = null;
      devToolsManager.onLightsUpdate = null;
      devToolsManager.onPlacedMeshesUpdate = null;
      devToolsManager.onSelectionChange = null;
    };
  }, [devToolsManager]);

  // ============================================================================
  // TAB CHANGE
  // ============================================================================

  const handleTabChange = (tabId: TabId) => {
    setActiveTab(tabId);

    // Show light helpers only in Lights tab
    if (tabId === 'lights') {
      devToolsManager?.setLightHelpersVisible(showHelpers);
    } else {
      devToolsManager?.setLightHelpersVisible(false);
      devToolsManager?.deselectLight();
      setSelectedLightId(null);
    }

    // Deselect mesh when leaving meshes tab
    if (tabId !== 'meshes') {
      devToolsManager?.deselectPlacedMesh();
      setSelectedMeshId(null);
    }

    // Disable terraforming mode when leaving terrain tab
    if (tabId !== 'terrain') {
      if (terraformingEnabled) {
        devToolsManager?.disableTerraformingMode?.();
        setTerraformingEnabled(false);
      }
    }

    // Disable material selection mode when leaving materials tab
    if (tabId !== 'materials') {
      if (isSelectingMaterial) {
        devToolsManager?.disableMeshSelection();
        setIsSelectingMaterial(false);
      }
    }

    // Reload data for specific tabs
    if (tabId === 'skybox') loadSkyboxAssets();
    if (tabId === 'assets') loadAssets();
    if (tabId === 'environment') loadEnvironments();

    // Close mesh dropdown when changing tabs
    setShowMeshDropdown(false);
    setMeshSearchQuery('');
  };

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = () => {
      setShowMeshDropdown(false);
    };
    if (showMeshDropdown) {
      document.addEventListener('click', handleClickOutside);
      return () => document.removeEventListener('click', handleClickOutside);
    }
  }, [showMeshDropdown]);

  // ============================================================================
  // DRAG & RESIZE HANDLERS
  // ============================================================================

  const handleDragStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    dragStartRef.current = {
      x: e.clientX,
      y: e.clientY,
      panelX: panelState.x,
      panelY: panelState.y,
    };
  }, [panelState.x, panelState.y]);

  const handleResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsResizing(true);
    resizeStartRef.current = {
      x: e.clientX,
      y: e.clientY,
      width: panelState.width,
      height: panelState.height,
    };
  }, [panelState.width, panelState.height]);

  useEffect(() => {
    if (!isDragging && !isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      if (isDragging && dragStartRef.current) {
        const dx = e.clientX - dragStartRef.current.x;
        const dy = e.clientY - dragStartRef.current.y;
        const newX = Math.max(0, Math.min(dragStartRef.current.panelX + dx, window.innerWidth - 100));
        const newY = Math.max(0, Math.min(dragStartRef.current.panelY + dy, window.innerHeight - 100));
        setPanelState(prev => ({ ...prev, x: newX, y: newY }));
      } else if (isResizing && resizeStartRef.current) {
        const dx = e.clientX - resizeStartRef.current.x;
        const dy = e.clientY - resizeStartRef.current.y;
        const newWidth = Math.max(MIN_WIDTH, Math.min(resizeStartRef.current.width + dx, MAX_WIDTH));
        const newHeight = Math.max(MIN_HEIGHT, Math.min(resizeStartRef.current.height + dy, window.innerHeight - panelState.y - 20));
        setPanelState(prev => ({ ...prev, width: newWidth, height: newHeight }));
      }
    };

    const handleMouseUp = () => {
      if (isDragging || isResizing) {
        setIsDragging(false);
        setIsResizing(false);
        dragStartRef.current = null;
        resizeStartRef.current = null;
        // Save to localStorage
        setPanelState(prev => {
          savePanelState(prev);
          return prev;
        });
      }
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, isResizing, panelState.y]);

  // ============================================================================
  // ENVIRONMENT HANDLERS
  // ============================================================================

  const handleLoadEnvironment = async (envId: string) => {
    try {
      const env = await environmentApi.get(envId);
      setCurrentEnvironmentId(envId);
      setCurrentEnvironmentName(env.name);
      await devToolsManager?.loadEnvironmentInEditor(env);
      setPlacedMeshes(devToolsManager?.getPlacedMeshesList() || []);

      // Update skybox state
      if (env.skyboxAssetId) {
        setSelectedSkyboxId(env.skyboxAssetId);
        onSkyboxChange?.(env.skyboxAssetId);
      }
      if (env.skyboxExposure !== undefined) {
        setExposure(env.skyboxExposure);
        // Apply exposure to renderer
        if (devToolsManager?.renderer) {
          devToolsManager.renderer.toneMappingExposure = env.skyboxExposure;
        }
      }

      // Update skybox rotation state (Dev Mode Lot 3 - US1/US2)
      setSkyboxRotationX(env.skyboxRotationX ?? 0);
      setSkyboxRotationY(env.skyboxRotationY ?? 0);
      setSkyboxRotationZ(env.skyboxRotationZ ?? 0);
      setSkyboxAnimationEnabled(env.skyboxAnimationEnabled ?? false);
      setSkyboxAnimationSpeed(env.skyboxAnimationSpeed ?? 1.0);

      // Update ground state (Dev Mode Lot 3 - US3/US4)
      setGroundEnabled(env.groundEnabled ?? true);
      setGroundTextureId(env.groundTextureId ?? null);
      setGroundRepeatX(env.groundRepeatX ?? 10);
      setGroundRepeatY(env.groundRepeatY ?? 10);
      setGroundHeight(env.groundHeight ?? 0);

      // Update arena decoration state
      const showDecor = env.showArenaDecor ?? true;
      setShowArenaDecor(showDecor);
      devToolsManager?.arenaManager?.setArenaDecorVisible(showDecor);

      // Disable terraforming mode when loading new environment
      setTerraformingEnabled(false);
      devToolsManager?.disableTerraformingMode?.();
    } catch (error) {
      console.error('[DevTools] Failed to load environment:', error);
    }
  };

  const handleNewEnvironment = async () => {
    // Clear current environment
    setCurrentEnvironmentId(null);
    setCurrentEnvironmentName('');

    // Clear the replay environment first (meshes, lights, ground from EnvironmentManager)
    await devToolsManager?.environmentManager?.clearEnvironment();

    // Clear all DevTools meshes and lights
    devToolsManager?.clearAllPlacedMeshes();
    const lightIds = devToolsManager?.getLightsList().map(l => l.id) || [];
    lightIds.forEach(id => devToolsManager?.removeLight(id));

    // Reset skybox and exposure
    setSelectedSkyboxId(null);
    setExposure(1.0);
    if (devToolsManager?.renderer) {
      devToolsManager.renderer.toneMappingExposure = 1.0;
    }

    // Reset skybox rotation state
    setSkyboxRotationX(0);
    setSkyboxRotationY(0);
    setSkyboxRotationZ(0);
    setSkyboxAnimationEnabled(false);
    setSkyboxAnimationSpeed(1.0);

    // Reset ground state
    setGroundEnabled(false);
    setGroundTextureId(null);
    setGroundRepeatX(10);
    setGroundRepeatY(10);
    setGroundHeight(0);
    setTerraformingEnabled(false);
    devToolsManager?.disableTerraformingMode?.();

    // Reset arena decoration state
    setShowArenaDecor(true);
    devToolsManager?.arenaManager?.setArenaDecorVisible(true);

    // Update UI
    setPlacedMeshes([]);
    setLights([]);
  };

  const handleSaveEnvironment = async (saveAsNew = false) => {
    const name = saveAsNew || !currentEnvironmentId
      ? prompt('Environment name:', currentEnvironmentName || 'New Environment')
      : currentEnvironmentName;

    if (!name) return;

    setSaving(true);
    try {
      const envData = devToolsManager?.collectEnvironmentData();
      const heightmap = devToolsManager?.environmentManager?.serializeHeightmap?.() || null;
      const payload = {
        name,
        skyboxAssetId: selectedSkyboxId || undefined,
        skyboxExposure: exposure,
        // Skybox rotation (Dev Mode Lot 3 - US1/US2)
        skyboxRotationX: skyboxRotationX,
        skyboxRotationY: skyboxRotationY,
        skyboxRotationZ: skyboxRotationZ,
        skyboxAnimationEnabled: skyboxAnimationEnabled,
        skyboxAnimationSpeed: skyboxAnimationSpeed,
        // Ground settings (Dev Mode Lot 3 - US3/US4)
        groundEnabled: groundEnabled,
        groundTextureId: groundTextureId || undefined,
        groundRepeatX: groundRepeatX,
        groundRepeatY: groundRepeatY,
        groundHeight: groundHeight,
        groundHeightmap: heightmap,
        // Arena decoration (stands, stadium surroundings)
        showArenaDecor: showArenaDecor,
        meshes: (envData?.meshes || []) as CreateMeshRequest[],
        lights: (envData?.lights || []) as CreateLightRequest[],
      };

      if (currentEnvironmentId && !saveAsNew) {
        await environmentApi.update(currentEnvironmentId, payload);
      } else {
        const newEnv = await environmentApi.create(payload);
        setCurrentEnvironmentId(newEnv.id);
        setCurrentEnvironmentName(newEnv.name);
      }

      await loadEnvironments();
    } catch (error) {
      console.error('[DevTools] Failed to save environment:', error);
      alert('Failed to save environment');
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteEnvironment = async (envId: string) => {
    if (!confirm('Delete this environment?')) return;
    try {
      await environmentApi.delete(envId);
      if (currentEnvironmentId === envId) {
        setCurrentEnvironmentId(null);
        setCurrentEnvironmentName('');
      }
      await loadEnvironments();
    } catch (error) {
      console.error('[DevTools] Failed to delete environment:', error);
    }
  };

  const handleSetDefaultEnvironment = async (envId: string) => {
    try {
      await environmentApi.setDefault(envId);
      await loadEnvironments();
    } catch (error) {
      console.error('[DevTools] Failed to set default environment:', error);
    }
  };

  // ============================================================================
  // ASSET HANDLERS
  // ============================================================================

  const uploadFiles = useCallback(async (files: File[]) => {
    if (files.length === 0) return;

    setUploading(true);
    setUploadProgress(0);

    try {
      const totalFiles = files.length;
      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        setUploadFileName(file.name);
        // Progress: each file is a portion of total (simulate progress within file)
        const baseProgress = (i / totalFiles) * 100;
        const fileProgress = (1 / totalFiles) * 100;

        // Determine asset type from file
        let type: AssetType = 'mesh';
        if (file.name.endsWith('.hdr') || file.name.endsWith('.exr')) {
          type = 'skybox';
        } else if (file.type.startsWith('image/')) {
          type = 'ground_texture';
        }

        // Simulate upload progress (since we don't have real progress from API)
        setUploadProgress(baseProgress + fileProgress * 0.3);
        await assetApi.upload(file, type);
        setUploadProgress(baseProgress + fileProgress);
      }
      await loadAssets();
      await loadSkyboxAssets();
      await loadGroundTextureAssets();
    } catch (error) {
      console.error('[DevTools] Failed to upload:', error);
      alert('Upload failed');
    } finally {
      setUploading(false);
      setUploadProgress(0);
      setUploadFileName('');
      if (fileInputRef.current) fileInputRef.current.value = '';
    }
  }, [loadAssets, loadSkyboxAssets, loadGroundTextureAssets]);

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;
    await uploadFiles(Array.from(files));
  };

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    // Only set false if we're leaving the drop zone entirely
    if (dropZoneRef.current && !dropZoneRef.current.contains(e.relatedTarget as Node)) {
      setIsDragOver(false);
    }
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);

    const files = Array.from(e.dataTransfer.files).filter(file => {
      const ext = file.name.toLowerCase();
      return ext.endsWith('.glb') || ext.endsWith('.gltf') ||
             ext.endsWith('.hdr') || ext.endsWith('.exr') ||
             ext.endsWith('.png') || ext.endsWith('.jpg') || ext.endsWith('.jpeg');
    });

    if (files.length > 0) {
      await uploadFiles(files);
    }
  }, [uploadFiles]);

  const handleDeleteAsset = async (assetId: string) => {
    if (!confirm('Delete this asset?')) return;
    try {
      await assetApi.delete(assetId);
      await loadAssets();
      await loadSkyboxAssets();
      await loadGroundTextureAssets();
    } catch (error) {
      console.error('[DevTools] Failed to delete asset:', error);
    }
  };

  const handleRenameAsset = async (assetId: string, newName: string): Promise<{ success: boolean; error?: string }> => {
    try {
      await assetApi.rename(assetId, newName);
      await loadAssets();
      await loadSkyboxAssets();
      await loadGroundTextureAssets();
      return { success: true };
    } catch (error) {
      console.error('[DevTools] Failed to rename asset:', error);
      // Extract error message from API response
      if (error instanceof Error) {
        // Try to parse API error message
        const match = error.message.match(/already exists|cannot be empty|too long/i);
        if (match) {
          return { success: false, error: error.message };
        }
      }
      return { success: false, error: 'Failed to rename asset' };
    }
  };

  const handleAddMeshToScene = async (asset: Asset) => {
    if (!devToolsManager) return;
    setAddingMesh(true);
    try {
      await devToolsManager.addMeshFromAsset(asset.id, asset.name);
      setPlacedMeshes(devToolsManager.getPlacedMeshesList());
      setActiveTab('meshes');
      setShowMeshDropdown(false);
      setMeshSearchQuery('');
    } catch (error) {
      console.error('[DevTools] Failed to add mesh:', error);
      alert('Failed to add mesh to scene');
    } finally {
      setAddingMesh(false);
    }
  };

  // ============================================================================
  // SKYBOX HANDLERS
  // ============================================================================

  const handleSelectSkybox = (assetId: string) => {
    setSelectedSkyboxId(assetId);
    onSkyboxChange?.(assetId);
  };

  const handleExposureChange = (value: number) => {
    setExposure(value);
    if (devToolsManager?.renderer) {
      devToolsManager.renderer.toneMappingExposure = value;
    }
  };

  // ============================================================================
  // SKYBOX ROTATION HANDLERS (Dev Mode Lot 3 - US1/US2)
  // ============================================================================

  const handleSkyboxRotationXChange = (value: number) => {
    setSkyboxRotationX(value);
    devToolsManager?.environmentManager?.setSkyboxRotation(value, 'x');
  };

  const handleSkyboxRotationYChange = (value: number) => {
    setSkyboxRotationY(value);
    devToolsManager?.environmentManager?.setSkyboxRotation(value, 'y');
  };

  const handleSkyboxRotationZChange = (value: number) => {
    setSkyboxRotationZ(value);
    devToolsManager?.environmentManager?.setSkyboxRotation(value, 'z');
  };

  const handleSkyboxAnimationToggle = (enabled: boolean) => {
    setSkyboxAnimationEnabled(enabled);
    if (enabled) {
      devToolsManager?.environmentManager?.startSkyboxAnimation(skyboxAnimationSpeed);
    } else {
      devToolsManager?.environmentManager?.stopSkyboxAnimation();
    }
  };

  const handleSkyboxAnimationSpeedChange = (value: number) => {
    setSkyboxAnimationSpeed(value);
    devToolsManager?.environmentManager?.setSkyboxAnimationSpeed(value);
  };

  // ============================================================================
  // GROUND HANDLERS (Dev Mode Lot 3 - US3/US4)
  // ============================================================================

  const handleGroundEnabledChange = async (enabled: boolean) => {
    setGroundEnabled(enabled);
    const envManager = devToolsManager?.environmentManager;
    if (!envManager) return;

    if (!enabled) {
      // Remove ground from scene (properly dispose resources)
      envManager.removeGround();
    } else if (groundTextureId) {
      // Re-add ground with current texture
      await envManager.loadGround(groundTextureId, groundRepeatX, groundRepeatY, groundHeight, null);
    }
  };

  const handleGroundTextureChange = async (assetId: string) => {
    setGroundTextureId(assetId);
    const envManager = devToolsManager?.environmentManager;
    if (!envManager || !groundEnabled) return;

    // Save current heightmap before removing ground
    const currentHeightmap = envManager.serializeHeightmap();

    // Remove existing ground (properly dispose resources)
    envManager.removeGround();

    // Load new ground with selected texture, preserving heightmap
    await envManager.loadGround(assetId, groundRepeatX, groundRepeatY, groundHeight, currentHeightmap);
  };

  const handleGroundHeightChange = (value: number) => {
    setGroundHeight(value);
    devToolsManager?.environmentManager?.setGroundHeight(value);
  };

  const handleGroundRepeatChange = async (x: number, y: number) => {
    setGroundRepeatX(x);
    setGroundRepeatY(y);

    // Reload ground with new repeat values if texture is selected
    const envManager = devToolsManager?.environmentManager;
    if (!envManager || !groundEnabled || !groundTextureId) return;

    // Save current heightmap before removing ground
    const currentHeightmap = envManager.serializeHeightmap();

    // Remove existing ground (properly dispose resources)
    envManager.removeGround();

    // Reload with new texture scale, preserving heightmap
    await envManager.loadGround(groundTextureId, x, y, groundHeight, currentHeightmap);
  };

  // ============================================================================
  // TERRAFORMING HANDLERS (Dev Mode Lot 3 - US4)
  // ============================================================================

  const handleTerraformingToggle = (enabled: boolean) => {
    setTerraformingEnabled(enabled);
    if (enabled) {
      devToolsManager?.enableTerraformingMode?.();
    } else {
      devToolsManager?.disableTerraformingMode?.();
    }
  };

  const handleBrushSizeChange = (value: number) => {
    setBrushSize(value);
    devToolsManager?.setTerraformBrushSize?.(value);
  };

  const handleBrushStrengthChange = (value: number) => {
    setBrushStrength(value);
    devToolsManager?.setTerraformBrushStrength?.(value);
  };

  const handleFlattenTerrain = () => {
    devToolsManager?.flattenTerrain?.();
  };

  // ============================================================================
  // MESH HANDLERS
  // ============================================================================

  const handleSelectMesh = (meshId: string) => {
    setSelectedMeshId(meshId);
    devToolsManager?.selectPlacedMesh(meshId);
  };

  const handleDeselectMesh = () => {
    setSelectedMeshId(null);
    devToolsManager?.deselectPlacedMesh();
  };

  const handleRemoveMesh = (meshId: string) => {
    devToolsManager?.removePlacedMesh(meshId);
    if (selectedMeshId === meshId) {
      setSelectedMeshId(null);
    }
  };

  const handleDuplicateMesh = (meshId: string) => {
    devToolsManager?.duplicatePlacedMesh(meshId);
  };

  const handleRenameMesh = (meshId: string, displayName: string) => {
    devToolsManager?.updatePlacedMeshDisplayName(meshId, displayName);
    // Update local state
    setPlacedMeshes(prev => prev.map(m =>
      m.id === meshId ? { ...m, displayName } : m
    ));
  };

  const handleMeshTransformModeChange = (mode: 'translate' | 'rotate' | 'scale') => {
    setMeshTransformMode(mode);
    devToolsManager?.setMeshTransformMode(mode);
  };

  // ============================================================================
  // MATERIAL HANDLERS
  // ============================================================================

  const handleScanMaterials = useCallback(() => {
    if (devToolsManager) {
      const scanned = devToolsManager.scanMaterials();
      setMaterials(scanned);
    }
  }, [devToolsManager]);

  const handleToggleMaterial = (id: string) => {
    setExpandedMaterials((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(id)) newSet.delete(id);
      else newSet.add(id);
      return newSet;
    });
  };

  const handleMaterialChange = (id: string, property: string, value: number | string | boolean) => {
    devToolsManager?.updateMaterial(id, property, value);
  };

  const handleResetMaterial = (id: string) => {
    devToolsManager?.resetMaterial(id);
  };

  const handleResetAllMaterials = () => {
    devToolsManager?.resetAllMaterials();
  };

  const handleStartMaterialSelection = () => {
    setIsSelectingMaterial(true);
    devToolsManager?.enableMeshSelection();
  };

  const handleCancelMaterialSelection = () => {
    setIsSelectingMaterial(false);
    devToolsManager?.disableMeshSelection();
  };

  // ============================================================================
  // LIGHT HANDLERS
  // ============================================================================

  const handleAddLight = (type: string) => {
    if (devToolsManager) {
      const id = devToolsManager.addLight(type);
      if (id) {
        setSelectedLightId(id);
        devToolsManager.selectLight(id);
      }
    }
  };

  const handleRemoveLight = (id: string) => {
    devToolsManager?.removeLight(id);
    if (selectedLightId === id) setSelectedLightId(null);
  };

  const handleSelectLight = (id: string) => {
    setSelectedLightId(id);
    devToolsManager?.selectLight(id);
  };

  const handleDeselectLight = () => {
    setSelectedLightId(null);
    devToolsManager?.deselectLight();
  };

  const handleLightChange = (id: string, property: string, value: unknown) => {
    devToolsManager?.updateLight(id, property, value);
  };

  const handlePositionLight = (id: string) => {
    setIsPositioning(true);
    devToolsManager?.enableLightPositioning(id);
    setTimeout(() => setIsPositioning(false), 10000);
  };

  const handleSetLightTransformMode = (mode: 'translate' | 'rotate') => {
    setLightTransformMode(mode);
    devToolsManager?.setLightTransformMode(mode);
  };

  const handleToggleHelpers = () => {
    const newValue = !showHelpers;
    setShowHelpers(newValue);
    devToolsManager?.setLightHelpersVisible(newValue);
  };

  // ============================================================================
  // COMPUTED VALUES
  // ============================================================================

  const filteredAssets = assets.filter(a => assetFilter === 'all' || a.type === assetFilter);
  const meshAssets = assets.filter(a => a.type === 'mesh');
  const filteredMeshAssets = meshAssets.filter(a =>
    meshSearchQuery === '' || a.name.toLowerCase().includes(meshSearchQuery.toLowerCase())
  );

  const filteredMaterials = materials.filter(
    (m) =>
      m.name.toLowerCase().includes(materialSearchFilter.toLowerCase()) ||
      m.id.toLowerCase().includes(materialSearchFilter.toLowerCase())
  );

  const tabs = [
    { id: 'environment' as const, label: 'Env', icon: FolderOpen },
    { id: 'assets' as const, label: 'Assets', icon: Upload },
    { id: 'meshes' as const, label: 'Meshes', icon: Box },
    { id: 'skybox' as const, label: 'Sky', icon: Cloud },
    { id: 'terrain' as const, label: 'Ground', icon: Mountain },
    { id: 'lights' as const, label: 'Lights', icon: Lightbulb },
    { id: 'materials' as const, label: 'Mat', icon: Palette },
  ];

  if (!devToolsManager) return null;

  // ============================================================================
  // RENDER
  // ============================================================================

  return (
    <>
      {/* DevTools Button - Gradient style matching app design */}
      <button
        onClick={handleOpen}
        className="pointer-events-auto relative p-[1px] rounded-lg bg-gradient-to-r from-violet-500 to-blue-500 hover:from-violet-400 hover:to-blue-400 transition-all shadow-lg shadow-violet-500/20 hover:shadow-violet-500/40"
        title="DevTools"
      >
        <div className="bg-gray-900 rounded-[7px] p-2">
          <Wrench size={20} className="text-violet-400" />
        </div>
      </button>

      {/* DevTools Modal */}
      {isOpen && (
        <div className="fixed inset-0 z-50 pointer-events-none">
          {/* Panel with gradient border */}
          <div
            ref={panelRef}
            className="absolute p-[1px] rounded-xl bg-gradient-to-br from-violet-500/50 via-blue-500/30 to-violet-500/50 shadow-2xl shadow-violet-500/10 pointer-events-auto"
            style={{
              left: panelState.x,
              top: panelState.y,
              width: panelState.width,
              height: panelState.height,
              cursor: isDragging ? 'grabbing' : undefined,
            }}
          >
            <div className="bg-gray-900/95 backdrop-blur-sm rounded-[11px] h-full flex flex-col overflow-hidden">
              {/* Header with gradient accent */}
              <div
                className="flex items-center justify-between p-3 bg-gradient-to-r from-violet-900/30 to-blue-900/30 border-b border-violet-500/20 flex-shrink-0 cursor-grab active:cursor-grabbing select-none"
                onMouseDown={handleDragStart}
              >
                <div className="flex items-center gap-2">
                  <GripVertical size={14} className="text-gray-500" />
                  <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-violet-500 to-blue-500 flex items-center justify-center">
                    <Wrench size={16} className="text-white" />
                  </div>
                  <div>
                    <span className="font-bold text-sm text-white">Dev Tools</span>
                    {currentEnvironmentName && (
                      <span className="text-xs text-violet-300 ml-2">· {currentEnvironmentName}</span>
                    )}
                  </div>
                </div>
                <button
                  onClick={handleClose}
                  onMouseDown={(e) => e.stopPropagation()}
                  className="p-1.5 rounded-lg bg-gray-800/50 hover:bg-red-500/80 text-gray-400 hover:text-white transition-colors"
                  title="Close"
                >
                  <X size={16} />
                </button>
              </div>

              {/* Camera Controls Hint - More subtle */}
              <div className="px-3 py-1.5 bg-violet-900/20 border-b border-violet-500/10 text-[10px] text-violet-300/80">
                <strong>Camera:</strong> WASD move · Right-click rotate · Scroll zoom
              </div>

              {/* Tabs - Redesigned with pills */}
              <div className="flex gap-1 p-2 border-b border-gray-800 flex-shrink-0 overflow-x-auto bg-gray-900/50">
                {tabs.map((tab) => {
                  const Icon = tab.icon;
                  const isActive = activeTab === tab.id;
                  return (
                    <button
                      key={tab.id}
                      onClick={() => handleTabChange(tab.id)}
                      className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium transition-all rounded-lg whitespace-nowrap ${
                        isActive
                          ? 'bg-gradient-to-r from-violet-600 to-blue-600 text-white shadow-lg shadow-violet-500/25'
                          : 'text-gray-400 hover:text-white hover:bg-gray-800/50'
                      }`}
                    >
                      <Icon size={14} />
                      {tab.label}
                    </button>
                  );
                })}
              </div>

            {/* Content */}
            <div className="p-3 space-y-3 overflow-y-auto flex-1">
              {/* ============================================================ */}
              {/* ENVIRONMENT TAB */}
              {/* ============================================================ */}
              {activeTab === 'environment' && (
                <div className="space-y-4">
                  {/* Save Controls */}
                  <div className="flex gap-2">
                    <button
                      onClick={handleNewEnvironment}
                      className="flex items-center gap-1 px-3 py-2 bg-gray-800 hover:bg-gray-700 border border-gray-700 text-gray-300 hover:text-white text-xs rounded-lg transition-colors"
                      title="Start a new empty environment"
                    >
                      <FilePlus size={14} />
                      New
                    </button>
                    <button
                      onClick={() => handleSaveEnvironment(false)}
                      disabled={saving}
                      className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 bg-gradient-to-r from-violet-600 to-blue-600 hover:from-violet-500 hover:to-blue-500 disabled:opacity-50 text-white text-xs rounded-lg transition-all shadow-lg shadow-violet-500/20"
                    >
                      {saving ? <Loader2 size={14} className="animate-spin" /> : <Save size={14} />}
                      {currentEnvironmentId ? 'Save' : 'Save New'}
                    </button>
                    {currentEnvironmentId && (
                      <button
                        onClick={() => handleSaveEnvironment(true)}
                        disabled={saving}
                        className="flex items-center gap-1 px-3 py-2 bg-blue-600/80 hover:bg-blue-500 disabled:opacity-50 text-white text-xs rounded-lg transition-colors"
                      >
                        <Copy size={14} />
                        Save As
                      </button>
                    )}
                  </div>

                  {/* Environment List */}
                  <div className="space-y-2">
                    <label className="text-xs text-gray-400 font-medium">Load Environment</label>
                    {environmentsLoading ? (
                      <div className="flex items-center justify-center py-4">
                        <Loader2 size={20} className="animate-spin text-gray-400" />
                      </div>
                    ) : environments.length === 0 ? (
                      <p className="text-xs text-gray-500 text-center py-4">No environments yet</p>
                    ) : (
                      <div className="space-y-1 max-h-[200px] overflow-y-auto">
                        {environments.map((env) => (
                          <div
                            key={env.id}
                            className={`flex items-center gap-2 p-2 rounded border ${
                              currentEnvironmentId === env.id
                                ? 'border-violet-500 bg-violet-900/20'
                                : 'border-gray-700 bg-gray-800/50 hover:bg-gray-800'
                            }`}
                          >
                            <button
                              onClick={() => handleLoadEnvironment(env.id)}
                              className="flex-1 text-left text-xs text-white truncate"
                            >
                              {env.name}
                              {env.isDefault && (
                                <span className="ml-1 text-[10px] text-violet-400">(default)</span>
                              )}
                            </button>
                            <span className="text-[10px] text-gray-500">
                              {env.meshCount}m {env.lightCount}l
                            </span>
                            <button
                              onClick={() => handleSetDefaultEnvironment(env.id)}
                              className={`p-1 rounded ${
                                env.isDefault
                                  ? 'bg-violet-600 text-white'
                                  : 'bg-gray-700 text-gray-400 hover:bg-violet-700 hover:text-white'
                              }`}
                              title={env.isDefault ? 'Default environment' : 'Set as default'}
                            >
                              <Star size={12} fill={env.isDefault ? 'currentColor' : 'none'} />
                            </button>
                            <button
                              onClick={() => handleDeleteEnvironment(env.id)}
                              className="p-1 rounded bg-red-900/50 text-red-400 hover:bg-red-800 hover:text-red-300"
                              title="Delete environment"
                            >
                              <Trash2 size={12} />
                            </button>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>

                  {/* Quick Stats */}
                  <div className="grid grid-cols-3 gap-2 text-center">
                    <div className="p-2 bg-gray-800/50 rounded border border-gray-700">
                      <div className="text-lg font-bold text-white">{placedMeshes.length}</div>
                      <div className="text-[10px] text-gray-400">Meshes</div>
                    </div>
                    <div className="p-2 bg-gray-800/50 rounded border border-gray-700">
                      <div className="text-lg font-bold text-white">{lights.length}</div>
                      <div className="text-[10px] text-gray-400">Lights</div>
                    </div>
                    <div className="p-2 bg-gray-800/50 rounded border border-gray-700">
                      <div className="text-lg font-bold text-white">{assets.length}</div>
                      <div className="text-[10px] text-gray-400">Assets</div>
                    </div>
                  </div>
                </div>
              )}

              {/* ============================================================ */}
              {/* ASSETS TAB */}
              {/* ============================================================ */}
              {activeTab === 'assets' && (
                <div className="space-y-3">
                  {/* Upload Drop Zone */}
                  <div className="space-y-2">
                    <input
                      ref={fileInputRef}
                      type="file"
                      multiple
                      accept=".glb,.gltf,.hdr,.exr,.png,.jpg,.jpeg"
                      onChange={handleFileUpload}
                      className="hidden"
                    />
                    <div
                      ref={dropZoneRef}
                      onClick={() => !uploading && fileInputRef.current?.click()}
                      onDragEnter={handleDragEnter}
                      onDragLeave={handleDragLeave}
                      onDragOver={handleDragOver}
                      onDrop={handleDrop}
                      className={`relative w-full p-4 border-2 border-dashed rounded-lg transition-all cursor-pointer ${
                        isDragOver
                          ? 'border-blue-400 bg-blue-900/30'
                          : uploading
                          ? 'border-gray-600 bg-gray-800/50 cursor-wait'
                          : 'border-gray-600 bg-gray-800/30 hover:border-gray-500 hover:bg-gray-800/50'
                      }`}
                    >
                      {uploading ? (
                        <div className="space-y-2">
                          <div className="flex items-center justify-center gap-2 text-blue-400">
                            <Loader2 size={16} className="animate-spin" />
                            <span className="text-xs font-medium">Uploading...</span>
                          </div>
                          <p className="text-[10px] text-gray-400 text-center truncate">
                            {uploadFileName}
                          </p>
                          {/* Progress bar */}
                          <div className="w-full bg-gray-700 rounded-full h-1.5 overflow-hidden">
                            <div
                              className="bg-blue-500 h-full transition-all duration-300 ease-out"
                              style={{ width: `${uploadProgress}%` }}
                            />
                          </div>
                          <p className="text-[10px] text-gray-500 text-center">
                            {Math.round(uploadProgress)}%
                          </p>
                        </div>
                      ) : (
                        <div className="text-center">
                          <Upload size={20} className={`mx-auto mb-1.5 ${isDragOver ? 'text-blue-400' : 'text-gray-400'}`} />
                          <p className={`text-xs font-medium ${isDragOver ? 'text-blue-300' : 'text-gray-300'}`}>
                            {isDragOver ? 'Drop files here' : 'Drop files or click to upload'}
                          </p>
                          <p className="text-[10px] text-gray-500 mt-1">
                            GLB/GLTF · HDR · PNG/JPG
                          </p>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Filter with color indicators */}
                  <div className="flex gap-1">
                    {(['all', 'mesh', 'skybox', 'ground_texture'] as const).map((type) => {
                      const typeColors: Record<string, string> = {
                        all: 'bg-gray-500',
                        mesh: 'bg-blue-500',
                        skybox: 'bg-purple-500',
                        ground_texture: 'bg-green-500',
                      };
                      const typeLabels: Record<string, string> = {
                        all: 'All',
                        mesh: 'Mesh',
                        skybox: 'Skybox',
                        ground_texture: 'Texture',
                      };
                      return (
                        <button
                          key={type}
                          onClick={() => setAssetFilter(type)}
                          className={`flex-1 flex items-center justify-center gap-1.5 px-2 py-1.5 text-[10px] rounded transition-colors ${
                            assetFilter === type
                              ? 'bg-violet-600 text-white'
                              : 'bg-gray-700 text-gray-400 hover:text-white'
                          }`}
                        >
                          <span className={`w-2 h-2 rounded-full ${typeColors[type]}`} />
                          {typeLabels[type]}
                        </button>
                      );
                    })}
                  </div>

                  {/* Asset List */}
                  {assetsLoading ? (
                    <div className="flex items-center justify-center py-8">
                      <Loader2 size={24} className="animate-spin text-gray-400" />
                    </div>
                  ) : filteredAssets.length === 0 ? (
                    <p className="text-xs text-gray-500 text-center py-8">No assets uploaded yet</p>
                  ) : (
                    <div className="space-y-1 max-h-[300px] overflow-y-auto">
                      {filteredAssets.map((asset) => (
                        <AssetCard
                          key={asset.id}
                          asset={asset}
                          onAddToScene={asset.type === 'mesh' ? () => handleAddMeshToScene(asset) : undefined}
                          onDelete={() => handleDeleteAsset(asset.id)}
                          onRename={(newName) => handleRenameAsset(asset.id, newName)}
                          isAdding={addingMesh}
                        />
                      ))}
                    </div>
                  )}
                </div>
              )}

              {/* ============================================================ */}
              {/* MESHES TAB */}
              {/* ============================================================ */}
              {activeTab === 'meshes' && (
                <div className="space-y-3">
                  {/* Transform Mode */}
                  {selectedMeshId && (
                    <div className="flex gap-1">
                      <button
                        onClick={() => handleMeshTransformModeChange('translate')}
                        className={`flex-1 flex items-center justify-center gap-1 px-2 py-1.5 rounded text-xs ${
                          meshTransformMode === 'translate' ? 'bg-violet-600 text-white' : 'bg-gray-700 text-gray-400'
                        }`}
                      >
                        <Move size={12} /> Move
                      </button>
                      <button
                        onClick={() => handleMeshTransformModeChange('rotate')}
                        className={`flex-1 flex items-center justify-center gap-1 px-2 py-1.5 rounded text-xs ${
                          meshTransformMode === 'rotate' ? 'bg-violet-600 text-white' : 'bg-gray-700 text-gray-400'
                        }`}
                      >
                        <RotateCw size={12} /> Rotate
                      </button>
                      <button
                        onClick={() => handleMeshTransformModeChange('scale')}
                        className={`flex-1 flex items-center justify-center gap-1 px-2 py-1.5 rounded text-xs ${
                          meshTransformMode === 'scale' ? 'bg-violet-600 text-white' : 'bg-gray-700 text-gray-400'
                        }`}
                      >
                        <Scale size={12} /> Scale
                      </button>
                    </div>
                  )}

                  {/* Add Mesh from Library - Searchable Dropdown */}
                  <div className="space-y-2">
                    <label className="text-xs text-gray-400 font-medium">Add Mesh to Scene</label>
                    <div className="relative">
                      <div className="flex items-center gap-1 px-2 py-1.5 bg-gray-800 border border-gray-600 rounded">
                        <Search size={12} className="text-gray-400" />
                        <input
                          type="text"
                          placeholder={meshAssets.length > 0 ? "Search meshes..." : "No meshes available"}
                          value={meshSearchQuery}
                          onChange={(e) => {
                            setMeshSearchQuery(e.target.value);
                            setShowMeshDropdown(true);
                          }}
                          onFocus={() => setShowMeshDropdown(true)}
                          disabled={meshAssets.length === 0 || addingMesh}
                          className="flex-1 bg-transparent text-xs text-white placeholder-gray-500 outline-none disabled:opacity-50"
                        />
                        {addingMesh && <Loader2 size={12} className="animate-spin text-gray-400" />}
                      </div>
                      {showMeshDropdown && filteredMeshAssets.length > 0 && (
                        <div className="absolute top-full left-0 right-0 mt-1 bg-gray-800 border border-gray-600 rounded shadow-lg max-h-[150px] overflow-y-auto z-10">
                          {filteredMeshAssets.map((asset) => (
                            <button
                              key={asset.id}
                              onClick={() => handleAddMeshToScene(asset)}
                              disabled={addingMesh}
                              className="w-full flex items-center gap-2 px-2 py-1.5 text-left text-xs text-white hover:bg-gray-700 disabled:opacity-50"
                            >
                              <Plus size={10} className="text-green-400" />
                              <span className="truncate">{asset.name}</span>
                              <span className="ml-auto text-[10px] text-gray-500">{(asset.fileSize / 1024).toFixed(0)}KB</span>
                            </button>
                          ))}
                        </div>
                      )}
                      {showMeshDropdown && meshSearchQuery && filteredMeshAssets.length === 0 && (
                        <div className="absolute top-full left-0 right-0 mt-1 bg-gray-800 border border-gray-600 rounded p-2 text-xs text-gray-500 text-center">
                          No meshes match "{meshSearchQuery}"
                        </div>
                      )}
                    </div>
                    {meshAssets.length === 0 && (
                      <p className="text-[10px] text-gray-500">Upload GLB/GLTF files in the Assets tab first</p>
                    )}
                  </div>

                  {/* Placed Meshes List */}
                  <div className="space-y-2">
                    <label className="text-xs text-gray-400 font-medium">
                      Placed Meshes ({placedMeshes.length})
                    </label>
                    {placedMeshes.length === 0 ? (
                      <p className="text-xs text-gray-500 text-center py-4">
                        No meshes placed. Add some from the Assets tab!
                      </p>
                    ) : (
                      <div className="space-y-1 max-h-[300px] overflow-y-auto">
                        {placedMeshes.map((mesh) => (
                          <PlacedMeshItem
                            key={mesh.id}
                            mesh={mesh}
                            isSelected={selectedMeshId === mesh.id}
                            onSelect={() => handleSelectMesh(mesh.id)}
                            onDeselect={handleDeselectMesh}
                            onRemove={() => handleRemoveMesh(mesh.id)}
                            onDuplicate={() => handleDuplicateMesh(mesh.id)}
                            onRename={(displayName) => handleRenameMesh(mesh.id, displayName)}
                          />
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* ============================================================ */}
              {/* SKYBOX TAB */}
              {/* ============================================================ */}
              {activeTab === 'skybox' && (
                <div className="space-y-4">
                  {/* Skybox Selection */}
                  <div className="space-y-2">
                    <label className="text-xs text-gray-400 font-medium">Skybox HDR</label>
                    {skyboxAssetsLoading ? (
                      <div className="flex items-center justify-center py-4">
                        <Loader2 size={20} className="animate-spin text-gray-400" />
                      </div>
                    ) : skyboxAssets.length === 0 ? (
                      <p className="text-xs text-gray-500 text-center py-4">
                        No skybox HDRs uploaded. Go to Assets tab to upload.
                      </p>
                    ) : (
                      <div className="space-y-1 max-h-[200px] overflow-y-auto">
                        {skyboxAssets.map((asset) => (
                          <button
                            key={asset.id}
                            onClick={() => handleSelectSkybox(asset.id)}
                            className={`w-full flex items-center gap-2 p-2 rounded border text-left ${
                              selectedSkyboxId === asset.id
                                ? 'border-violet-500 bg-violet-900/20'
                                : 'border-gray-700 bg-gray-800/50 hover:bg-gray-800'
                            }`}
                          >
                            <Cloud size={14} className={selectedSkyboxId === asset.id ? 'text-violet-400' : 'text-gray-400'} />
                            <span className="text-xs text-white flex-1 truncate">{asset.name}</span>
                            {selectedSkyboxId === asset.id && <Check size={14} className="text-violet-400" />}
                          </button>
                        ))}
                      </div>
                    )}
                  </div>

                  {/* Exposure Slider */}
                  <div className="space-y-2">
                    <div className="flex justify-between text-xs">
                      <span className="text-gray-400 font-medium">Exposure</span>
                      <span className="text-gray-500 font-mono">{exposure.toFixed(2)}</span>
                    </div>
                    <Slider
                      min={0.1}
                      max={3}
                      step={0.05}
                      value={[exposure]}
                      onValueChange={(v: number[]) => handleExposureChange(v[0])}
                    />
                    <p className="text-[10px] text-gray-500">HDR tone mapping brightness</p>
                  </div>

                  {/* Skybox Rotation Sliders (US1) - 3-axis rotation */}
                  <div className="space-y-3">
                    <span className="text-xs text-gray-400 font-medium">Rotation</span>

                    {/* X-axis rotation */}
                    <div className="space-y-1">
                      <div className="flex justify-between text-xs">
                        <span className="text-gray-500">X (Pitch)</span>
                        <span className="text-gray-500 font-mono">{skyboxRotationX.toFixed(0)}°</span>
                      </div>
                      <Slider
                        min={0}
                        max={360}
                        step={1}
                        value={[skyboxRotationX]}
                        onValueChange={(v: number[]) => handleSkyboxRotationXChange(v[0])}
                      />
                    </div>

                    {/* Y-axis rotation */}
                    <div className="space-y-1">
                      <div className="flex justify-between text-xs">
                        <span className="text-gray-500">Y (Yaw)</span>
                        <span className="text-gray-500 font-mono">{skyboxRotationY.toFixed(0)}°</span>
                      </div>
                      <Slider
                        min={0}
                        max={360}
                        step={1}
                        value={[skyboxRotationY]}
                        onValueChange={(v: number[]) => handleSkyboxRotationYChange(v[0])}
                      />
                    </div>

                    {/* Z-axis rotation */}
                    <div className="space-y-1">
                      <div className="flex justify-between text-xs">
                        <span className="text-gray-500">Z (Roll)</span>
                        <span className="text-gray-500 font-mono">{skyboxRotationZ.toFixed(0)}°</span>
                      </div>
                      <Slider
                        min={0}
                        max={360}
                        step={1}
                        value={[skyboxRotationZ]}
                        onValueChange={(v: number[]) => handleSkyboxRotationZChange(v[0])}
                      />
                    </div>

                    <p className="text-[10px] text-gray-500">Rotate skybox on all 3 axes</p>
                  </div>

                  {/* Skybox Animation Toggle (US2) */}
                  <div className="space-y-2">
                    <div className="flex items-center justify-between">
                      <span className="text-xs text-gray-400 font-medium">Animate Rotation</span>
                      <button
                        onClick={() => handleSkyboxAnimationToggle(!skyboxAnimationEnabled)}
                        className={`px-3 py-1 text-xs rounded transition-colors ${skyboxAnimationEnabled ? "bg-violet-600 text-white" : "bg-gray-700 text-gray-400 hover:bg-gray-600"}`}
                      >
                        {skyboxAnimationEnabled ? "ON" : "OFF"}
                      </button>
                    </div>
                    {skyboxAnimationEnabled && (
                      <div className="space-y-2">
                        <div className="flex justify-between text-xs">
                          <span className="text-gray-400 font-medium">Speed</span>
                          <span className="text-gray-500 font-mono">{skyboxAnimationSpeed.toFixed(1)}°/s</span>
                        </div>
                        <Slider
                          min={0.1}
                          max={10}
                          step={0.1}
                          value={[skyboxAnimationSpeed]}
                          onValueChange={(v: number[]) => handleSkyboxAnimationSpeedChange(v[0])}
                        />
                        <p className="text-[10px] text-gray-500">Rotation speed in degrees per second</p>
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* ============================================================ */}
              {/* TERRAIN TAB (Dev Mode Lot 3 - US3/US4) */}
              {/* ============================================================ */}
              {activeTab === "terrain" && (
                <div className="space-y-4">
                  {/* Arena Decoration Toggle */}
                  <div className="flex items-center justify-between">
                    <div>
                      <span className="text-xs text-gray-400 font-medium">Arena Decoration</span>
                      <p className="text-[10px] text-gray-500">Stands, stadium surroundings</p>
                    </div>
                    <button
                      onClick={() => {
                        const newValue = !showArenaDecor;
                        setShowArenaDecor(newValue);
                        devToolsManager?.arenaManager?.setArenaDecorVisible(newValue);
                      }}
                      className={`px-3 py-1 text-xs rounded transition-colors ${showArenaDecor ? "bg-green-600 text-white" : "bg-gray-700 text-gray-400 hover:bg-gray-600"}`}
                    >
                      {showArenaDecor ? "VISIBLE" : "HIDDEN"}
                    </button>
                  </div>

                  {/* Ground Enable Toggle */}
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-gray-400 font-medium">Ground Plane</span>
                    <button
                      onClick={() => handleGroundEnabledChange(!groundEnabled)}
                      className={`px-3 py-1 text-xs rounded transition-colors ${groundEnabled ? "bg-green-600 text-white" : "bg-gray-700 text-gray-400 hover:bg-gray-600"}`}
                    >
                      {groundEnabled ? "ENABLED" : "DISABLED"}
                    </button>
                  </div>

                  {groundEnabled && (
                    <>
                      {/* Ground Texture Selection - Combobox */}
                      <div className="space-y-2">
                        <label className="text-xs text-gray-400 font-medium">Ground Texture</label>
                        {groundTextureLoading ? (
                          <div className="flex items-center justify-center py-2">
                            <Loader2 size={16} className="animate-spin text-gray-400" />
                          </div>
                        ) : groundTextureAssets.length === 0 ? (
                          <p className="text-xs text-gray-500 text-center py-2">
                            No ground textures. Upload in Assets tab.
                          </p>
                        ) : (
                          <select
                            value={groundTextureId || ''}
                            onChange={(e) => handleGroundTextureChange(e.target.value)}
                            className="w-full px-3 py-2 text-xs bg-gray-800 border border-gray-700 rounded text-white focus:border-green-500 focus:outline-none"
                          >
                            <option value="" disabled>Select a texture...</option>
                            {groundTextureAssets.map((asset) => (
                              <option key={asset.id} value={asset.id}>
                                {asset.name}
                              </option>
                            ))}
                          </select>
                        )}
                      </div>

                      {/* Ground Height Slider */}
                      <div className="space-y-2">
                        <div className="flex justify-between text-xs">
                          <span className="text-gray-400 font-medium">Height</span>
                          <span className="text-gray-500 font-mono">{groundHeight.toFixed(0)}</span>
                        </div>
                        <Slider
                          min={-2500}
                          max={2500}
                          step={25}
                          value={[groundHeight]}
                          onValueChange={(v: number[]) => handleGroundHeightChange(v[0])}
                        />
                        <p className="text-[10px] text-gray-500">Adjust ground plane height</p>
                      </div>

                      {/* Texture Scale */}
                      <div className="space-y-2">
                        <div className="flex justify-between text-xs">
                          <span className="text-gray-400 font-medium">Texture Scale</span>
                          <span className="text-gray-500 font-mono">{groundRepeatX}x{groundRepeatY}</span>
                        </div>
                        <Slider
                          min={1}
                          max={50}
                          step={1}
                          value={[groundRepeatX]}
                          onValueChange={(v: number[]) => handleGroundRepeatChange(v[0], v[0])}
                        />
                      </div>
                    </>
                  )}

                  {/* Terraforming Controls (US4) */}
                  <div className="space-y-3 border-t border-gray-700 pt-3">
                    <div className="flex items-center justify-between">
                      <span className="text-xs text-gray-400 font-medium">Terraforming Mode</span>
                      <button
                        onClick={() => handleTerraformingToggle(!terraformingEnabled)}
                        className={`px-3 py-1 text-xs rounded transition-colors ${terraformingEnabled ? "bg-green-600 text-white" : "bg-gray-700 text-gray-400 hover:bg-gray-600"}`}
                      >
                        {terraformingEnabled ? "ACTIVE" : "OFF"}
                      </button>
                    </div>

                    {terraformingEnabled && (
                      <>
                        <p className="text-[10px] text-green-400 bg-green-900/30 p-2 rounded">
                          Click+drag to raise terrain. Hold Shift to lower.
                        </p>

                        {/* Brush Size */}
                        <div className="space-y-2">
                          <div className="flex justify-between text-xs">
                            <span className="text-gray-400 font-medium">Brush Size</span>
                            <span className="text-gray-500 font-mono">{brushSize}</span>
                          </div>
                          <Slider
                            min={100}
                            max={5000}
                            step={100}
                            value={[brushSize]}
                            onValueChange={(v: number[]) => handleBrushSizeChange(v[0])}
                          />
                        </div>

                        {/* Brush Strength */}
                        <div className="space-y-2">
                          <div className="flex justify-between text-xs">
                            <span className="text-gray-400 font-medium">Brush Strength</span>
                            <span className="text-gray-500 font-mono">{brushStrength}</span>
                          </div>
                          <Slider
                            min={1}
                            max={100}
                            step={1}
                            value={[brushStrength]}
                            onValueChange={(v: number[]) => handleBrushStrengthChange(v[0])}
                          />
                        </div>
                      </>
                    )}
                  </div>

                  {/* Flatten Terrain Button */}
                  <div className="pt-2">
                    <button
                      onClick={() => {
                        handleFlattenTerrain();
                        setGroundHeight(0);
                      }}
                      className="w-full flex items-center justify-center gap-1.5 px-3 py-2 bg-gray-700 hover:bg-gray-600 text-white text-xs rounded transition-colors"
                    >
                      <RotateCcw size={14} />
                      Flatten Terrain
                    </button>
                  </div>
                </div>
              )}

              {/* ============================================================ */}
              {/* LIGHTS TAB */}
              {/* ============================================================ */}
              {activeTab === 'lights' && (
                <div className="space-y-3">
                  {/* Add Light Buttons */}
                  <div className="flex gap-2">
                    <button onClick={() => handleAddLight('ambient')} className="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 bg-purple-600 hover:bg-purple-500 text-white text-xs rounded">
                      <Plus size={12} /> Ambient
                    </button>
                    <button onClick={() => handleAddLight('hemisphere')} className="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs rounded">
                      <Plus size={12} /> Hemi
                    </button>
                  </div>
                  <div className="flex gap-2">
                    <button onClick={() => handleAddLight('point')} className="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 bg-yellow-600 hover:bg-yellow-500 text-white text-xs rounded">
                      <Plus size={12} /> Point
                    </button>
                    <button onClick={() => handleAddLight('spot')} className="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 bg-orange-600 hover:bg-orange-500 text-white text-xs rounded">
                      <Plus size={12} /> Spot
                    </button>
                    <button onClick={() => handleAddLight('directional')} className="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 bg-cyan-600 hover:bg-cyan-500 text-white text-xs rounded">
                      <Plus size={12} /> Dir
                    </button>
                  </div>

                  {/* Helpers Toggle */}
                  <div className="flex items-center justify-between px-1">
                    <span className="text-xs text-gray-400">Show Light Helpers</span>
                    <button
                      onClick={handleToggleHelpers}
                      className={`flex items-center gap-1 px-2 py-1 rounded text-xs ${showHelpers ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-400'}`}
                    >
                      {showHelpers ? <Eye size={12} /> : <EyeOff size={12} />}
                    </button>
                  </div>

                  {isPositioning && (
                    <div className="p-2 bg-violet-900/30 border border-violet-700 rounded text-xs text-violet-300">
                      Click in the scene to position the light
                    </div>
                  )}

                  {/* Lights List */}
                  <div className="space-y-2 max-h-[300px] overflow-y-auto">
                    {lights.length === 0 ? (
                      <p className="text-gray-500 text-xs text-center py-4">No custom lights. Add one above!</p>
                    ) : (
                      lights.map((light) => (
                        <LightItem
                          key={light.id}
                          light={light}
                          isSelected={selectedLightId === light.id}
                          transformMode={selectedLightId === light.id ? lightTransformMode : 'translate'}
                          onSelect={() => handleSelectLight(light.id)}
                          onDeselect={handleDeselectLight}
                          onChange={(prop, val) => handleLightChange(light.id, prop, val)}
                          onRemove={() => handleRemoveLight(light.id)}
                          onPosition={() => handlePositionLight(light.id)}
                          onSetTransformMode={handleSetLightTransformMode}
                        />
                      ))
                    )}
                  </div>
                </div>
              )}

              {/* ============================================================ */}
              {/* MATERIALS TAB */}
              {/* ============================================================ */}
              {activeTab === 'materials' && (
                <div className="space-y-3">
                  {/* Actions */}
                  <div className="flex gap-2">
                    <button onClick={handleScanMaterials} className="flex-1 px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-xs rounded">
                      Scan Scene
                    </button>
                    {isSelectingMaterial ? (
                      <button onClick={handleCancelMaterialSelection} className="flex items-center gap-1 px-3 py-1.5 bg-red-600 hover:bg-red-500 text-white text-xs rounded">
                        <X size={12} /> Cancel
                      </button>
                    ) : (
                      <button onClick={handleStartMaterialSelection} className="flex items-center gap-1 px-3 py-1.5 bg-green-600 hover:bg-green-500 text-white text-xs rounded">
                        <MousePointer size={12} /> Pick Mesh
                      </button>
                    )}
                  </div>

                  {isSelectingMaterial && (
                    <div className="p-2 bg-green-900/30 border border-green-700 rounded text-xs text-green-300">
                      Click on a mesh in the scene to select it
                    </div>
                  )}

                  {/* Search */}
                  {materials.length > 0 && (
                    <input
                      type="text"
                      placeholder="Search materials..."
                      value={materialSearchFilter}
                      onChange={(e) => setMaterialSearchFilter(e.target.value)}
                      className="w-full px-2 py-1.5 bg-gray-800 text-white text-xs rounded border border-gray-600 focus:border-violet-500 focus:outline-none"
                    />
                  )}

                  {/* Materials List */}
                  <div ref={materialsListRef} className="space-y-1 max-h-[300px] overflow-y-auto">
                    {filteredMaterials.length === 0 ? (
                      <p className="text-gray-500 text-xs text-center py-4">
                        {materials.length === 0 ? 'Click "Scan Scene" to find materials' : 'No materials match your search'}
                      </p>
                    ) : (
                      filteredMaterials.map((mat) => (
                        <MaterialItem
                          key={mat.id}
                          ref={(el) => {
                            if (el) materialItemRefs.current.set(mat.id, el);
                            else materialItemRefs.current.delete(mat.id);
                          }}
                          material={mat}
                          isExpanded={expandedMaterials.has(mat.id)}
                          isSelected={selectedMaterialId === mat.id}
                          onToggle={() => handleToggleMaterial(mat.id)}
                          onChange={(prop, val) => handleMaterialChange(mat.id, prop, val)}
                          onReset={() => handleResetMaterial(mat.id)}
                        />
                      ))
                    )}
                  </div>

                  {/* Reset All */}
                  {materials.length > 0 && (
                    <button onClick={handleResetAllMaterials} className="w-full px-3 py-1.5 bg-gray-700 hover:bg-gray-600 text-gray-300 text-xs rounded flex items-center justify-center gap-1">
                      <RotateCcw size={12} /> Reset All Materials
                    </button>
                  )}
                </div>
              )}

            </div>

              {/* Resize handle */}
              <div
                className="absolute bottom-1 right-1 w-4 h-4 cursor-se-resize opacity-50 hover:opacity-100 transition-opacity"
                onMouseDown={handleResizeStart}
              >
                <svg
                  className="w-full h-full text-violet-400"
                  viewBox="0 0 16 16"
                  fill="currentColor"
                >
                  <path d="M14 14v-2h-2v2h2zm0-4v-2h-2v2h2zm-4 4v-2H8v2h2zm0-4v-2H8v2h2zm-4 4v-2H4v2h2z" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

// ============================================================================
// SUB-COMPONENTS
// ============================================================================

interface AssetCardProps {
  asset: Asset;
  onAddToScene?: () => void;
  onDelete: () => void;
  onRename: (newName: string) => Promise<{ success: boolean; error?: string }>;
  isAdding: boolean;
}

function AssetCard({ asset, onAddToScene, onDelete, onRename, isAdding }: AssetCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editName, setEditName] = useState(asset.name);
  const [renameError, setRenameError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const typeColors: Record<string, string> = {
    mesh: 'bg-blue-600',
    skybox: 'bg-purple-600',
    ground_texture: 'bg-green-600',
  };

  const handleStartEdit = () => {
    setEditName(asset.name);
    setRenameError(null);
    setIsEditing(true);
    setTimeout(() => inputRef.current?.select(), 0);
  };

  const handleCancelEdit = () => {
    setIsEditing(false);
    setEditName(asset.name);
    setRenameError(null);
  };

  const handleConfirmEdit = async () => {
    const trimmedName = editName.trim();
    if (!trimmedName) {
      setRenameError('Name cannot be empty');
      return;
    }
    if (trimmedName === asset.name) {
      setIsEditing(false);
      return;
    }

    setIsSaving(true);
    setRenameError(null);
    const result = await onRename(trimmedName);
    setIsSaving(false);

    if (result.success) {
      setIsEditing(false);
    } else {
      setRenameError(result.error || 'Failed to rename');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleConfirmEdit();
    } else if (e.key === 'Escape') {
      handleCancelEdit();
    }
  };

  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-center gap-2 p-2 rounded border border-gray-700 bg-gray-800/50">
        <div className={`w-2 h-2 rounded-full ${typeColors[asset.type] || 'bg-gray-600'}`} />
        {isEditing ? (
          <input
            ref={inputRef}
            type="text"
            value={editName}
            onChange={(e) => setEditName(e.target.value)}
            onKeyDown={handleKeyDown}
            onBlur={handleCancelEdit}
            disabled={isSaving}
            className="flex-1 bg-gray-900 border border-gray-600 rounded px-1 py-0.5 text-xs text-white outline-none focus:border-violet-500"
            autoFocus
          />
        ) : (
          <span className="text-xs text-white flex-1 truncate">{asset.name}</span>
        )}
        <span className="text-[10px] text-gray-500">{(asset.fileSize / 1024).toFixed(0)}KB</span>
        {isEditing ? (
          <>
            <button
              onMouseDown={(e) => e.preventDefault()}
              onClick={handleConfirmEdit}
              disabled={isSaving}
              className="p-1 rounded bg-green-600 hover:bg-green-500 disabled:opacity-50 text-white"
              title="Save"
            >
              {isSaving ? <Loader2 size={12} className="animate-spin" /> : <Check size={12} />}
            </button>
            <button
              onMouseDown={(e) => e.preventDefault()}
              onClick={handleCancelEdit}
              disabled={isSaving}
              className="p-1 rounded bg-gray-600 hover:bg-gray-500 text-white"
              title="Cancel"
            >
              <X size={12} />
            </button>
          </>
        ) : (
          <>
            <button
              onClick={handleStartEdit}
              className="p-1 rounded bg-gray-700 text-gray-400 hover:text-white hover:bg-gray-600"
              title="Rename"
            >
              <Pencil size={12} />
            </button>
            {onAddToScene && (
              <button
                onClick={onAddToScene}
                disabled={isAdding}
                className="p-1 rounded bg-green-600 hover:bg-green-500 disabled:opacity-50 text-white"
                title="Add to scene"
              >
                <Plus size={12} />
              </button>
            )}
            <button
              onClick={onDelete}
              className="p-1 rounded bg-red-900/50 text-red-400 hover:bg-red-800"
              title="Delete"
            >
              <Trash2 size={12} />
            </button>
          </>
        )}
      </div>
      {renameError && (
        <p className="text-[10px] text-red-400 px-2">{renameError}</p>
      )}
    </div>
  );
}

interface PlacedMeshItemProps {
  mesh: PlacedMeshInfo;
  isSelected: boolean;
  onSelect: () => void;
  onDeselect: () => void;
  onRemove: () => void;
  onDuplicate: () => void;
  onRename: (displayName: string) => void;
}

function PlacedMeshItem({ mesh, isSelected, onSelect, onDeselect, onRemove, onDuplicate, onRename }: PlacedMeshItemProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editName, setEditName] = useState(mesh.displayName || mesh.name);
  const inputRef = useRef<HTMLInputElement>(null);

  const displayName = mesh.displayName || mesh.name;

  const handleStartEdit = () => {
    setEditName(displayName);
    setIsEditing(true);
    setTimeout(() => inputRef.current?.select(), 0);
  };

  const handleCancelEdit = () => {
    setIsEditing(false);
    setEditName(displayName);
  };

  const handleConfirmEdit = () => {
    const trimmedName = editName.trim();
    if (trimmedName && trimmedName !== displayName) {
      onRename(trimmedName);
    }
    setIsEditing(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleConfirmEdit();
    } else if (e.key === 'Escape') {
      handleCancelEdit();
    }
  };

  return (
    <div className={`flex items-center gap-2 p-2 rounded border ${isSelected ? 'border-violet-500 bg-violet-900/20' : 'border-gray-700 bg-gray-800/50'}`}>
      <button
        onClick={isSelected ? onDeselect : onSelect}
        className={`p-1 rounded ${isSelected ? 'bg-violet-600 text-white' : 'bg-gray-700 text-gray-400 hover:text-white'}`}
      >
        <Move size={12} />
      </button>
      {isEditing ? (
        <input
          ref={inputRef}
          type="text"
          value={editName}
          onChange={(e) => setEditName(e.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={handleConfirmEdit}
          className="flex-1 bg-gray-900 border border-gray-600 rounded px-1 py-0.5 text-xs text-white outline-none focus:border-violet-500"
          autoFocus
        />
      ) : (
        <span
          className="text-xs text-white flex-1 truncate cursor-pointer hover:text-violet-300"
          onClick={handleStartEdit}
          title="Click to rename"
        >
          {displayName}
          {mesh.displayName && mesh.displayName !== mesh.name && (
            <span className="text-[10px] text-gray-500 ml-1">({mesh.name})</span>
          )}
        </span>
      )}
      {isEditing ? (
        <>
          <button
            onMouseDown={(e) => e.preventDefault()}
            onClick={handleConfirmEdit}
            className="p-1 rounded bg-green-600 hover:bg-green-500 text-white"
            title="Save"
          >
            <Check size={12} />
          </button>
          <button
            onMouseDown={(e) => e.preventDefault()}
            onClick={handleCancelEdit}
            className="p-1 rounded bg-gray-600 hover:bg-gray-500 text-white"
            title="Cancel"
          >
            <X size={12} />
          </button>
        </>
      ) : (
        <>
          <button onClick={handleStartEdit} className="p-1 rounded bg-gray-700 text-gray-400 hover:text-white" title="Rename">
            <Pencil size={12} />
          </button>
          <button onClick={onDuplicate} className="p-1 rounded bg-gray-700 text-gray-400 hover:text-white" title="Duplicate">
            <Copy size={12} />
          </button>
          <button onClick={onRemove} className="p-1 rounded bg-red-900/50 text-red-400 hover:bg-red-800" title="Remove">
            <Trash2 size={12} />
          </button>
        </>
      )}
    </div>
  );
}

interface MaterialItemProps {
  material: MaterialInfo;
  isExpanded: boolean;
  isSelected: boolean;
  onToggle: () => void;
  onChange: (property: string, value: number | string | boolean) => void;
  onReset: () => void;
}

const MaterialItem = forwardRef<HTMLDivElement, MaterialItemProps>(
  ({ material, isExpanded, isSelected, onToggle, onChange, onReset }, ref) => {
    // Check if emissive is active (not black)
    const hasEmissive = material.emissive !== '#000000';

    return (
      <div ref={ref} className={`border rounded ${isSelected ? 'border-violet-500 bg-violet-900/20' : 'border-gray-700 bg-gray-800/50'}`}>
        <button onClick={onToggle} className="w-full flex flex-col gap-1 px-2 py-1.5 text-left hover:bg-gray-700/50">
          {/* Main row: expand icon, color swatch, name */}
          <div className="flex items-center gap-2 w-full">
            {isExpanded ? <ChevronDown size={12} className="text-gray-400 flex-shrink-0" /> : <ChevronRight size={12} className="text-gray-400 flex-shrink-0" />}
            <div className="w-3 h-3 rounded-sm border border-gray-600 flex-shrink-0" style={{ backgroundColor: material.color }} />
            {hasEmissive && (
              <div
                className="w-3 h-3 rounded-full flex-shrink-0 ring-1 ring-white/30"
                style={{ backgroundColor: material.emissive, boxShadow: `0 0 4px ${material.emissive}` }}
                title="Emissive color"
              />
            )}
            <span className="text-xs text-white truncate flex-1">
              {material.materialName || material.name}
            </span>
            {material.materialIndex > 0 && <span className="text-[10px] text-gray-500">[{material.materialIndex}]</span>}
          </div>
          {/* Path info (shown in collapsed view) */}
          {material.meshPath && (
            <div className="flex items-center gap-2 w-full pl-5">
              <span className="text-[9px] text-gray-500 truncate" title={material.meshPath}>
                📍 {material.meshPath}
              </span>
            </div>
          )}
        </button>
        {isExpanded && (
          <div className="px-3 pb-3 pt-1 space-y-3 border-t border-gray-700">
            {/* Material info header */}
            <div className="p-2 bg-gray-900/50 rounded text-[9px] space-y-1">
              <div className="flex items-start gap-1">
                <span className="text-gray-500 flex-shrink-0">Mesh:</span>
                <span className="text-gray-300 break-all">{material.name}</span>
              </div>
              {material.materialName && (
                <div className="flex items-start gap-1">
                  <span className="text-gray-500 flex-shrink-0">Material:</span>
                  <span className="text-cyan-400 break-all">{material.materialName}</span>
                </div>
              )}
              {material.meshPath && (
                <div className="flex items-start gap-1">
                  <span className="text-gray-500 flex-shrink-0">Path:</span>
                  <span className="text-gray-400 break-all">{material.meshPath}</span>
                </div>
              )}
            </div>

            {/* Base Properties Section */}
            <div className="space-y-2">
              <div className="text-[10px] text-gray-500 font-medium uppercase tracking-wider border-b border-gray-700 pb-1">
                Base Properties
              </div>

              {/* Color */}
              <div className="space-y-1">
                <span className="text-[10px] text-gray-400">Color</span>
                <div className="flex items-center gap-2">
                  <input type="color" value={material.color} onChange={(e) => onChange('color', e.target.value)} className="w-8 h-6 rounded cursor-pointer" />
                  <span className="text-[10px] text-gray-500 font-mono">{material.color}</span>
                </div>
              </div>

              {/* Roughness */}
              <div className="space-y-1">
                <div className="flex justify-between text-[10px]">
                  <span className="text-gray-400">Roughness</span>
                  <span className="text-gray-500 font-mono">{material.roughness.toFixed(2)}</span>
                </div>
                <Slider min={0} max={1} step={0.01} value={[material.roughness]} onValueChange={(v: number[]) => onChange('roughness', v[0])} />
              </div>

              {/* Metalness */}
              <div className="space-y-1">
                <div className="flex justify-between text-[10px]">
                  <span className="text-gray-400">Metalness</span>
                  <span className="text-gray-500 font-mono">{material.metalness.toFixed(2)}</span>
                </div>
                <Slider min={0} max={1} step={0.01} value={[material.metalness]} onValueChange={(v: number[]) => onChange('metalness', v[0])} />
              </div>
            </div>

            {/* Emissive Section */}
            <div className="space-y-2">
              <div className="text-[10px] text-gray-500 font-medium uppercase tracking-wider border-b border-gray-700 pb-1 flex items-center gap-2">
                Emissive
                {hasEmissive && <span className="text-[8px] text-violet-500">● ACTIVE</span>}
              </div>

              {/* Emissive Color */}
              <div className="space-y-1">
                <span className="text-[10px] text-gray-400">Emissive Color</span>
                <div className="flex items-center gap-2">
                  <input type="color" value={material.emissive} onChange={(e) => onChange('emissive', e.target.value)} className="w-8 h-6 rounded cursor-pointer" />
                  <span className="text-[10px] text-gray-500 font-mono">{material.emissive}</span>
                  {hasEmissive && (
                    <button
                      onClick={() => onChange('emissive', '#000000')}
                      className="text-[9px] text-gray-500 hover:text-gray-300 px-1 py-0.5 bg-gray-800 rounded"
                      title="Remove emissive"
                    >
                      Clear
                    </button>
                  )}
                </div>
              </div>

              {/* Emissive Intensity */}
              <div className="space-y-1">
                <div className="flex justify-between text-[10px]">
                  <span className="text-gray-400">Emissive Intensity</span>
                  <span className="text-gray-500 font-mono">{material.emissiveIntensity.toFixed(2)}</span>
                </div>
                <Slider min={0} max={5} step={0.1} value={[material.emissiveIntensity]} onValueChange={(v: number[]) => onChange('emissiveIntensity', v[0])} />
              </div>
            </div>

            {/* Environment & Transparency Section */}
            <div className="space-y-2">
              <div className="text-[10px] text-gray-500 font-medium uppercase tracking-wider border-b border-gray-700 pb-1">
                Advanced
              </div>

              {/* Environment Map Intensity */}
              <div className="space-y-1">
                <div className="flex justify-between text-[10px]">
                  <span className="text-gray-400">Env Map Intensity</span>
                  <span className="text-gray-500 font-mono">{material.envMapIntensity.toFixed(2)}</span>
                </div>
                <Slider min={0} max={3} step={0.05} value={[material.envMapIntensity]} onValueChange={(v: number[]) => onChange('envMapIntensity', v[0])} />
              </div>

              {/* Opacity */}
              <div className="space-y-1">
                <div className="flex justify-between text-[10px]">
                  <span className="text-gray-400">Opacity</span>
                  <span className="text-gray-500 font-mono">{material.opacity.toFixed(2)}</span>
                </div>
                <Slider min={0} max={1} step={0.01} value={[material.opacity]} onValueChange={(v: number[]) => onChange('opacity', v[0])} />
              </div>

              {/* Transparent toggle */}
              <div className="flex items-center justify-between">
                <span className="text-[10px] text-gray-400">Transparent</span>
                <button
                  onClick={() => onChange('transparent', !material.transparent)}
                  className={`px-2 py-0.5 rounded text-[10px] ${material.transparent ? 'bg-violet-600 text-white' : 'bg-gray-700 text-gray-400'}`}
                >
                  {material.transparent ? 'ON' : 'OFF'}
                </button>
              </div>
            </div>

            {/* Reset button */}
            <button onClick={onReset} className="w-full px-2 py-1 bg-gray-700 hover:bg-gray-600 text-gray-300 text-[10px] rounded flex items-center justify-center gap-1">
              <RotateCcw size={10} /> Reset All Properties
            </button>
          </div>
        )}
      </div>
    );
  }
);
MaterialItem.displayName = 'MaterialItem';

interface LightItemProps {
  light: LightInfo;
  isSelected: boolean;
  transformMode: 'translate' | 'rotate';
  onSelect: () => void;
  onDeselect: () => void;
  onChange: (property: string, value: unknown) => void;
  onRemove: () => void;
  onPosition: () => void;
  onSetTransformMode: (mode: 'translate' | 'rotate') => void;
}

function LightItem({ light, isSelected, transformMode, onSelect, onDeselect, onChange, onRemove, onPosition, onSetTransformMode }: LightItemProps) {
  const typeColors: Record<string, string> = {
    ambient: 'bg-purple-600',
    hemisphere: 'bg-indigo-600',
    point: 'bg-yellow-600',
    spot: 'bg-orange-600',
    directional: 'bg-cyan-600',
  };

  const hasPosition = light.position !== null && light.type !== 'ambient';
  const hasTarget = light.hasTarget;

  return (
    <div className={`border rounded ${isSelected ? 'border-violet-500 bg-violet-900/20' : 'border-gray-700 bg-gray-800/50'}`}>
      <div className="flex items-center gap-2 px-2 py-1.5">
        <div className={`w-2 h-2 rounded-full ${typeColors[light.type] || 'bg-gray-500'}`} />
        <span className="text-xs text-white flex-1 capitalize">{light.type}</span>
        {hasPosition && (
          <>
            <button onClick={isSelected ? onDeselect : onSelect} className={`p-1 rounded ${isSelected ? 'bg-violet-600 text-white' : 'bg-gray-700 text-gray-400 hover:text-white'}`}>
              <Move size={12} />
            </button>
            <button onClick={onPosition} className="p-1 rounded bg-gray-700 text-gray-400 hover:text-white">
              <MousePointer size={12} />
            </button>
          </>
        )}
        <button onClick={onRemove} className="p-1 rounded bg-red-900/50 text-red-400 hover:bg-red-800">
          <Trash2 size={12} />
        </button>
      </div>
      <div className="px-3 pb-3 pt-1 space-y-3 border-t border-gray-700">
        {isSelected && hasTarget && (
          <div className="space-y-1">
            <span className="text-[10px] text-gray-400">Gizmo Mode</span>
            <div className="flex gap-1">
              <button onClick={() => onSetTransformMode('translate')} className={`flex-1 flex items-center justify-center gap-1 px-2 py-1 rounded text-[10px] ${transformMode === 'translate' ? 'bg-violet-600 text-white' : 'bg-gray-700 text-gray-400'}`}>
                <Move size={10} /> Position
              </button>
              <button onClick={() => onSetTransformMode('rotate')} className={`flex-1 flex items-center justify-center gap-1 px-2 py-1 rounded text-[10px] ${transformMode === 'rotate' ? 'bg-violet-600 text-white' : 'bg-gray-700 text-gray-400'}`}>
                <Target size={10} /> Direction
              </button>
            </div>
          </div>
        )}
        {hasPosition && light.position && (
          <div className="space-y-1">
            <span className="text-[10px] text-gray-400">Position</span>
            <div className="grid grid-cols-3 gap-1 text-[10px] text-center">
              <div><span className="text-red-400">X:</span><span className="text-gray-300 font-mono ml-1">{light.position.x.toFixed(1)}</span></div>
              <div><span className="text-green-400">Y:</span><span className="text-gray-300 font-mono ml-1">{light.position.y.toFixed(1)}</span></div>
              <div><span className="text-blue-400">Z:</span><span className="text-gray-300 font-mono ml-1">{light.position.z.toFixed(1)}</span></div>
            </div>
          </div>
        )}
        <div className="space-y-1">
          <div className="flex justify-between text-[10px]">
            <span className="text-gray-400">Intensity</span>
            <span className="text-gray-500 font-mono">{light.intensity.toFixed(1)}</span>
          </div>
          <Slider
            min={0}
            max={light.type === 'spot' ? 100000 : light.type === 'directional' ? 100 : light.type === 'point' ? 5000 : 100}
            step={light.type === 'spot' ? 10 : light.type === 'point' ? 5 : 0.5}
            value={[light.intensity]}
            onValueChange={(v: number[]) => onChange('intensity', v[0])}
          />
        </div>
        <div className="space-y-1">
          <span className="text-[10px] text-gray-400">{light.type === 'hemisphere' ? 'Sky Color' : 'Color'}</span>
          <div className="flex items-center gap-2">
            <input type="color" value={light.color} onChange={(e) => onChange(light.type === 'hemisphere' ? 'skyColor' : 'color', e.target.value)} className="w-8 h-6 rounded cursor-pointer" />
            <span className="text-[10px] text-gray-500 font-mono">{light.color}</span>
          </div>
        </div>
        {light.groundColor !== null && (
          <div className="space-y-1">
            <span className="text-[10px] text-gray-400">Ground Color</span>
            <div className="flex items-center gap-2">
              <input type="color" value={light.groundColor} onChange={(e) => onChange('groundColor', e.target.value)} className="w-8 h-6 rounded cursor-pointer" />
              <span className="text-[10px] text-gray-500 font-mono">{light.groundColor}</span>
            </div>
          </div>
        )}
        {light.distance !== null && (
          <div className="space-y-1">
            <div className="flex justify-between text-[10px]">
              <span className="text-gray-400">Distance</span>
              <span className="text-gray-500 font-mono">{light.distance.toFixed(0)}</span>
            </div>
            <Slider min={1} max={40000} step={100} value={[light.distance]} onValueChange={(v: number[]) => onChange('distance', v[0])} />
          </div>
        )}
        {light.angle !== null && (
          <div className="space-y-1">
            <div className="flex justify-between text-[10px]">
              <span className="text-gray-400">Angle</span>
              <span className="text-gray-500 font-mono">{((light.angle * 180) / Math.PI).toFixed(0)}°</span>
            </div>
            <Slider min={0.1} max={1.57} step={0.01} value={[light.angle]} onValueChange={(v: number[]) => onChange('angle', v[0])} />
          </div>
        )}
        {light.penumbra !== null && (
          <div className="space-y-1">
            <div className="flex justify-between text-[10px]">
              <span className="text-gray-400">Penumbra</span>
              <span className="text-gray-500 font-mono">{light.penumbra.toFixed(2)}</span>
            </div>
            <Slider min={0} max={1} step={0.01} value={[light.penumbra]} onValueChange={(v: number[]) => onChange('penumbra', v[0])} />
          </div>
        )}
        {/* Shadow controls for directional and spot lights */}
        {light.shadow !== null && (
          <div className="space-y-2 border-t border-gray-700 pt-2 mt-2">
            <div className="flex items-center justify-between">
              <span className="text-[10px] text-gray-400 font-medium">Shadows</span>
              <label className="flex items-center gap-1.5 cursor-pointer">
                <input
                  type="checkbox"
                  checked={light.shadow.enabled}
                  onChange={(e) => onChange('castShadow', e.target.checked)}
                  className="w-3 h-3 rounded border-gray-600 bg-gray-700 text-violet-500 focus:ring-violet-500"
                />
                <span className="text-[10px] text-gray-400">{light.shadow.enabled ? 'On' : 'Off'}</span>
              </label>
            </div>
            {light.shadow.enabled && (
              <>
                <div className="space-y-1">
                  <div className="flex justify-between text-[10px]">
                    <span className="text-gray-400">Map Size</span>
                    <span className="text-gray-500 font-mono">{light.shadow.mapSize}</span>
                  </div>
                  <select
                    value={light.shadow.mapSize}
                    onChange={(e) => onChange('shadowMapSize', parseInt(e.target.value))}
                    className="w-full bg-gray-700 border border-gray-600 rounded text-[10px] text-white px-2 py-1"
                  >
                    <option value={512}>512 (Low)</option>
                    <option value={1024}>1024 (Medium)</option>
                    <option value={2048}>2048 (High)</option>
                    <option value={4096}>4096 (Ultra)</option>
                  </select>
                </div>
                <div className="space-y-1">
                  <div className="flex justify-between text-[10px]">
                    <span className="text-gray-400">Bias</span>
                    <span className="text-gray-500 font-mono">{light.shadow.bias.toFixed(5)}</span>
                  </div>
                  <Slider min={-0.01} max={0.01} step={0.0001} value={[light.shadow.bias]} onValueChange={(v: number[]) => onChange('shadowBias', v[0])} />
                </div>
                <div className="space-y-1">
                  <div className="flex justify-between text-[10px]">
                    <span className="text-gray-400">Normal Bias</span>
                    <span className="text-gray-500 font-mono">{light.shadow.normalBias.toFixed(2)}</span>
                  </div>
                  <Slider min={0} max={1} step={0.01} value={[light.shadow.normalBias]} onValueChange={(v: number[]) => onChange('shadowNormalBias', v[0])} />
                </div>
                {/* Directional light camera bounds */}
                {light.type === 'directional' && light.shadow.cameraLeft !== null && (
                  <div className="space-y-2 border-t border-gray-600 pt-2">
                    <span className="text-[10px] text-gray-500">Shadow Camera Bounds</span>
                    <div className="grid grid-cols-2 gap-2">
                      <div className="space-y-1">
                        <span className="text-[9px] text-gray-500">Left / Right</span>
                        <div className="flex gap-1">
                          <input
                            type="number"
                            value={light.shadow.cameraLeft}
                            onChange={(e) => onChange('shadowCameraLeft', parseFloat(e.target.value))}
                            className="w-full bg-gray-700 border border-gray-600 rounded text-[10px] text-white px-1 py-0.5"
                          />
                          <input
                            type="number"
                            value={light.shadow.cameraRight ?? 0}
                            onChange={(e) => onChange('shadowCameraRight', parseFloat(e.target.value))}
                            className="w-full bg-gray-700 border border-gray-600 rounded text-[10px] text-white px-1 py-0.5"
                          />
                        </div>
                      </div>
                      <div className="space-y-1">
                        <span className="text-[9px] text-gray-500">Top / Bottom</span>
                        <div className="flex gap-1">
                          <input
                            type="number"
                            value={light.shadow.cameraTop ?? 0}
                            onChange={(e) => onChange('shadowCameraTop', parseFloat(e.target.value))}
                            className="w-full bg-gray-700 border border-gray-600 rounded text-[10px] text-white px-1 py-0.5"
                          />
                          <input
                            type="number"
                            value={light.shadow.cameraBottom ?? 0}
                            onChange={(e) => onChange('shadowCameraBottom', parseFloat(e.target.value))}
                            className="w-full bg-gray-700 border border-gray-600 rounded text-[10px] text-white px-1 py-0.5"
                          />
                        </div>
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-2">
                      <div className="space-y-1">
                        <span className="text-[9px] text-gray-500">Near</span>
                        <input
                          type="number"
                          value={light.shadow.cameraNear}
                          onChange={(e) => onChange('shadowCameraNear', parseFloat(e.target.value))}
                          className="w-full bg-gray-700 border border-gray-600 rounded text-[10px] text-white px-1 py-0.5"
                        />
                      </div>
                      <div className="space-y-1">
                        <span className="text-[9px] text-gray-500">Far</span>
                        <input
                          type="number"
                          value={light.shadow.cameraFar}
                          onChange={(e) => onChange('shadowCameraFar', parseFloat(e.target.value))}
                          className="w-full bg-gray-700 border border-gray-600 rounded text-[10px] text-white px-1 py-0.5"
                        />
                      </div>
                    </div>
                  </div>
                )}
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
