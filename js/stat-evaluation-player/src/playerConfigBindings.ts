import { applyConfigAdapterSnapshot } from "./configAdapters.ts";
import type { FloatingWindowController } from "./floatingWindows.ts";
import type { RecordingWindowController } from "./recordingWindow.ts";
import type { ActiveModulesRuntime } from "./activeModulesRuntime.ts";
import type { CameraControlsController } from "./cameraControls.ts";
import type { StatModule } from "./statModules.ts";
import type { StatsWindowsController } from "./statsWindows.ts";
import {
  setStatsPlayerConfigOnUrl,
  type StatsPlayerConfig,
  type StatsWindowConfig,
} from "./playerConfig.ts";
import {
  getCameraConfigSnapshot,
  getConfigAdapters,
  getModuleConfigSnapshot,
  getPlaybackConfigSnapshot,
  getStatsPlayerConfigSnapshot,
} from "./playerConfigRuntime.ts";
import type { StatsReplayPlayer } from "./statsReplayPlayer.ts";
import { formatPlaybackRate, snapPlaybackRate } from "./playbackRateControl.ts";

export interface PlayerConfigBindings {
  setApplyingConfig(value: boolean): void;
  reset(): void;
  scheduleConfigUrlUpdate(): void;
  applyConfigToStaticControls(config: StatsPlayerConfig): void;
}

export interface PlayerConfigBindingsOptions {
  readonly modules: readonly StatModule[];
  readonly playbackRate: HTMLInputElement;
  readonly playbackRateReadout: HTMLElement;
  readonly skipPostGoalTransitions: HTMLInputElement;
  readonly skipKickoffs: HTMLInputElement;
  readonly hitboxWireframes: HTMLInputElement;
  readonly hitboxOnlyMode: HTMLInputElement;
  getReplayPlayer(): StatsReplayPlayer | null;
  getCameraControlsController(): CameraControlsController | null;
  getRecordingWindowController(): RecordingWindowController | null;
  getFloatingWindowController(): FloatingWindowController | null;
  getStatsWindowsController(): StatsWindowsController | null;
  getActiveModulesRuntime(): ActiveModulesRuntime;
  getInitialConfig(): StatsPlayerConfig | null;
  renderModuleSummary(): void;
  renderModuleSettings(): void;
  renderTimelineEventCount(): void;
}

export function createPlayerConfigBindings(
  options: PlayerConfigBindingsOptions,
): PlayerConfigBindings {
  let isApplyingConfig = false;
  let configUrlUpdateTimer: number | null = null;

  const getSingletonWindowConfigs = () =>
    options.getFloatingWindowController()?.getSingletonConfigs() ?? [];

  const getStatsWindowConfigs = (): StatsWindowConfig[] =>
    options.getStatsWindowsController()?.getConfigs() ?? [];

  const getConfigSnapshot = (): StatsPlayerConfig => {
    const activeModulesRuntime = options.getActiveModulesRuntime();
    const replayPlayer = options.getReplayPlayer();

    return getStatsPlayerConfigSnapshot({
      playback: getPlaybackConfigSnapshot({
        replayPlayer,
        playbackRate: options.playbackRate,
        skipPostGoalTransitions: options.skipPostGoalTransitions,
        skipKickoffs: options.skipKickoffs,
      }),
      camera: getCameraConfigSnapshot({
        replayPlayer,
        cameraControlsController: options.getCameraControlsController(),
      }),
      activeTimelineEventSourceIds: activeModulesRuntime.getActiveTimelineEventSourceIds(),
      activeTimelineRangeModuleIds: activeModulesRuntime.getActiveTimelineRangeModuleIds(),
      activeMechanicTimelineKinds: activeModulesRuntime.getActiveMechanicTimelineKinds(),
      activeRenderEffectModuleIds: activeModulesRuntime.getActiveRenderEffectModuleIds(),
      initialConfig: options.getInitialConfig(),
      replayPlayer,
      boostPadOverlayEnabled: activeModulesRuntime.getBoostPadOverlayEnabled(),
      recording: options.getRecordingWindowController()?.getConfigSnapshot() ?? {},
      singletonWindows: getSingletonWindowConfigs(),
      statsWindows: getStatsWindowConfigs(),
      moduleConfigs: getModuleConfigSnapshot(options.modules),
    });
  };

  const applyModuleConfigSnapshot = (configs: Record<string, unknown>) => {
    applyConfigAdapterSnapshot(getConfigAdapters(options.modules), configs);
  };

  return {
    setApplyingConfig(value) {
      isApplyingConfig = value;
    },

    reset() {
      if (configUrlUpdateTimer !== null) {
        window.clearTimeout(configUrlUpdateTimer);
        configUrlUpdateTimer = null;
      }
      isApplyingConfig = false;
    },

    scheduleConfigUrlUpdate() {
      if (isApplyingConfig) {
        return;
      }
      if (configUrlUpdateTimer !== null) {
        window.clearTimeout(configUrlUpdateTimer);
      }
      configUrlUpdateTimer = window.setTimeout(() => {
        configUrlUpdateTimer = null;
        const nextUrl = setStatsPlayerConfigOnUrl(
          new URL(window.location.href),
          getConfigSnapshot(),
        );
        window.history.replaceState(window.history.state, "", nextUrl);
      }, 150);
    },

    applyConfigToStaticControls(config) {
      options.getActiveModulesRuntime().applyOverlayConfig(config.overlays);
      options.skipPostGoalTransitions.checked =
        config.playback.skipPostGoalTransitions ?? options.skipPostGoalTransitions.checked;
      options.skipKickoffs.checked = config.playback.skipKickoffs ?? options.skipKickoffs.checked;
      options.hitboxWireframes.checked = config.overlays.hitboxWireframes;
      options.hitboxOnlyMode.checked = config.overlays.hitboxOnlyMode;
      options.getCameraControlsController()?.applyNameplateLiftUu(config.camera.nameplateLiftUu);
      options
        .getCameraControlsController()
        ?.setAutoPossessionEnabled(config.camera.autoPossession ?? false, {
          requestConfigSync: false,
        });
      if (config.playback.rate !== undefined) {
        const playbackRate = snapPlaybackRate(config.playback.rate);
        options.playbackRate.value = `${playbackRate}`;
        options.playbackRateReadout.textContent = formatPlaybackRate(playbackRate);
      }
      options.getRecordingWindowController()?.applyConfig(config.recording);
      applyModuleConfigSnapshot(config.moduleConfigs);
      options.getFloatingWindowController()?.applySingletonConfigs(config.singletonWindows);
      options.getStatsWindowsController()?.replaceFromConfig(config.statsWindows);
      options.renderModuleSummary();
      options.renderModuleSettings();
      options.renderTimelineEventCount();
    },
  };
}
