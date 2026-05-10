export {
  createStatModules,
  DEPTH_ROLE_LABELS,
  getCurrentDepthRole,
  getCurrentRole,
  getStatsPlayerSnapshot,
  getTeamClass,
  hasBoostPickupAnimationTimelineMatch,
  RELATIVE_POSITIONING_MODULE_ID,
  ROLE_LABELS,
} from "./stat-modules/index.ts";

export type {
  DepthRole,
  Role,
  StatModule,
  StatModuleContext,
  StatModuleRuntime,
} from "./stat-modules/index.ts";
