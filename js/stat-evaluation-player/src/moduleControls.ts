import type { BoostPickupFilterController } from "./boostPickupFilters.ts";
import type { EventTimelineSource } from "./eventTimelineSources.ts";
import {
  RELATIVE_POSITIONING_MODULE_ID,
  type StatModule,
  type StatModuleContext,
} from "./statModules.ts";

export type ModuleCapabilityKind = "events" | "ranges" | "effects";

const RENDER_EFFECT_MODULE_IDS = new Set([
  "ceiling-shot",
  "fifty-fifty",
  "ball_half",
  RELATIVE_POSITIONING_MODULE_ID,
  "absolute-positioning",
  "dodge",
  "speed-flip",
  "touch",
]);
const TOUCH_MODULE_ID = "touch";

export interface ModuleControlsElements {
  readonly summary: HTMLDivElement;
  readonly settings: HTMLDivElement;
  readonly boostPickupFilters: HTMLDivElement;
  readonly touchControls: HTMLDivElement;
}

export interface ModuleControlsOptions {
  readonly elements: ModuleControlsElements;
  readonly modules: readonly StatModule[];
  readonly boostPickupFilters: BoostPickupFilterController;
  getContext(): StatModuleContext | null;
  getTimelineSources(): readonly EventTimelineSource[];
  getActiveModules(): readonly StatModule[];
  getActiveCapabilityIds(kind: ModuleCapabilityKind): ReadonlySet<string>;
  getBoostPickupAnimationEnabled(): boolean;
  toggleCapability(id: string, kind: ModuleCapabilityKind, enabled: boolean): void;
  toggleBoostPickupAnimation(): void;
  syncTimelineEvents(): void;
  syncTimelineRanges(): void;
  renderTimelineEventCount(): void;
  requestConfigSync(): void;
}

export class ModuleControlsController {
  constructor(private readonly options: ModuleControlsOptions) {}

  renderSummary(): void {
    const { summary } = this.options.elements;
    summary.replaceChildren();

    const timelineSources = this.options.getTimelineSources();
    const markerToggles = timelineSources.map((source) => this.renderTimelineSourceToggle(source));
    const rangeToggles: HTMLButtonElement[] = [];
    const inGameVisualizationToggles: HTMLButtonElement[] = [];
    const ctx = this.options.getContext();

    for (const mod of this.options.modules) {
      const hasRenderEffect = RENDER_EFFECT_MODULE_IDS.has(mod.id);
      if (!mod.getTimelineEvents && !mod.getTimelineRanges && !hasRenderEffect) {
        continue;
      }

      if (timelineSources.length === 0 && mod.getTimelineEvents) {
        markerToggles.push(
          this.renderCapabilityToggle(mod.id, getCapabilityLabel(mod, "events"), "events"),
        );
      }
      if (mod.getTimelineRanges) {
        rangeToggles.push(
          this.renderCapabilityToggle(
            mod.id,
            getCapabilityLabel(mod, "ranges"),
            "ranges",
            ctx ? mod.getTimelineRanges(ctx).length : undefined,
          ),
        );
      }
      if (hasRenderEffect) {
        inGameVisualizationToggles.push(
          this.renderCapabilityToggle(mod.id, getCapabilityLabel(mod, "effects"), "effects"),
        );
      }
    }

    inGameVisualizationToggles.push(this.renderBoostPickupAnimationToggle());

    summary.append(
      renderModuleSummaryGroup("Timeline markers", markerToggles),
      renderModuleSummaryGroup("Timeline ranges", rangeToggles),
      renderModuleSummaryGroup("In-game visualizations", inGameVisualizationToggles),
    );
  }

  renderSettings(): void {
    const { settings } = this.options.elements;
    settings.replaceChildren();

    const ctx = this.options.getContext();
    const panels = this.options
      .getActiveModules()
      .filter((mod) => mod.id !== "boost" && mod.id !== TOUCH_MODULE_ID)
      .map((mod) => mod.renderSettings?.(ctx) ?? null)
      .filter((panel): panel is HTMLElement => panel instanceof HTMLElement);

    if (panels.length === 0) {
      settings.hidden = true;
      this.renderBoostPickupFiltersWindow();
      this.renderTouchControlsWindow();
      return;
    }

    settings.hidden = false;
    settings.append(...panels);
    this.renderBoostPickupFiltersWindow();
    this.renderTouchControlsWindow();
  }

  private renderBoostPickupAnimationToggle(): HTMLButtonElement {
    const active = this.options.getBoostPickupAnimationEnabled();
    const item = document.createElement("button");
    item.type = "button";
    item.className = "module-summary-item";
    item.dataset.active = active ? "true" : "false";
    item.setAttribute("aria-pressed", active ? "true" : "false");
    item.addEventListener("click", this.options.toggleBoostPickupAnimation);

    const name = document.createElement("span");
    name.textContent = "Boost pickup animation";

    const state = document.createElement("strong");
    state.textContent = active ? "On" : "Off";

    item.append(name, state);
    return item;
  }

  private renderCapabilityToggle(
    moduleId: string,
    label: string,
    kind: ModuleCapabilityKind,
    count?: number,
  ): HTMLButtonElement {
    const activeIds = this.options.getActiveCapabilityIds(kind);
    const active = activeIds.has(moduleId);
    const item = document.createElement("button");
    item.type = "button";
    item.className = "module-summary-item";
    item.dataset.active = active ? "true" : "false";
    item.setAttribute("aria-pressed", active ? "true" : "false");
    item.addEventListener("click", () => {
      this.options.toggleCapability(
        moduleId,
        kind,
        !this.options.getActiveCapabilityIds(kind).has(moduleId),
      );
    });

    const name = document.createElement("span");
    name.textContent = label;

    const state = document.createElement("strong");
    state.textContent = formatToggleState(active, count);

    item.append(name, state);
    return item;
  }

  private renderTimelineSourceToggle(source: EventTimelineSource): HTMLButtonElement {
    const item = document.createElement("button");
    item.type = "button";
    item.className = "module-summary-item";
    item.dataset.active = source.active ? "true" : "false";
    item.setAttribute("aria-pressed", source.active ? "true" : "false");
    item.addEventListener("click", () => {
      source.setActive(!source.active);
      this.options.syncTimelineEvents();
      this.options.syncTimelineRanges();
      this.options.renderTimelineEventCount();
      this.options.requestConfigSync();
      this.renderSummary();
    });

    const name = document.createElement("span");
    name.textContent = getTimelineSourceLabel(source);

    const state = document.createElement("strong");
    state.textContent = formatToggleState(source.active, source.count);

    item.append(name, state);
    return item;
  }

  private renderBoostPickupFiltersWindow(): void {
    const ctx = this.options.getContext();
    const panel = this.options.boostPickupFilters.renderSettings(ctx, {
      showHeader: false,
    });
    this.options.elements.boostPickupFilters.replaceChildren(panel);
  }

  private renderTouchControlsWindow(): void {
    const ctx = this.options.getContext();
    const touchModule = this.options.modules.find((mod) => mod.id === TOUCH_MODULE_ID);
    const panel = touchModule?.renderSettings?.(ctx) ?? null;
    this.options.elements.touchControls.replaceChildren();
    if (panel instanceof HTMLElement) {
      this.options.elements.touchControls.append(panel);
    }
  }
}

function renderModuleSummaryGroup(title: string, items: HTMLButtonElement[]): HTMLElement {
  const group = document.createElement("section");
  group.className = "module-summary-group";

  const heading = document.createElement("h3");
  heading.textContent = title;

  const list = document.createElement("div");
  list.className = "module-list";
  list.append(...items);

  group.append(heading, list);
  return group;
}

function formatToggleState(active: boolean, count?: number): string {
  const state = active ? "On" : "Off";
  return count == null ? state : `${state} ${count}`;
}

function getTimelineSourceLabel(source: EventTimelineSource): string {
  if (source.group === "Replay") {
    return source.label;
  }
  return `${source.label} events`;
}

function getCapabilityLabel(mod: StatModule, kind: ModuleCapabilityKind): string {
  const timelineLabels: Record<string, string> = {
    "absolute-positioning:ranges": "Position zones",
    "backboard:events": "Backboard",
    "ball-carry:events": "Ball carry",
    "boost:ranges": "Boost pickup timeline",
    "bump:events": "Bump",
    "ceiling-shot:events": "Ceiling shot",
    "demo:events": "Demo",
    "dodge-reset:events": "Dodge refresh",
    "double-tap:events": "Double tap",
    "fifty-fifty:events": "50/50",
    "fifty-fifty:ranges": "50/50",
    "dodge:events": "Dodge",
    "half-flip:events": "Half flip",
    "musty-flick:events": "Musty flick",
    "possession:ranges": "Possession",
    "powerslide:events": "Powerslide",
    "powerslide:ranges": "Powerslide",
    "ball_half:ranges": "Half control",
    "rush:ranges": "Rush",
    "speed-flip:events": "Speed flip",
    "touch:events": "Touch",
    "wavedash:events": "Wavedash",
  };
  const inGameVisualizationLabels: Record<string, string> = {
    "absolute-positioning": "Position zones",
    "ceiling-shot": "Ceiling shot labels",
    "fifty-fifty": "50/50 labels",
    dodge: "Dodge impulse arrows",
    ball_half: "Half control",
    "relative-positioning": "Player roles",
    "speed-flip": "Speed flip labels",
    touch: "Touch labels",
  };

  if (kind === "effects") {
    return inGameVisualizationLabels[mod.id] ?? mod.label;
  }

  return timelineLabels[`${mod.id}:${kind}`] ?? `${mod.label} timeline`;
}

export function createModuleControlsController(
  options: ModuleControlsOptions,
): ModuleControlsController {
  return new ModuleControlsController(options);
}
