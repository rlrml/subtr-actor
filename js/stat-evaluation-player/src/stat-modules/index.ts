import {
  createFiftyFiftyModule,
  createPossessionModule,
  createPressureModule,
  createRushModule,
} from "./teamModules.ts";
import {
  createBackboardModule,
  createBallCarryModule,
  createBoostModule,
  createCeilingShotModule,
  createCoreModule,
  createDemoModule,
  createDodgeResetModule,
  createDoubleTapModule,
  createFlickModule,
  createMovementModule,
  createMustyFlickModule,
  createPowerslideModule,
  createSpeedFlipModule,
  createTouchModule,
  createWavedashModule,
  createWhiffModule,
} from "./playerModules.ts";
import {
  createAbsolutePositioningModule,
  createRelativePositioningModule,
} from "./positioningModules.ts";
import type { BoostPickupFilterController } from "../boostPickupFilters.ts";

export {
  hasBoostPickupAnimationTimelineMatch,
} from "../boostPickupFilters.ts";
export {
  DEPTH_ROLE_LABELS,
  getCurrentDepthRole,
  getCurrentRole,
  getStatsPlayerSnapshot,
  getTeamClass,
  RELATIVE_POSITIONING_MODULE_ID,
  ROLE_LABELS,
} from "./types.ts";
export type {
  DepthRole,
  Role,
  StatModule,
  StatModuleContext,
  StatModuleRuntime,
} from "./types.ts";

export function createStatModules(
  runtime: import("./types.ts").StatModuleRuntime,
  options: { boostPickupFilters?: BoostPickupFilterController } = {},
) {
  return [
    createCoreModule(),
    createBackboardModule(),
    createCeilingShotModule(),
    createDoubleTapModule(),
    createPossessionModule(runtime),
    createFiftyFiftyModule(),
    createPressureModule(),
    createRushModule(),
    createRelativePositioningModule(),
    createAbsolutePositioningModule(),
    createSpeedFlipModule(),
    createWavedashModule(),
    createTouchModule(runtime),
    createWhiffModule(),
    createFlickModule(),
    createMustyFlickModule(),
    createDodgeResetModule(),
    createBoostModule(runtime, options.boostPickupFilters),
    createBallCarryModule(),
    createMovementModule(runtime),
    createPowerslideModule(),
    createDemoModule(),
  ];
}
