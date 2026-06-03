import {
  getEventTimelineSources as getConfiguredEventTimelineSources,
  type EventTimelineSource,
} from "./eventTimelineSources.ts";
import type { StatModule, StatModuleContext } from "./statModules.ts";

export interface EventTimelineControlsOptions {
  readonly body: HTMLElement;
  readonly modules: readonly StatModule[];
  getContext(): StatModuleContext | null;
  getActiveTimelineEventSourceIds(): ReadonlySet<string>;
  getActiveMechanicTimelineKinds(): ReadonlySet<string>;
  toggleEventSource(id: string, enabled: boolean): void;
  setMechanicTimelineKind(kind: string, enabled: boolean): void;
  setupActiveModules(): void;
  syncTimelineEvents(): void;
  syncTimelineRanges(): void;
  renderModuleSummary(): void;
  renderModuleSettings(): void;
  renderTimelineEventCount(): void;
  requestConfigSync(): void;
}

export class EventTimelineControlsController {
  constructor(private readonly options: EventTimelineControlsOptions) {}

  getSources(ctx: StatModuleContext | null = this.options.getContext()): EventTimelineSource[] {
    return getConfiguredEventTimelineSources({
      ctx,
      modules: this.options.modules,
      activeTimelineEventSourceIds: this.options.getActiveTimelineEventSourceIds(),
      activeMechanicTimelineKinds: this.options.getActiveMechanicTimelineKinds(),
      toggleEventSource: this.options.toggleEventSource,
      setMechanicTimelineKind: this.options.setMechanicTimelineKind,
    });
  }

  countVisibleSources(ctx: StatModuleContext): number {
    const goalCount = ctx.replay.timelineEvents.filter((event) => event.kind === "goal").length;
    return (
      goalCount +
      this.getSources(ctx)
        .filter((source) => source.active)
        .reduce((count, source) => count + source.count, 0)
    );
  }

  render(): void {
    const { body } = this.options;
    body.replaceChildren();

    const sources = this.getSources();

    if (sources.length === 0) {
      const empty = document.createElement("p");
      empty.className = "stat-window-empty";
      empty.textContent = "No events loaded.";
      body.append(empty);
      return;
    }

    const actions = document.createElement("div");
    actions.className = "mechanics-actions";

    const allButton = document.createElement("button");
    allButton.type = "button";
    allButton.className = "module-summary-item";
    allButton.addEventListener("click", () => {
      for (const source of sources) {
        source.setActive(true);
      }
      this.options.setupActiveModules();
      this.options.syncTimelineEvents();
      this.options.syncTimelineRanges();
      this.render();
      this.options.renderModuleSummary();
      this.options.renderModuleSettings();
      this.options.renderTimelineEventCount();
      this.options.requestConfigSync();
    });
    const allName = document.createElement("span");
    allName.textContent = "All events";
    const allCount = document.createElement("strong");
    allCount.textContent = `${sources.length}`;
    allButton.append(allName, allCount);

    const noneButton = document.createElement("button");
    noneButton.type = "button";
    noneButton.className = "module-summary-item";
    noneButton.addEventListener("click", () => {
      for (const source of sources) {
        source.setActive(false);
      }
      this.options.setupActiveModules();
      this.options.syncTimelineEvents();
      this.options.syncTimelineRanges();
      this.render();
      this.options.renderModuleSummary();
      this.options.renderModuleSettings();
      this.options.renderTimelineEventCount();
      this.options.requestConfigSync();
    });
    const noneName = document.createElement("span");
    noneName.textContent = "No events";
    const noneState = document.createElement("strong");
    noneState.textContent = "Off";
    noneButton.append(noneName, noneState);

    actions.append(allButton, noneButton);
    body.append(actions);

    const list = this.renderSourceList(sources);
    if (list) {
      body.append(list);
    }
  }

  private renderSourceList(sources: EventTimelineSource[]): HTMLElement | null {
    if (sources.length === 0) {
      return null;
    }

    const list = document.createElement("div");
    list.className = "module-list mechanics-list mechanics-event-list";
    list.style.setProperty(
      "--event-source-columns",
      `${getEventSourceColumnCount(sources.length)}`,
    );

    for (const source of sources) {
      const item = document.createElement("button");
      item.type = "button";
      item.className = "module-summary-item";
      item.dataset.active = source.active ? "true" : "false";
      item.setAttribute("aria-pressed", source.active ? "true" : "false");
      item.addEventListener("click", () => {
        source.setActive(!source.active);
        this.options.syncTimelineEvents();
        this.options.syncTimelineRanges();
        this.render();
        this.options.renderTimelineEventCount();
      });

      const name = document.createElement("span");
      name.textContent = source.label;

      const state = document.createElement("strong");
      state.textContent = `${source.active ? "On" : "Off"} ${source.count}`;

      item.append(name, state);
      list.append(item);
    }

    return list;
  }
}

function getEventSourceColumnCount(sourceCount: number): number {
  if (window.innerWidth < 640) {
    return 1;
  }
  if (window.innerWidth < 900) {
    return sourceCount >= 7 ? 2 : 1;
  }
  if (sourceCount >= 13) {
    return 3;
  }
  if (sourceCount >= 7) {
    return 2;
  }
  return 1;
}

export function createEventTimelineControlsController(
  options: EventTimelineControlsOptions,
): EventTimelineControlsController {
  return new EventTimelineControlsController(options);
}
