import { createBoostPadOverlayPlugin, type ReplayPlayer } from "@rlrml/player";

interface StandalonePluginControllerDeps {
  getReplayPlayer(): ReplayPlayer | null;
}

export interface StandalonePluginController {
  clear(): void;
  isBoostPadOverlayEnabled(): boolean;
  reset(): void;
  setBoostPadOverlayEnabled(enabled: boolean): void;
  syncBoostPadOverlayPlugin(): void;
  toggleBoostPadOverlay(): void;
}

export function createStandalonePluginController(
  deps: StandalonePluginControllerDeps,
): StandalonePluginController {
  const pluginRemovers = new Map<string, () => void>();
  let boostPadOverlayEnabled = true;

  function clear(): void {
    for (const removePlugin of pluginRemovers.values()) {
      removePlugin();
    }
    pluginRemovers.clear();
  }

  function syncBoostPadOverlayPlugin(): void {
    pluginRemovers.get("boost-pad-overlay")?.();
    pluginRemovers.delete("boost-pad-overlay");

    const replayPlayer = deps.getReplayPlayer();
    if (!replayPlayer || !boostPadOverlayEnabled) {
      return;
    }

    pluginRemovers.set("boost-pad-overlay", replayPlayer.addPlugin(createBoostPadOverlayPlugin()));
  }

  return {
    clear,
    isBoostPadOverlayEnabled() {
      return boostPadOverlayEnabled;
    },
    reset() {
      clear();
      boostPadOverlayEnabled = true;
    },
    setBoostPadOverlayEnabled(enabled) {
      boostPadOverlayEnabled = enabled;
    },
    syncBoostPadOverlayPlugin,
    toggleBoostPadOverlay() {
      boostPadOverlayEnabled = !boostPadOverlayEnabled;
      syncBoostPadOverlayPlugin();
    },
  };
}
