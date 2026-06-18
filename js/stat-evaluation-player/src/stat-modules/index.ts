import {
  createFiftyFiftyModule,
  createPossessionModule,
  createBallHalfModule,
  createBallThirdModule,
  createRushModule,
} from "./teamModules.ts";
import {
  createBackboardModule,
  createAirDribbleModule,
  createBallCarryModule,
  createBoostModule,
  createBumpModule,
  createCeilingShotModule,
  createCoreModule,
  createDemoModule,
  createDodgeModule,
  createDodgeResetModule,
  createDoubleTapModule,
  createFlickModule,
  createHalfFlipModule,
  createMovementModule,
  createMustyFlickModule,
  createOneTimerModule,
  createPassModule,
  createPowerslideModule,
  createRotationModule,
  createSpeedFlipModule,
  createTouchModule,
  createWavedashModule,
  createWallAerialModule,
  createWallAerialShotModule,
  createWhiffModule,
} from "./playerModules.ts";
import {
  createAbsolutePositioningModule,
  createRelativePositioningModule,
} from "./positioningModules.ts";
import type { BoostPickupFilterController } from "../boostPickupFilters.ts";

export { hasBoostPickupAnimationTimelineMatch } from "../boostPickupFilters.ts";
export {
  DEPTH_ROLE_LABELS,
  getCurrentDepthRole,
  getCurrentRole,
  getStatsPlayerSnapshot,
  getTeamClass,
  RELATIVE_POSITIONING_MODULE_ID,
  ROLE_LABELS,
} from "./types.ts";
export type { DepthRole, Role, StatModule, StatModuleContext, StatModuleRuntime } from "./types.ts";

export function createStatModules(
  runtime: import("./types.ts").StatModuleRuntime,
  options: { boostPickupFilters?: BoostPickupFilterController } = {},
) {
  return [
    createCoreModule(),
    createBackboardModule(),
    createCeilingShotModule(),
    createWallAerialModule(),
    createWallAerialShotModule(),
    createDoubleTapModule(),
    createOneTimerModule(),
    createPassModule(),
    createPossessionModule(runtime),
    createFiftyFiftyModule(),
    createBallHalfModule(),
    createBallThirdModule(),
    createRushModule(),
    createRelativePositioningModule(),
    createAbsolutePositioningModule(),
    createRotationModule(),
    createDodgeModule(),
    createSpeedFlipModule(),
    createHalfFlipModule(),
    createWavedashModule(),
    createTouchModule(runtime),
    createWhiffModule(),
    createFlickModule(),
    createMustyFlickModule(),
    createDodgeResetModule(),
    createAirDribbleModule(),
    createBoostModule(runtime, options.boostPickupFilters),
    createBallCarryModule(),
    createMovementModule(runtime),
    createPowerslideModule(),
    createDemoModule(),
    createBumpModule(),
  ];
}
