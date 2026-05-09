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
  createMovementModule,
  createMustyFlickModule,
  createPowerslideModule,
  createSpeedFlipModule,
  createTouchModule,
} from "./playerModules.ts";
import {
  createAbsolutePositioningModule,
  createRelativePositioningModule,
} from "./positioningModules.ts";
export {
  hasBoostPickupAnimationTimelineMatch,
} from "../boostPickupFilters.ts";
export {
  getCurrentRole,
  getStatsPlayerSnapshot,
  getTeamClass,
  RELATIVE_POSITIONING_MODULE_ID,
  ROLE_LABELS,
} from "./types.ts";
export type {
  Role,
  StatModule,
  StatModuleContext,
  StatModuleRuntime,
} from "./types.ts";

export function createStatModules(runtime: import("./types.ts").StatModuleRuntime) {
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
    createTouchModule(runtime),
    createMustyFlickModule(),
    createDodgeResetModule(),
    createBoostModule(runtime),
    createBallCarryModule(),
    createMovementModule(runtime),
    createPowerslideModule(),
    createDemoModule(),
  ];
}
