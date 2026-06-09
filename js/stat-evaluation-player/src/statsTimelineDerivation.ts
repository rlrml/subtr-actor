import {
  applyBackboardEventDerivedStats,
  createBackboardEventDerivedStatsAccumulator,
} from "./backboardEventDerivation.ts";
import {
  applyBallCarryEventDerivedStats,
  createBallCarryEventDerivedStatsAccumulator,
} from "./ballCarryEventDerivation.ts";
import {
  applyBumpEventDerivedStats,
  createBumpEventDerivedStatsAccumulator,
} from "./bumpEventDerivation.ts";
import {
  applyBoostTrackDerivedStats,
  createBoostTrackDerivedStatsAccumulator,
} from "./boostTrackDerivation.ts";
import {
  applyCeilingShotEventDerivedStats,
  createCeilingShotEventDerivedStatsAccumulator,
} from "./ceilingShotEventDerivation.ts";
import {
  applyCoreEventDerivedStats,
  createCoreEventDerivedStatsAccumulator,
} from "./coreEventDerivation.ts";
import {
  applyControlledPlayEventDerivedStats,
  createControlledPlayEventDerivedStatsAccumulator,
} from "./controlledPlayEventDerivation.ts";
import {
  applyEventCountDerivedStats,
  createEventCountDerivedStatsAccumulator,
} from "./eventCountDerivation.ts";
import {
  applyDodgeResetEventDerivedStats,
  createDodgeResetEventDerivedStatsAccumulator,
} from "./dodgeResetEventDerivation.ts";
import {
  applyDoubleTapEventDerivedStats,
  createDoubleTapEventDerivedStatsAccumulator,
} from "./doubleTapEventDerivation.ts";
import {
  applyDemoEventDerivedStats,
  createDemoEventDerivedStatsAccumulator,
} from "./demoEventDerivation.ts";
import {
  applyFiftyFiftyEventDerivedStats,
  createFiftyFiftyEventDerivedStatsAccumulator,
} from "./fiftyFiftyEventDerivation.ts";
import {
  applyKickoffEventDerivedStats,
  createKickoffEventDerivedStatsAccumulator,
} from "./kickoffEventDerivation.ts";
import {
  applyFlickEventDerivedStats,
  createFlickEventDerivedStatsAccumulator,
} from "./flickEventDerivation.ts";
import {
  applyHalfVolleyEventDerivedStats,
  createHalfVolleyEventDerivedStatsAccumulator,
} from "./halfVolleyEventDerivation.ts";
import {
  applyMechanicEventDerivedStats,
  createMechanicEventDerivedStatsAccumulator,
} from "./mechanicEventDerivation.ts";
import {
  applyMovementEventDerivedStats,
  createMovementEventDerivedStatsAccumulator,
} from "./movementEventDerivation.ts";
import {
  applyMustyFlickEventDerivedStats,
  createMustyFlickEventDerivedStatsAccumulator,
} from "./mustyFlickEventDerivation.ts";
import {
  applyOneTimerEventDerivedStats,
  createOneTimerEventDerivedStatsAccumulator,
} from "./oneTimerEventDerivation.ts";
import {
  applyPassEventDerivedStats,
  createPassEventDerivedStatsAccumulator,
} from "./passEventDerivation.ts";
import {
  applyPossessionEventDerivedStats,
  createPossessionEventDerivedStatsAccumulator,
} from "./possessionEventDerivation.ts";
import {
  applyPositioningEventDerivedStats,
  createPositioningEventDerivedStatsAccumulator,
} from "./positioningEventDerivation.ts";
import {
  applyPowerslideEventDerivedStats,
  createPowerslideEventDerivedStatsAccumulator,
} from "./powerslideEventDerivation.ts";
import {
  applyBallHalfEventDerivedStats,
  createBallHalfEventDerivedStatsAccumulator,
} from "./ballHalfEventDerivation.ts";
import {
  applyTerritorialPressureEventDerivedStats,
  createTerritorialPressureEventDerivedStatsAccumulator,
} from "./territorialPressureEventDerivation.ts";
import {
  applyRotationEventDerivedStats,
  createRotationEventDerivedStatsAccumulator,
} from "./rotationEventDerivation.ts";
import {
  applyRushEventDerivedStats,
  createRushEventDerivedStatsAccumulator,
} from "./rushEventDerivation.ts";
import {
  applyTouchEventDerivedStats,
  createTouchEventDerivedStatsAccumulator,
} from "./touchEventDerivation.ts";
import {
  applyWhiffEventDerivedStats,
  createWhiffEventDerivedStatsAccumulator,
} from "./whiffEventDerivation.ts";
import {
  applyWallAerialEventDerivedStats,
  createWallAerialEventDerivedStatsAccumulator,
} from "./wallAerialEventDerivation.ts";
import {
  applyWallAerialShotEventDerivedStats,
  createWallAerialShotEventDerivedStatsAccumulator,
} from "./wallAerialShotEventDerivation.ts";
import type { ReplayLoadProgress } from "./replayLoadProgress.ts";
import type {
  StatsFrame,
  StatsFrameLookup,
  StatsFrameScaffold,
  MaterializedStatsTimeline,
  StatsTimeline,
} from "./statsTimeline";
import {
  createPlayerStatsSnapshot,
  createTeamStatsSnapshot,
  type DeepPartial,
} from "./statsSnapshotFactories.ts";

interface StatsTimelineEventDerivedApplier {
  id: string;
  playerModules: readonly string[];
  teamModules: readonly string[];
  apply: (timeline: MaterializedStatsTimeline) => MaterializedStatsTimeline;
  createFrameAccumulator?: (timeline: MaterializedStatsTimeline) => StatsFrameAccumulator;
}

interface StatsFrameAccumulator {
  applyFrame(frame: StatsFrame): void;
}

const DEFAULT_STATS_FRAME_MATERIALIZATION_CHUNK_SIZE = 300;
const DEFAULT_STATS_FRAME_MAX_MATERIALIZATION_CHUNK_SIZE = 1_200;
const STATS_FRAME_MATERIALIZATION_CHUNK_GROWTH_FACTOR = 2;

export const STATS_TIMELINE_EVENT_DERIVED_APPLIERS: readonly StatsTimelineEventDerivedApplier[] = [
  {
    id: "event-counts",
    playerModules: ["event_counts"],
    teamModules: ["event_counts"],
    apply: applyEventCountDerivedStats,
    createFrameAccumulator: createEventCountDerivedStatsAccumulator,
  },
  {
    id: "boost-track",
    playerModules: ["boost"],
    teamModules: ["boost"],
    apply: applyBoostTrackDerivedStats,
    createFrameAccumulator: createBoostTrackDerivedStatsAccumulator,
  },
  {
    id: "core",
    playerModules: ["core"],
    teamModules: ["core"],
    apply: applyCoreEventDerivedStats,
    createFrameAccumulator: createCoreEventDerivedStatsAccumulator,
  },
  {
    id: "possession",
    playerModules: [],
    teamModules: ["possession"],
    apply: applyPossessionEventDerivedStats,
    createFrameAccumulator: createPossessionEventDerivedStatsAccumulator,
  },
  {
    id: "ball_half",
    playerModules: [],
    teamModules: ["ball_half"],
    apply: applyBallHalfEventDerivedStats,
    createFrameAccumulator: createBallHalfEventDerivedStatsAccumulator,
  },
  {
    id: "territorial-pressure",
    playerModules: [],
    teamModules: ["territorial_pressure"],
    apply: applyTerritorialPressureEventDerivedStats,
    createFrameAccumulator: createTerritorialPressureEventDerivedStatsAccumulator,
  },
  {
    id: "movement",
    playerModules: ["movement"],
    teamModules: ["movement"],
    apply: applyMovementEventDerivedStats,
    createFrameAccumulator: createMovementEventDerivedStatsAccumulator,
  },
  {
    id: "positioning",
    playerModules: ["positioning"],
    teamModules: ["positioning"],
    apply: applyPositioningEventDerivedStats,
    createFrameAccumulator: createPositioningEventDerivedStatsAccumulator,
  },
  {
    id: "rotation",
    playerModules: ["rotation"],
    teamModules: ["rotation"],
    apply: applyRotationEventDerivedStats,
    createFrameAccumulator: createRotationEventDerivedStatsAccumulator,
  },
  {
    id: "mechanics",
    playerModules: ["speed_flip", "half_flip", "wavedash"],
    teamModules: [],
    apply: applyMechanicEventDerivedStats,
    createFrameAccumulator: createMechanicEventDerivedStatsAccumulator,
  },
  {
    id: "whiff",
    playerModules: ["whiff"],
    teamModules: [],
    apply: applyWhiffEventDerivedStats,
    createFrameAccumulator: createWhiffEventDerivedStatsAccumulator,
  },
  {
    id: "backboard",
    playerModules: ["backboard"],
    teamModules: ["backboard"],
    apply: applyBackboardEventDerivedStats,
    createFrameAccumulator: createBackboardEventDerivedStatsAccumulator,
  },
  {
    id: "double-tap",
    playerModules: ["double_tap"],
    teamModules: ["double_tap"],
    apply: applyDoubleTapEventDerivedStats,
    createFrameAccumulator: createDoubleTapEventDerivedStatsAccumulator,
  },
  {
    id: "demo",
    playerModules: ["demo"],
    teamModules: ["demo"],
    apply: applyDemoEventDerivedStats,
    createFrameAccumulator: createDemoEventDerivedStatsAccumulator,
  },
  {
    id: "fifty-fifty",
    playerModules: ["fifty_fifty"],
    teamModules: ["fifty_fifty"],
    apply: applyFiftyFiftyEventDerivedStats,
    createFrameAccumulator: createFiftyFiftyEventDerivedStatsAccumulator,
  },
  {
    id: "kickoff",
    playerModules: ["kickoff"],
    teamModules: ["kickoff"],
    apply: applyKickoffEventDerivedStats,
    createFrameAccumulator: createKickoffEventDerivedStatsAccumulator,
  },
  {
    id: "bump",
    playerModules: ["bump"],
    teamModules: ["bump"],
    apply: applyBumpEventDerivedStats,
    createFrameAccumulator: createBumpEventDerivedStatsAccumulator,
  },
  {
    id: "rush",
    playerModules: [],
    teamModules: ["rush"],
    apply: applyRushEventDerivedStats,
    createFrameAccumulator: createRushEventDerivedStatsAccumulator,
  },
  {
    id: "pass",
    playerModules: ["pass"],
    teamModules: ["pass"],
    apply: applyPassEventDerivedStats,
    createFrameAccumulator: createPassEventDerivedStatsAccumulator,
  },
  {
    id: "one-timer",
    playerModules: ["one_timer"],
    teamModules: ["one_timer"],
    apply: applyOneTimerEventDerivedStats,
    createFrameAccumulator: createOneTimerEventDerivedStatsAccumulator,
  },
  {
    id: "ball-carry",
    playerModules: ["ball_carry", "air_dribble"],
    teamModules: ["ball_carry", "air_dribble"],
    apply: applyBallCarryEventDerivedStats,
    createFrameAccumulator: createBallCarryEventDerivedStatsAccumulator,
  },
  {
    id: "controlled-play",
    playerModules: ["controlled_play"],
    teamModules: ["controlled_play"],
    apply: applyControlledPlayEventDerivedStats,
    createFrameAccumulator: createControlledPlayEventDerivedStatsAccumulator,
  },
  {
    id: "wall-aerial",
    playerModules: ["wall_aerial"],
    teamModules: [],
    apply: applyWallAerialEventDerivedStats,
    createFrameAccumulator: createWallAerialEventDerivedStatsAccumulator,
  },
  {
    id: "wall-aerial-shot",
    playerModules: ["wall_aerial_shot"],
    teamModules: [],
    apply: applyWallAerialShotEventDerivedStats,
    createFrameAccumulator: createWallAerialShotEventDerivedStatsAccumulator,
  },
  {
    id: "flick",
    playerModules: ["flick"],
    teamModules: [],
    apply: applyFlickEventDerivedStats,
    createFrameAccumulator: createFlickEventDerivedStatsAccumulator,
  },
  {
    id: "ceiling-shot",
    playerModules: ["ceiling_shot"],
    teamModules: [],
    apply: applyCeilingShotEventDerivedStats,
    createFrameAccumulator: createCeilingShotEventDerivedStatsAccumulator,
  },
  {
    id: "musty-flick",
    playerModules: ["musty_flick"],
    teamModules: [],
    apply: applyMustyFlickEventDerivedStats,
    createFrameAccumulator: createMustyFlickEventDerivedStatsAccumulator,
  },
  {
    id: "dodge-reset",
    playerModules: ["dodge_reset"],
    teamModules: [],
    apply: applyDodgeResetEventDerivedStats,
    createFrameAccumulator: createDodgeResetEventDerivedStatsAccumulator,
  },
  {
    id: "powerslide",
    playerModules: ["powerslide"],
    teamModules: ["powerslide"],
    apply: applyPowerslideEventDerivedStats,
    createFrameAccumulator: createPowerslideEventDerivedStatsAccumulator,
  },
  {
    id: "touch",
    playerModules: ["touch"],
    teamModules: [],
    apply: applyTouchEventDerivedStats,
    createFrameAccumulator: createTouchEventDerivedStatsAccumulator,
  },
  {
    id: "half-volley",
    playerModules: ["half_volley"],
    teamModules: ["half_volley"],
    apply: applyHalfVolleyEventDerivedStats,
    createFrameAccumulator: createHalfVolleyEventDerivedStatsAccumulator,
  },
];

export function applyStatsTimelineEventDerivedStats(
  timeline: StatsTimeline,
): MaterializedStatsTimeline {
  let materializedTimeline = hydrateStatsTimelineFrameScaffolding(timeline);
  for (const applier of STATS_TIMELINE_EVENT_DERIVED_APPLIERS) {
    materializedTimeline = applier.apply(materializedTimeline);
  }
  return materializedTimeline;
}

export function createEventDerivedStatsFrameLookup(
  timeline: StatsTimeline,
  onProgress?: (progress: ReplayLoadProgress) => void,
  options: { materializationChunkSize?: number; maxMaterializationChunkSize?: number } = {},
): StatsFrameLookup {
  const scaffoldFrames = timeline.frames;
  const frameIndexByNumber = new Map(
    scaffoldFrames.map((frame, index) => [frame.frame_number, index] as const),
  );
  const materializedFrames = new Map<number, StatsFrame>();
  const accumulatorTimeline = { ...timeline, frames: [] } as MaterializedStatsTimeline;
  const incrementalAccumulators = STATS_TIMELINE_EVENT_DERIVED_APPLIERS.flatMap((applier) =>
    applier.createFrameAccumulator ? [applier.createFrameAccumulator(accumulatorTimeline)] : [],
  );
  const materializationChunkSize = Math.max(
    1,
    options.materializationChunkSize ?? DEFAULT_STATS_FRAME_MATERIALIZATION_CHUNK_SIZE,
  );
  const maxMaterializationChunkSize = Math.max(
    materializationChunkSize,
    options.maxMaterializationChunkSize ?? DEFAULT_STATS_FRAME_MAX_MATERIALIZATION_CHUNK_SIZE,
  );
  let materializedUntilIndex = -1;
  let nextMaterializationChunkSize = materializationChunkSize;

  const materializeUntilIndex = (frameIndex: number) => {
    if (frameIndex <= materializedUntilIndex) {
      return;
    }
    const targetIndex = Math.min(
      scaffoldFrames.length - 1,
      Math.max(frameIndex, materializedUntilIndex + nextMaterializationChunkSize),
    );
    for (let index = materializedUntilIndex + 1; index <= targetIndex; index += 1) {
      const scaffoldFrame = scaffoldFrames[index];
      const frame = scaffoldFrame
        ? hydrateStatsFrameScaffolding(cloneStatsFrameScaffold(scaffoldFrame))
        : undefined;
      if (frame) {
        for (const accumulator of incrementalAccumulators) {
          accumulator.applyFrame(frame);
        }
        materializedFrames.set(frame.frame_number, frame);
      }
    }
    materializedUntilIndex = targetIndex;
    onProgress?.({
      stage: "deriving-stats",
      processedFrames: materializedUntilIndex + 1,
      totalFrames: scaffoldFrames.length,
      progress:
        scaffoldFrames.length === 0 ? 1 : (materializedUntilIndex + 1) / scaffoldFrames.length,
    });
    nextMaterializationChunkSize = Math.min(
      maxMaterializationChunkSize,
      scaffoldFrames.length,
      nextMaterializationChunkSize * STATS_FRAME_MATERIALIZATION_CHUNK_GROWTH_FACTOR,
    );
  };

  return {
    get(frameNumber: number) {
      const frameIndex = frameIndexByNumber.get(frameNumber);
      if (frameIndex === undefined) {
        return undefined;
      }
      materializeUntilIndex(frameIndex);
      return materializedFrames.get(frameNumber);
    },
  };
}

function clonePlayerId(
  playerId: (StatsFrame | StatsFrameScaffold)["players"][number]["player_id"],
) {
  if (!playerId || typeof playerId !== "object") {
    return playerId;
  }
  return { ...(playerId as Record<string, unknown>) } as typeof playerId;
}

function cloneStatsFrameScaffold(
  frame: StatsFrame | StatsFrameScaffold,
): StatsFrame | StatsFrameScaffold {
  return {
    ...frame,
    team_zero: { ...(frame.team_zero as Record<string, unknown>) } as typeof frame.team_zero,
    team_one: { ...(frame.team_one as Record<string, unknown>) } as typeof frame.team_one,
    players: frame.players.map((player) => ({
      ...player,
      player_id: clonePlayerId(player.player_id),
    })),
  };
}

function hydrateStatsTimelineFrameScaffolding(timeline: StatsTimeline): MaterializedStatsTimeline {
  return {
    ...timeline,
    frames: timeline.frames.map((frame) => hydrateStatsFrameScaffolding(frame)),
  };
}

function hydrateStatsFrameScaffolding(frame: StatsFrame | StatsFrameScaffold): StatsFrame {
  const hydratedFrame = {
    ...frame,
    team_zero: createTeamStatsSnapshot(
      (frame.team_zero ?? {}) as DeepPartial<typeof frame.team_zero>,
    ),
    team_one: createTeamStatsSnapshot((frame.team_one ?? {}) as DeepPartial<typeof frame.team_one>),
    players: frame.players.map((player) =>
      createPlayerStatsSnapshot(player as DeepPartial<typeof player>),
    ),
  };
  return hydratedFrame as StatsFrame;
}
