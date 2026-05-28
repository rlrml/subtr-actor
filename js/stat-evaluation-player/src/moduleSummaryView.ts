import type { StatModule } from "./statModules.ts";

export type ModuleCapabilityKind = "events" | "ranges" | "effects";

export interface ModuleSummaryViewOptions {
  container: HTMLElement;
  modules: StatModule[];
  renderEffectModuleIds: Set<string>;
  getActiveCapabilityIds(kind: ModuleCapabilityKind): Set<string>;
  toggleCapability(id: string, kind: ModuleCapabilityKind, enabled: boolean): void;
  boostPickupAnimationEnabled: boolean;
  toggleBoostPickupAnimation(): void;
  boostPadOverlayEnabled: boolean;
  toggleBoostPadOverlay(): void;
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

function renderToggle(label: string, active: boolean, onClick: () => void): HTMLButtonElement {
  const item = document.createElement("button");
  item.type = "button";
  item.className = "module-summary-item";
  item.dataset.active = active ? "true" : "false";
  item.setAttribute("aria-pressed", active ? "true" : "false");
  item.addEventListener("click", onClick);

  const name = document.createElement("span");
  name.textContent = label;

  const state = document.createElement("strong");
  state.textContent = active ? "On" : "Off";

  item.append(name, state);
  return item;
}

function renderCapabilityToggle(
  options: ModuleSummaryViewOptions,
  moduleId: string,
  label: string,
  kind: ModuleCapabilityKind,
): HTMLButtonElement {
  const activeIds = options.getActiveCapabilityIds(kind);
  const active = activeIds.has(moduleId);
  return renderToggle(label, active, () => {
    options.toggleCapability(moduleId, kind, !activeIds.has(moduleId));
  });
}

export function renderModuleSummaryView(options: ModuleSummaryViewOptions): void {
  options.container.replaceChildren();

  const timelineToggles: HTMLButtonElement[] = [];
  const inGameVisualizationToggles: HTMLButtonElement[] = [];

  for (const mod of options.modules) {
    const hasRenderEffect = options.renderEffectModuleIds.has(mod.id);
    if (!mod.getTimelineEvents && !mod.getTimelineRanges && !hasRenderEffect) {
      continue;
    }

    if (mod.getTimelineEvents) {
      timelineToggles.push(
        renderCapabilityToggle(options, mod.id, getCapabilityLabel(mod, "events"), "events"),
      );
    }
    if (mod.getTimelineRanges) {
      timelineToggles.push(
        renderCapabilityToggle(options, mod.id, getCapabilityLabel(mod, "ranges"), "ranges"),
      );
    }
    if (hasRenderEffect) {
      inGameVisualizationToggles.push(
        renderCapabilityToggle(options, mod.id, getCapabilityLabel(mod, "effects"), "effects"),
      );
    }
  }

  inGameVisualizationToggles.push(
    renderToggle(
      "Boost pickup animation",
      options.boostPickupAnimationEnabled,
      options.toggleBoostPickupAnimation,
    ),
  );
  inGameVisualizationToggles.push(
    renderToggle(
      "Boost pad locations",
      options.boostPadOverlayEnabled,
      options.toggleBoostPadOverlay,
    ),
  );

  options.container.append(
    renderModuleSummaryGroup("Timeline visualizations", timelineToggles),
    renderModuleSummaryGroup("In-game visualizations", inGameVisualizationToggles),
  );
}
