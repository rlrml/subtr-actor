import { getStatsPlayerConfigSnapshot } from "./appConfigSnapshot.ts";
import { setStatsPlayerConfigOnUrl } from "./playerConfig.ts";

type StatsPlayerConfigSnapshotOptions = Parameters<typeof getStatsPlayerConfigSnapshot>[0];

interface StatsPlayerConfigUrlSyncControllerDeps {
  getLocation(): Location;
  getSnapshotOptions(): StatsPlayerConfigSnapshotOptions;
  replaceUrl(url: URL): void;
}

export interface StatsPlayerConfigUrlSyncController {
  reset(): void;
  schedule(): void;
  setApplyingConfig(isApplying: boolean): void;
}

export function createStatsPlayerConfigUrlSyncController(
  deps: StatsPlayerConfigUrlSyncControllerDeps,
): StatsPlayerConfigUrlSyncController {
  let isApplyingConfig = false;
  let configUrlUpdateTimer: number | null = null;

  function reset(): void {
    if (configUrlUpdateTimer !== null) {
      window.clearTimeout(configUrlUpdateTimer);
      configUrlUpdateTimer = null;
    }
    isApplyingConfig = false;
  }

  return {
    reset,
    schedule(): void {
      if (isApplyingConfig) {
        return;
      }
      if (configUrlUpdateTimer !== null) {
        window.clearTimeout(configUrlUpdateTimer);
      }
      configUrlUpdateTimer = window.setTimeout(() => {
        configUrlUpdateTimer = null;
        const nextUrl = setStatsPlayerConfigOnUrl(
          new URL(deps.getLocation().href),
          getStatsPlayerConfigSnapshot(deps.getSnapshotOptions()),
        );
        deps.replaceUrl(nextUrl);
      }, 150);
    },
    setApplyingConfig(isApplying): void {
      isApplyingConfig = isApplying;
    },
  };
}
