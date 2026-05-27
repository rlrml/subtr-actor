import type { StatsWindowKind } from "./playerConfig.ts";
import type { StatDefinition, StatScopeKind } from "./statRegistry.ts";
import { getStatDefinitionSearchMatches } from "./statSearch.ts";
import type { StatsWindowState } from "./statsWindowTypes.ts";

export interface StatsWindowPickerDeps {
  getAllowedScope(kind: StatsWindowKind): StatScopeKind | null;
  getDefaultAdHocTargetId(definition: StatDefinition): string;
  getStatRegistry(): readonly StatDefinition[];
  hasScopeSelector(kind: StatsWindowKind): boolean;
  renderStatsWindow(statsWindow: StatsWindowState): void;
  scheduleConfigUrlUpdate(): void;
}

export function renderStatsWindowAddControl(
  statsWindow: StatsWindowState,
  deps: StatsWindowPickerDeps,
): void {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "stats-window-add-button";
  button.textContent = "+";
  button.title = "Add stat";
  button.setAttribute("aria-label", "Add stat");
  button.setAttribute("aria-expanded", String(statsWindow.pickerOpen));
  activateButton(button, () => {
    statsWindow.pickerOpen = !statsWindow.pickerOpen;
    deps.renderStatsWindow(statsWindow);
  });

  if (deps.hasScopeSelector(statsWindow.kind)) {
    const scopeRow = statsWindow.body.querySelector(".stats-window-scope-row");
    scopeRow?.append(button);
    return;
  }

  const toolbar = document.createElement("div");
  toolbar.className = "stats-window-toolbar";
  toolbar.append(button);
  statsWindow.body.append(toolbar);
}

export function renderStatsWindowPicker(
  statsWindow: StatsWindowState,
  deps: StatsWindowPickerDeps,
): void {
  const picker = document.createElement("div");
  picker.className = "stats-window-picker";
  picker.hidden = !statsWindow.pickerOpen;
  if (picker.hidden) {
    statsWindow.body.append(picker);
    return;
  }

  const allowedScope = deps.getAllowedScope(statsWindow.kind);
  const queryInput = document.createElement("input");
  queryInput.type = "search";
  queryInput.placeholder = "Search stats";
  queryInput.value = statsWindow.query;
  queryInput.dataset.statsWindowSearch = statsWindow.id;

  const list = document.createElement("div");
  list.className = "stats-window-picker-list";
  queryInput.addEventListener("input", () => {
    statsWindow.query = queryInput.value;
    renderStatsWindowPickerList(statsWindow, list, allowedScope, deps);
  });

  renderStatsWindowPickerList(statsWindow, list, allowedScope, deps);

  picker.append(queryInput, list);
  statsWindow.body.append(picker);
}

function activateButton(button: HTMLButtonElement, callback: () => void): void {
  let pointerActivated = false;
  button.addEventListener("pointerdown", (event) => {
    if (button.disabled) {
      return;
    }
    pointerActivated = true;
    event.preventDefault();
    callback();
  });
  button.addEventListener("click", () => {
    if (pointerActivated) {
      pointerActivated = false;
      return;
    }
    if (!button.disabled) {
      callback();
    }
  });
}

function renderStatsWindowPickerList(
  statsWindow: StatsWindowState,
  list: HTMLElement,
  allowedScope: StatScopeKind | null,
  deps: StatsWindowPickerDeps,
): void {
  list.replaceChildren();

  const statRegistry = [...deps.getStatRegistry()];
  const scopeDefinitions = allowedScope
    ? statRegistry.filter((definition) => definition.scope === allowedScope)
    : statRegistry;
  const definitions = getStatDefinitionSearchMatches(scopeDefinitions, statsWindow.query);

  const groupByCategory = new Map<string, StatDefinition[]>();
  for (const definition of definitions) {
    const group = groupByCategory.get(definition.category) ?? [];
    group.push(definition);
    groupByCategory.set(definition.category, group);
  }

  for (const [category, group] of groupByCategory) {
    if (group.length < 2) continue;
    const addGroup = document.createElement("button");
    addGroup.type = "button";
    addGroup.className = "stats-window-picker-item";
    addGroup.innerHTML = `<span>Add all ${category}</span><strong>${group.length}</strong>`;
    activateButton(addGroup, () => {
      for (const definition of group) {
        addStatToWindow(statsWindow, definition, deps);
      }
      deps.renderStatsWindow(statsWindow);
      deps.scheduleConfigUrlUpdate();
    });
    list.append(addGroup);
  }

  for (const definition of definitions) {
    const item = document.createElement("button");
    item.type = "button";
    item.className = "stats-window-picker-item";
    item.innerHTML = `<span>${definition.label}</span><strong>${definition.scope}</strong>`;
    item.disabled =
      statsWindow.kind !== "ad-hoc" &&
      statsWindow.entries.some((entry) => entry.statId === definition.id);
    activateButton(item, () => {
      addStatToWindow(statsWindow, definition, deps);
      deps.renderStatsWindow(statsWindow);
      deps.scheduleConfigUrlUpdate();
    });
    list.append(item);
  }

  if (definitions.length === 0) {
    const empty = document.createElement("p");
    empty.className = "stat-window-empty";
    empty.textContent = statRegistry.length === 0 ? "No stats available." : "No matching stats.";
    list.append(empty);
  }
}

function addStatToWindow(
  statsWindow: StatsWindowState,
  definition: StatDefinition,
  deps: StatsWindowPickerDeps,
): void {
  const targetId =
    statsWindow.kind === "ad-hoc" ? deps.getDefaultAdHocTargetId(definition) : undefined;
  if (
    statsWindow.entries.some(
      (entry) => entry.statId === definition.id && entry.targetId === targetId,
    )
  ) {
    return;
  }
  statsWindow.entries.push({
    key: `${statsWindow.id}:${definition.id}:${targetId ?? "scope"}`,
    statId: definition.id,
    targetId,
  });
}
