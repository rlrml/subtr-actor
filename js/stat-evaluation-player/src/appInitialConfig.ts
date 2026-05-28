import {
  getStatsPlayerConfigFromLocation,
  getStatsPlayerConfigParamSnapshot,
  isStatsPlayerConfigDebugEnabled,
  type StatsPlayerConfig,
} from "./playerConfig.ts";
import { logStatsPlayerConfigLoadDebug } from "./playerConfigDebug.ts";

interface LoadInitialStatsPlayerConfigOptions {
  readonly initialConfig?: StatsPlayerConfig | null;
  readonly location: Location;
  setStatus(message: string): void;
}

export function loadInitialStatsPlayerConfig({
  initialConfig,
  location,
  setStatus,
}: LoadInitialStatsPlayerConfigOptions): StatsPlayerConfig | null {
  if (initialConfig !== undefined) {
    return initialConfig;
  }

  const configParamSnapshot = getStatsPlayerConfigParamSnapshot(location);
  const configDebugEnabled = isStatsPlayerConfigDebugEnabled(location);
  let configLoadError: unknown = null;
  let loadedConfig: StatsPlayerConfig | null = null;
  try {
    loadedConfig = getStatsPlayerConfigFromLocation(location);
  } catch (error) {
    configLoadError = error;
    console.error("Invalid stats player config:", error);
    setStatus(error instanceof Error ? error.message : "Invalid stats player config");
  }
  if (configDebugEnabled) {
    logStatsPlayerConfigLoadDebug(configParamSnapshot, loadedConfig, configLoadError);
  }

  return loadedConfig;
}
