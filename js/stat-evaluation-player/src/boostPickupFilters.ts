import type {
  BoostPickupAnimationPickup,
  ReplayModel,
} from "subtr-actor-player";
import type { BoostPickupActivity } from "./generated/BoostPickupActivity.ts";
import type { BoostPickupComparison } from "./generated/BoostPickupComparison.ts";
import type { BoostPickupFieldHalf } from "./generated/BoostPickupFieldHalf.ts";
import type { BoostPickupPadType } from "./generated/BoostPickupPadType.ts";
import type { BoostPickupTimelineRangeOptions } from "./timelineRanges.ts";
import type { BoostPickupComparisonEvent, StatsTimeline } from "./statsTimeline.ts";
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
}

interface BoostPickupFilterRenderOptions {
  eyebrow: string;
  title: string;
}

export interface BoostPickupFilterController {
  setup(ctx: BoostPickupFilterContext): void;
  teardown(): void;
  getTimelineRangeOptions(): BoostPickupTimelineRangeOptions;
  includePickup(pickup: BoostPickupAnimationPickup): boolean;
  renderSettings(
    ctx: BoostPickupFilterContext | null,
    options: BoostPickupFilterRenderOptions,
  ): HTMLElement;
}

const BOOST_PICKUP_PAD_TYPE_OPTIONS = [
  { value: "big", label: "Big pads" },
  { value: "small", label: "Small pads" },
  { value: "ambiguous", label: "Ambiguous pads" },
] satisfies Array<BoostPickupFilterOption<BoostPickupPadType>>;

const BOOST_PICKUP_COMPARISON_OPTIONS = [
  { value: "both", label: "Matched pickups" },
  { value: "ghost", label: "Ghost pickups" },
  { value: "missed", label: "Missed pickups" },
] satisfies Array<BoostPickupFilterOption<BoostPickupComparison>>;

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
): BoostPickupComparisonEvent | null {
  const comparisonEvents = timeline?.events.boost_pickups ?? [];
  if (comparisonEvents.length === 0) {
    return null;
  }

  return comparisonEvents.find((event) => {
    const playerId = playerIdToString(event.player_id);
    const reportedFrame = event.reported_frame ?? event.frame;
    return (
      playerId === pickup.player.id &&
      event.comparison !== "missed" &&
      reportedFrame === pickup.event.frame &&
      isBoostPickupPadTypeCompatible(event.pad_type, pickup.pad.size)
    );
  }) ?? null;
}

export function hasBoostPickupAnimationTimelineMatch(
  pickup: BoostPickupAnimationPickup,
  timeline: StatsTimeline | null,
): boolean {
  const comparisonEvents = timeline?.events.boost_pickups ?? [];
  if (comparisonEvents.length === 0) {
    return true;
  }

  return getBoostPickupAnimationTimelineMatch(pickup, timeline) !== null;
}

export function createBoostPickupFilterController(
  runtime: BoostPickupFilterRuntime = {},
): BoostPickupFilterController {
  let settingsEl: HTMLDivElement | null = null;
  let pickupReadoutEl: HTMLElement | null = null;
  let playerOptionsEl: HTMLDivElement | null = null;
  let lastReplay: ReplayModel | null = null;
  let lastStatsTimeline: StatsTimeline | null = null;
  const activePadTypes = new Set<BoostPickupPadType>(
    BOOST_PICKUP_PAD_TYPE_OPTIONS.map((option) => option.value),
  );
  const activeComparisons = new Set<BoostPickupComparison>(
    BOOST_PICKUP_COMPARISON_OPTIONS.map((option) => option.value),
  );
  const activeActivities = new Set<BoostPickupActivity>(
    BOOST_PICKUP_ACTIVITY_OPTIONS.map((option) => option.value),
  );
  const activeFieldHalves = new Set<BoostPickupFieldHalf>(
    BOOST_PICKUP_FIELD_HALF_OPTIONS.map((option) => option.value),
  );
  let activePlayerIds: Set<string> | null = null;

  function createFilterGroup<T extends string>(
    title: string,
    optionSpecs: Array<BoostPickupFilterOption<T>>,
    activeValues: Set<T>,
    filterKey: string,
  ): HTMLDivElement {
    const group = document.createElement("div");
    group.className = "module-settings-subgroup";

    const groupTitle = document.createElement("p");
    groupTitle.className = "module-settings-group-title";
    groupTitle.textContent = title;

    const groupOptions = document.createElement("div");
    groupOptions.className = "module-settings-options";

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
    group.className = "module-settings-subgroup";

    const groupTitle = document.createElement("p");
    groupTitle.className = "module-settings-group-title";
    groupTitle.textContent = "Player";

    playerOptionsEl = document.createElement("div");
    playerOptionsEl.className = "module-settings-options";

    group.append(groupTitle, playerOptionsEl);
    return group;
  }

  function rebuildPlayerOptions(replay: ReplayModel | null): void {
    if (!playerOptionsEl) {
      return;
    }

    playerOptionsEl.replaceChildren();
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
      checkbox.checked = playerId
        ? activePlayerIds?.has(playerId) ?? true
        : false;
    }

    if (pickupReadoutEl) {
      pickupReadoutEl.textContent = getPickupReadout(replay);
    }
  }

  function isFilterValueActive(
    filterKey: string | undefined,
    value: string | undefined,
  ): boolean {
    if (!value) {
      return false;
    }

    switch (filterKey) {
      case "pad-type":
        return activePadTypes.has(value as BoostPickupPadType);
      case "comparison":
        return activeComparisons.has(value as BoostPickupComparison);
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
    const hidden = activePadTypes.size === 0 ||
      activeComparisons.size === 0 ||
      activeActivities.size === 0 ||
      activeFieldHalves.size === 0 ||
      (activePlayerIds !== null && activePlayerIds.size === 0);
    if (hidden) {
      return "Hidden";
    }

    const constrainedGroups = [
      activePadTypes.size < BOOST_PICKUP_PAD_TYPE_OPTIONS.length,
      activeComparisons.size < BOOST_PICKUP_COMPARISON_OPTIONS.length,
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

    if ((lastStatsTimeline?.events.boost_pickups ?? []).length === 0) {
      return activePadTypes.has(pickup.pad.size) &&
        activeComparisons.has("both") &&
        activeActivities.has("unknown") &&
        activeFieldHalves.has("unknown");
    }

    const matchedEvent = getBoostPickupAnimationTimelineMatch(
      pickup,
      lastStatsTimeline,
    );
    if (!matchedEvent) {
      return false;
    }

    return activePadTypes.has(matchedEvent.pad_type) &&
      activeComparisons.has(matchedEvent.comparison) &&
      activeActivities.has(matchedEvent.activity) &&
      activeFieldHalves.has(matchedEvent.field_half);
  }

  return {
    setup(ctx) {
      if (lastReplay !== ctx.replay) {
        lastReplay = ctx.replay;
        activePlayerIds = null;
      }
      lastStatsTimeline = ctx.statsTimeline;
      syncSettingsUi(ctx.replay);
    },

    teardown() {},

    getTimelineRangeOptions() {
      const options: BoostPickupTimelineRangeOptions = {
        padTypes: activePadTypes,
        comparisons: activeComparisons,
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
        settingsEl.className = "module-settings-card";

        const header = document.createElement("div");
        header.className = "module-settings-header";

        const text = document.createElement("div");
        const eyebrow = document.createElement("p");
        eyebrow.className = "module-settings-eyebrow";
        eyebrow.textContent = options.eyebrow;
        const title = document.createElement("h3");
        title.textContent = options.title;
        text.append(eyebrow, title);

        pickupReadoutEl = document.createElement("strong");
        pickupReadoutEl.className = "metric-readout";
        header.append(text, pickupReadoutEl);

        settingsEl.append(
          header,
          createFilterGroup(
            "Pad type",
            BOOST_PICKUP_PAD_TYPE_OPTIONS,
            activePadTypes,
            "pad-type",
          ),
          createFilterGroup(
            "Pickup label",
            BOOST_PICKUP_COMPARISON_OPTIONS,
            activeComparisons,
            "comparison",
          ),
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
      }

      rebuildPlayerOptions(ctx?.replay ?? null);
      syncSettingsUi(ctx?.replay ?? null);
      return settingsEl;
    },
  };
}
