import type { BoostPickupAnimationPickup, ReplayModel } from "@rlrml/player";
import type { BoostPickupActivity } from "./generated/BoostPickupActivity.ts";
import type { BoostPickupDetection } from "./generated/BoostPickupDetection.ts";
import type { BoostPickupFieldHalf } from "./generated/BoostPickupFieldHalf.ts";
import type { BoostPickupPadType } from "./generated/BoostPickupPadType.ts";
import type { BoostPickupTimelineRangeOptions } from "./timelineRanges.ts";
import type { BoostPickupEvent, StatsTimeline } from "./statsTimeline.ts";
import { statsEventPayloads } from "./statsTimeline.ts";
import { playerIdToString } from "./touchOverlay.ts";

type BoostPickupFilterOption<T extends string> = {
  value: T;
  label: string;
};

interface BoostPickupFilterContext {
  replay: ReplayModel;
  statsTimeline: StatsTimeline;
}

interface BoostPickupFilterRuntime {
  refreshTimelineRanges?(): void;
  rerenderCurrentState?(): void;
  requestConfigSync?(): void;
}

interface BoostPickupFilterRenderOptions {
  showHeader?: boolean;
}

export interface BoostPickupFilterController {
  setup(ctx: BoostPickupFilterContext): void;
  teardown(): void;
  getConfig(): BoostPickupFilterConfig;
  applyConfig(config: unknown): void;
  getTimelineRangeOptions(): BoostPickupTimelineRangeOptions;
  includePickup(pickup: BoostPickupAnimationPickup): boolean;
  renderSettings(
    ctx: BoostPickupFilterContext | null,
    options: BoostPickupFilterRenderOptions,
  ): HTMLElement;
}

export interface BoostPickupFilterConfig {
  readonly padTypes: BoostPickupPadType[];
  readonly detections: BoostPickupDetection[];
  readonly activities: BoostPickupActivity[];
  readonly fieldHalves: BoostPickupFieldHalf[];
  readonly playerIds: string[] | null;
}

const BOOST_PICKUP_PAD_TYPE_OPTIONS = [
  { value: "big", label: "Big pads" },
  { value: "small", label: "Small pads" },
  { value: "ambiguous", label: "Ambiguous pads" },
] satisfies Array<BoostPickupFilterOption<BoostPickupPadType>>;

const BOOST_PICKUP_DETECTION_OPTIONS = [
  { value: "both", label: "Both detectors" },
  { value: "inferred_only", label: "Inferred only" },
  { value: "reported_only", label: "Reported only" },
] satisfies Array<BoostPickupFilterOption<BoostPickupDetection>>;

const BOOST_PICKUP_ACTIVITY_OPTIONS = [
  { value: "active", label: "Active play" },
  { value: "inactive", label: "Inactive play" },
  { value: "unknown", label: "Unknown activity" },
] satisfies Array<BoostPickupFilterOption<BoostPickupActivity>>;

const BOOST_PICKUP_FIELD_HALF_OPTIONS = [
  { value: "own", label: "Own half" },
  { value: "opponent", label: "Opponent half" },
  { value: "unknown", label: "Unknown half" },
] satisfies Array<BoostPickupFilterOption<BoostPickupFieldHalf>>;

function isBoostPickupPadTypeCompatible(
  padType: BoostPickupPadType,
  padSize: "big" | "small",
): boolean {
  return padType === padSize || padType === "ambiguous";
}

export function getBoostPickupAnimationTimelineMatch(
  pickup: BoostPickupAnimationPickup,
  timeline: StatsTimeline | null,
): BoostPickupEvent | null {
  const pickupEvents = timeline
    ? boostPickupEvents(timeline, statsEventPayloads(timeline, "boost_pickup"))
    : [];
  if (pickupEvents.length === 0) {
    return null;
  }

  return (
    pickupEvents.find((event) => {
      const playerId = playerIdToString(event.player_id);
      return (
        playerId === pickup.player.id &&
        // A reported pad pickup corresponds to any pickup the pad-event detector saw.
        event.detection !== "inferred_only" &&
        event.frame === pickup.event.frame &&
        isBoostPickupPadTypeCompatible(event.pad_type, pickup.pad.size)
      );
    }) ?? null
  );
}

export function hasBoostPickupAnimationTimelineMatch(
  pickup: BoostPickupAnimationPickup,
  timeline: StatsTimeline | null,
): boolean {
  const pickupEvents = timeline
    ? boostPickupEvents(timeline, statsEventPayloads(timeline, "boost_pickup"))
    : [];
  if (pickupEvents.length === 0) {
    return true;
  }

  return getBoostPickupAnimationTimelineMatch(pickup, timeline) !== null;
}

function boostPickupEvents(
  timeline: StatsTimeline,
  pickupEvents: BoostPickupEvent[],
): BoostPickupEvent[] {
  if (pickupEvents.length > 0) {
    return pickupEvents;
  }
  const legacyEvents = (timeline as unknown as { events?: { boost_pickups?: unknown } }).events
    ?.boost_pickups;
  return Array.isArray(legacyEvents) ? (legacyEvents as BoostPickupEvent[]) : pickupEvents;
}

export function createBoostPickupFilterController(
  runtime: BoostPickupFilterRuntime = {},
): BoostPickupFilterController {
  let settingsEl: HTMLDivElement | null = null;
  let pickupReadoutEl: HTMLElement | null = null;
  let playerGroupEl: HTMLDivElement | null = null;
  let playerOptionsEl: HTMLDivElement | null = null;
  let lastReplay: ReplayModel | null = null;
  let lastStatsTimeline: StatsTimeline | null = null;
  const activePadTypes = new Set<BoostPickupPadType>(
    BOOST_PICKUP_PAD_TYPE_OPTIONS.map((option) => option.value),
  );
  const activeDetections = new Set<BoostPickupDetection>(
    BOOST_PICKUP_DETECTION_OPTIONS.map((option) => option.value),
  );
  const activeActivities = new Set<BoostPickupActivity>(
    BOOST_PICKUP_ACTIVITY_OPTIONS.map((option) => option.value),
  );
  const activeFieldHalves = new Set<BoostPickupFieldHalf>(
    BOOST_PICKUP_FIELD_HALF_OPTIONS.map((option) => option.value),
  );
  let activePlayerIds: Set<string> | null = null;
  let preserveConfiguredPlayerIdsForNextReplay = false;

  function createFilterGroup<T extends string>(
    title: string,
    optionSpecs: Array<BoostPickupFilterOption<T>>,
    activeValues: Set<T>,
    filterKey: string,
  ): HTMLDivElement {
    const group = document.createElement("div");
    group.className = "boost-pickup-filter-group";

    const groupTitle = document.createElement("p");
    groupTitle.className = "module-settings-group-title";
    groupTitle.textContent = title;

    const groupOptions = document.createElement("div");
    groupOptions.className = "boost-pickup-filter-options";

    for (const option of optionSpecs) {
      const optionLabel = document.createElement("label");
      optionLabel.className = "toggle";

      const checkbox = document.createElement("input");
      checkbox.type = "checkbox";
      checkbox.dataset.boostPickupFilter = filterKey;
      checkbox.dataset.boostPickupValue = option.value;
      checkbox.addEventListener("change", () => {
        if (checkbox.checked) {
          activeValues.add(option.value);
        } else {
          activeValues.delete(option.value);
        }
        syncSettingsUi(lastReplay);
        runtime.refreshTimelineRanges?.();
        runtime.rerenderCurrentState?.();
        runtime.requestConfigSync?.();
      });

      const optionText = document.createElement("span");
      optionText.textContent = option.label;
      optionLabel.append(checkbox, optionText);
      groupOptions.append(optionLabel);
    }

    group.append(groupTitle, groupOptions);
    return group;
  }

  function createPlayerFilterGroup(): HTMLDivElement {
    const group = document.createElement("div");
    group.className = "boost-pickup-filter-group boost-pickup-filter-group-wide";
    playerGroupEl = group;

    const groupTitle = document.createElement("p");
    groupTitle.className = "module-settings-group-title";
    groupTitle.textContent = "Player";

    playerOptionsEl = document.createElement("div");
    playerOptionsEl.className = "boost-pickup-filter-options";

    group.append(groupTitle, playerOptionsEl);
    return group;
  }

  function rebuildPlayerOptions(replay: ReplayModel | null): void {
    if (!playerOptionsEl) {
      return;
    }

    playerOptionsEl.replaceChildren();
    if (playerGroupEl) {
      playerGroupEl.hidden = !replay || replay.players.length === 0;
    }
    if (!replay) {
      return;
    }

    for (const player of replay.players) {
      const optionLabel = document.createElement("label");
      optionLabel.className = "toggle";

      const checkbox = document.createElement("input");
      checkbox.type = "checkbox";
      checkbox.dataset.boostPickupPlayerId = player.id;
      checkbox.addEventListener("change", () => {
        if (!activePlayerIds) {
          activePlayerIds = new Set(replay.players.map((candidate) => candidate.id));
        }
        if (checkbox.checked) {
          activePlayerIds.add(player.id);
        } else {
          activePlayerIds.delete(player.id);
        }
        syncSettingsUi(replay);
        runtime.refreshTimelineRanges?.();
        runtime.rerenderCurrentState?.();
        runtime.requestConfigSync?.();
      });

      const optionText = document.createElement("span");
      optionText.textContent = `${player.name} (${player.isTeamZero ? "Blue" : "Orange"})`;
      optionLabel.append(checkbox, optionText);
      playerOptionsEl.append(optionLabel);
    }
  }

  function syncSettingsUi(replay: ReplayModel | null): void {
    if (!settingsEl) {
      return;
    }

    for (const checkbox of settingsEl.querySelectorAll<HTMLInputElement>(
      "input[data-boost-pickup-filter][data-boost-pickup-value]",
    )) {
      const filterKey = checkbox.dataset.boostPickupFilter;
      const value = checkbox.dataset.boostPickupValue;
      checkbox.checked = isFilterValueActive(filterKey, value);
    }

    for (const checkbox of settingsEl.querySelectorAll<HTMLInputElement>(
      "input[data-boost-pickup-player-id]",
    )) {
      const playerId = checkbox.dataset.boostPickupPlayerId;
      checkbox.checked = playerId ? (activePlayerIds?.has(playerId) ?? true) : false;
    }

    if (pickupReadoutEl) {
      pickupReadoutEl.textContent = getPickupReadout(replay);
    }
  }

  function isFilterValueActive(filterKey: string | undefined, value: string | undefined): boolean {
    if (!value) {
      return false;
    }

    switch (filterKey) {
      case "pad-type":
        return activePadTypes.has(value as BoostPickupPadType);
      case "detection":
        return activeDetections.has(value as BoostPickupDetection);
      case "activity":
        return activeActivities.has(value as BoostPickupActivity);
      case "field-half":
        return activeFieldHalves.has(value as BoostPickupFieldHalf);
      default:
        return false;
    }
  }

  function getPickupReadout(replay: ReplayModel | null): string {
    const playerCount = replay?.players.length ?? 0;
    const visiblePlayerCount = activePlayerIds ? activePlayerIds.size : playerCount;
    const hidden =
      activePadTypes.size === 0 ||
      activeDetections.size === 0 ||
      activeActivities.size === 0 ||
      activeFieldHalves.size === 0 ||
      (activePlayerIds !== null && activePlayerIds.size === 0);
    if (hidden) {
      return "Hidden";
    }

    const constrainedGroups = [
      activePadTypes.size < BOOST_PICKUP_PAD_TYPE_OPTIONS.length,
      activeDetections.size < BOOST_PICKUP_DETECTION_OPTIONS.length,
      activeActivities.size < BOOST_PICKUP_ACTIVITY_OPTIONS.length,
      activeFieldHalves.size < BOOST_PICKUP_FIELD_HALF_OPTIONS.length,
      activePlayerIds !== null && visiblePlayerCount < playerCount,
    ].filter(Boolean).length;

    return constrainedGroups === 0 ? "All labels" : `${constrainedGroups} filters`;
  }

  function includePickup(pickup: BoostPickupAnimationPickup): boolean {
    if (activePlayerIds && !activePlayerIds.has(pickup.player.id)) {
      return false;
    }

    if (!lastStatsTimeline || statsEventPayloads(lastStatsTimeline, "boost_pickup").length === 0) {
      return (
        activePadTypes.has(pickup.pad.size) &&
        activeDetections.has("both") &&
        activeActivities.has("unknown") &&
        activeFieldHalves.has("unknown")
      );
    }

    const matchedEvent = getBoostPickupAnimationTimelineMatch(pickup, lastStatsTimeline);
    if (!matchedEvent) {
      return false;
    }

    return (
      activePadTypes.has(matchedEvent.pad_type) &&
      activeDetections.has(matchedEvent.detection) &&
      activeActivities.has(matchedEvent.activity) &&
      activeFieldHalves.has(matchedEvent.field_half)
    );
  }

  function setActiveValues<T extends string>(
    activeValues: Set<T>,
    options: Array<BoostPickupFilterOption<T>>,
    values: unknown,
  ): void {
    activeValues.clear();
    if (!Array.isArray(values)) {
      for (const option of options) {
        activeValues.add(option.value);
      }
      return;
    }

    const allowed = new Set(options.map((option) => option.value));
    for (const value of values) {
      if (typeof value === "string" && allowed.has(value as T)) {
        activeValues.add(value as T);
      }
    }
  }

  function getConfig(): BoostPickupFilterConfig {
    return {
      padTypes: [...activePadTypes],
      detections: [...activeDetections],
      activities: [...activeActivities],
      fieldHalves: [...activeFieldHalves],
      playerIds: activePlayerIds ? [...activePlayerIds] : null,
    };
  }

  function applyConfig(config: unknown): void {
    if (!config || typeof config !== "object" || Array.isArray(config)) {
      return;
    }
    const record = config as Record<string, unknown>;
    setActiveValues(activePadTypes, BOOST_PICKUP_PAD_TYPE_OPTIONS, record.padTypes);
    setActiveValues(activeDetections, BOOST_PICKUP_DETECTION_OPTIONS, record.detections);
    setActiveValues(activeActivities, BOOST_PICKUP_ACTIVITY_OPTIONS, record.activities);
    setActiveValues(activeFieldHalves, BOOST_PICKUP_FIELD_HALF_OPTIONS, record.fieldHalves);
    activePlayerIds = Array.isArray(record.playerIds)
      ? new Set(record.playerIds.filter((id): id is string => typeof id === "string"))
      : null;
    preserveConfiguredPlayerIdsForNextReplay = lastReplay === null && activePlayerIds !== null;
    syncSettingsUi(lastReplay);
    runtime.refreshTimelineRanges?.();
    runtime.rerenderCurrentState?.();
    runtime.requestConfigSync?.();
  }

  return {
    setup(ctx) {
      if (lastReplay !== ctx.replay) {
        lastReplay = ctx.replay;
        if (preserveConfiguredPlayerIdsForNextReplay) {
          preserveConfiguredPlayerIdsForNextReplay = false;
        } else {
          activePlayerIds = null;
        }
      }
      lastStatsTimeline = ctx.statsTimeline;
      syncSettingsUi(ctx.replay);
    },

    teardown() {},

    getConfig,

    applyConfig,

    getTimelineRangeOptions() {
      const options: BoostPickupTimelineRangeOptions = {
        padTypes: activePadTypes,
        detections: activeDetections,
        activities: activeActivities,
        fieldHalves: activeFieldHalves,
      };
      if (activePlayerIds) {
        options.playerIds = activePlayerIds;
      }
      return options;
    },

    includePickup,

    renderSettings(ctx, options) {
      if (!settingsEl) {
        settingsEl = document.createElement("div");
        settingsEl.className = "boost-pickup-filter-panel";

        const header = document.createElement("div");
        header.className = "boost-pickup-filter-summary";

        pickupReadoutEl = document.createElement("strong");
        pickupReadoutEl.className = "metric-readout";
        header.append(pickupReadoutEl);

        const grid = document.createElement("div");
        grid.className = "boost-pickup-filter-grid";
        grid.append(
          createFilterGroup("Pad type", BOOST_PICKUP_PAD_TYPE_OPTIONS, activePadTypes, "pad-type"),
          createFilterGroup(
            "Activity",
            BOOST_PICKUP_ACTIVITY_OPTIONS,
            activeActivities,
            "activity",
          ),
          createFilterGroup(
            "Field half",
            BOOST_PICKUP_FIELD_HALF_OPTIONS,
            activeFieldHalves,
            "field-half",
          ),
          createPlayerFilterGroup(),
        );

        if (options.showHeader ?? false) {
          settingsEl.append(header);
        }
        settingsEl.append(grid);
      }

      rebuildPlayerOptions(ctx?.replay ?? null);
      syncSettingsUi(ctx?.replay ?? null);
      return settingsEl;
    },
  };
}
