import { mustElement } from "./floatingWindows.ts";

export interface StatEvaluationPlayerElements {
  fileInput: HTMLInputElement;
  viewport: HTMLDivElement;
  emptyState: HTMLDivElement;
  emptyLoadReplay: HTMLButtonElement;
  launcherToggle: HTMLButtonElement;
  launcherMenu: HTMLDivElement;
  loadReplayAction: HTMLButtonElement;
  floatingWindowLayer: HTMLDivElement;
  scoreboardWindowBody: HTMLDivElement;
  mechanicsTimelineWindowBody: HTMLDivElement;
  eventPlaylistWindowBody: HTMLDivElement;
  boostPickupFiltersWindowBody: HTMLDivElement;
  touchControlsWindowBody: HTMLDivElement;
  statsWindowLayer: HTMLDivElement;
  togglePlayback: HTMLButtonElement;
  playbackRate: HTMLSelectElement;
  moduleSummaryEl: HTMLDivElement;
  moduleSettingsEl: HTMLDivElement;
  timeReadout: HTMLElement;
  frameReadout: HTMLElement;
  durationReadout: HTMLElement;
  playbackStatusReadout: HTMLElement;
  statusReadout: HTMLElement;
  playersReadout: HTMLElement;
  framesReadout: HTMLElement;
  eventsReadout: HTMLElement;
  skipPostGoalTransitions: HTMLInputElement;
  skipKickoffs: HTMLInputElement;
}

export function getStatEvaluationPlayerElements(root: HTMLElement): StatEvaluationPlayerElements {
  return {
    fileInput: mustElement<HTMLInputElement>(root, "#replay-file"),
    viewport: mustElement<HTMLDivElement>(root, "#viewport"),
    emptyState: mustElement<HTMLDivElement>(root, "#empty-state"),
    emptyLoadReplay: mustElement<HTMLButtonElement>(root, "#empty-load-replay"),
    launcherToggle: mustElement<HTMLButtonElement>(root, "#launcher-toggle"),
    launcherMenu: mustElement<HTMLDivElement>(root, "#launcher-menu"),
    loadReplayAction: mustElement<HTMLButtonElement>(root, "#load-replay-action"),
    floatingWindowLayer: mustElement<HTMLDivElement>(root, "#floating-window-layer"),
    scoreboardWindowBody: mustElement<HTMLDivElement>(root, "#scoreboard-window-body"),
    mechanicsTimelineWindowBody: mustElement<HTMLDivElement>(
      root,
      "#mechanics-timeline-window-body",
    ),
    eventPlaylistWindowBody: mustElement<HTMLDivElement>(root, "#event-playlist-window-body"),
    boostPickupFiltersWindowBody: mustElement<HTMLDivElement>(
      root,
      "#boost-pickup-filters-window-body",
    ),
    touchControlsWindowBody: mustElement<HTMLDivElement>(root, "#touch-controls-window-body"),
    statsWindowLayer: mustElement<HTMLDivElement>(root, "#stats-window-layer"),
    togglePlayback: mustElement<HTMLButtonElement>(root, "#toggle-playback"),
    playbackRate: mustElement<HTMLSelectElement>(root, "#playback-rate"),
    moduleSummaryEl: mustElement<HTMLDivElement>(root, "#module-summary"),
    moduleSettingsEl: mustElement<HTMLDivElement>(root, "#module-settings"),
    timeReadout: mustElement<HTMLElement>(root, "#time-readout"),
    frameReadout: mustElement<HTMLElement>(root, "#frame-readout"),
    durationReadout: mustElement<HTMLElement>(root, "#duration-readout"),
    playbackStatusReadout: mustElement<HTMLElement>(root, "#playback-status-readout"),
    statusReadout: mustElement<HTMLElement>(root, "#status-readout"),
    playersReadout: mustElement<HTMLElement>(root, "#players-readout"),
    framesReadout: mustElement<HTMLElement>(root, "#frames-readout"),
    eventsReadout: mustElement<HTMLElement>(root, "#events-readout"),
    skipPostGoalTransitions: mustElement<HTMLInputElement>(root, "#skip-post-goal-transitions"),
    skipKickoffs: mustElement<HTMLInputElement>(root, "#skip-kickoffs"),
  };
}
