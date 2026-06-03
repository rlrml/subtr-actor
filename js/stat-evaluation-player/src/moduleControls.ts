import type { BoostPickupFilterController } from "./boostPickupFilters.ts";
import {
  RELATIVE_POSITIONING_MODULE_ID,
  type StatModule,
  type StatModuleContext,
} from "./statModules.ts";

export type ModuleCapabilityKind = "events" | "ranges" | "effects";

const RENDER_EFFECT_MODULE_IDS = new Set([
  "ceiling-shot",
  "fifty-fifty",
  "pressure",
  RELATIVE_POSITIONING_MODULE_ID,
  "absolute-positioning",
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
  getActiveModules(): readonly StatModule[];
  getActiveCapabilityIds(kind: ModuleCapabilityKind): ReadonlySet<string>;
  getBoostPickupAnimationEnabled(): boolean;
  getBoostPadOverlayEnabled(): boolean;
  toggleCapability(id: string, kind: ModuleCapabilityKind, enabled: boolean): void;
  toggleBoostPickupAnimation(): void;
  toggleBoostPadOverlay(): void;
}

export class ModuleControlsController {
  constructor(private readonly options: ModuleControlsOptions) {}

  renderSummary(): void {
    const { summary } = this.options.elements;
    summary.replaceChildren();

    const timelineToggles: HTMLButtonElement[] = [];
    const inGameVisualizationToggles: HTMLButtonElement[] = [];

    for (const mod of this.options.modules) {
      const hasRenderEffect = RENDER_EFFECT_MODULE_IDS.has(mod.id);
      if (!mod.getTimelineEvents && !mod.getTimelineRanges && !hasRenderEffect) {
        continue;
      }

      if (mod.getTimelineEvents) {
        timelineToggles.push(
          this.renderCapabilityToggle(mod.id, getCapabilityLabel(mod, "events"), "events"),
        );
      }
      if (mod.getTimelineRanges) {
        timelineToggles.push(
          this.renderCapabilityToggle(mod.id, getCapabilityLabel(mod, "ranges"), "ranges"),
        );
      }
      if (hasRenderEffect) {
        inGameVisualizationToggles.push(
          this.renderCapabilityToggle(mod.id, getCapabilityLabel(mod, "effects"), "effects"),
        );
      }
    }

    inGameVisualizationToggles.push(this.renderBoostPickupAnimationToggle());
    inGameVisualizationToggles.push(this.renderBoostPadOverlayToggle());

    summary.append(
      renderModuleSummaryGroup("Timeline visualizations", timelineToggles),
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

  private renderBoostPadOverlayToggle(): HTMLButtonElement {
    const active = this.options.getBoostPadOverlayEnabled();
    const item = document.createElement("button");
    item.type = "button";
    item.className = "module-summary-item";
    item.dataset.active = active ? "true" : "false";
    item.setAttribute("aria-pressed", active ? "true" : "false");
    item.addEventListener("click", this.options.toggleBoostPadOverlay);

    const name = document.createElement("span");
    name.textContent = "Boost pad locations";

    const state = document.createElement("strong");
    state.textContent = active ? "On" : "Off";

    item.append(name, state);
    return item;
  }

  private renderCapabilityToggle(
    moduleId: string,
    label: string,
    kind: ModuleCapabilityKind,
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
    state.textContent = active ? "On" : "Off";

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
    "half-flip:events": "Half flip",
    "musty-flick:events": "Musty flick",
    "possession:ranges": "Possession",
    "powerslide:events": "Powerslide",
    "pressure:ranges": "Half control",
    "rush:ranges": "Rush",
    "speed-flip:events": "Speed flip",
    "touch:events": "Touch",
    "wavedash:events": "Wavedash",
  };
  const inGameVisualizationLabels: Record<string, string> = {
    "absolute-positioning": "Position zones",
    "ceiling-shot": "Ceiling shot labels",
    "fifty-fifty": "50/50 labels",
    pressure: "Half control",
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
