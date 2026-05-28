import type { ReplayPlayerPluginContext } from "./types";

export interface TimelineOverlayElementRefs {
  root: HTMLDivElement;
  shell: HTMLDivElement;
  rangesRoot: HTMLDivElement;
  range: HTMLInputElement;
  toggleButton: HTMLButtonElement;
  toggleButtonIcon: HTMLSpanElement;
  toggleButtonLabel: HTMLSpanElement;
  currentTimeText: HTMLSpanElement;
  remainingTimeText: HTMLSpanElement;
  eventLanesRoot: HTMLDivElement;
  markers: HTMLDivElement;
  removeWindowListeners(): void;
}

export interface TimelineOverlayElementHandlers {
  beginScrub(): void;
  endScrub(): void;
}

export function createTimelineOverlayElements(
  context: ReplayPlayerPluginContext,
  handlers: TimelineOverlayElementHandlers,
): TimelineOverlayElementRefs {
  const root = document.createElement("div");
  root.className = "sap-tl-root";
  const shell = document.createElement("div");
  shell.className = "sap-tl-shell";
  shell.dataset.scrubbing = "false";

  const topLine = document.createElement("div");
  topLine.className = "sap-tl-topline";

  const primary = document.createElement("div");
  primary.className = "sap-tl-primary";

  const toggleButton = document.createElement("button");
  toggleButton.type = "button";
  toggleButton.className = "sap-tl-toggle sap-tl-track-toggle";
  const toggleButtonIcon = document.createElement("span");
  toggleButtonIcon.className = "sap-tl-toggle-icon";
  toggleButtonIcon.setAttribute("aria-hidden", "true");
  toggleButtonIcon.textContent = ">";
  const toggleButtonLabel = document.createElement("span");
  toggleButtonLabel.className = "sap-tl-toggle-label";
  toggleButtonLabel.textContent = "Play";
  toggleButton.append(toggleButtonIcon, toggleButtonLabel);
  toggleButton.addEventListener("click", () => {
    context.player.togglePlayback();
  });

  const currentTimeText = document.createElement("span");
  currentTimeText.className = "sap-tl-current";
  currentTimeText.textContent = "0:00.00";

  const remainingTimeText = document.createElement("span");
  remainingTimeText.className = "sap-tl-remaining";
  remainingTimeText.textContent = "-0:00.00";

  primary.append(currentTimeText);
  topLine.append(primary, remainingTimeText);

  const trackWrap = document.createElement("div");
  trackWrap.className = "sap-tl-track-wrap";

  const rangesRoot = document.createElement("div");
  rangesRoot.className = "sap-tl-ranges";
  rangesRoot.hidden = true;

  const eventLanesRoot = document.createElement("div");
  eventLanesRoot.className = "sap-tl-event-lanes";
  eventLanesRoot.hidden = true;

  const trackRail = document.createElement("div");
  trackRail.className = "sap-tl-track-rail";

  const mainRail = document.createElement("div");
  mainRail.className = "sap-tl-main-rail";

  const markers = document.createElement("div");
  markers.className = "sap-tl-markers";

  const range = document.createElement("input");
  range.className = "sap-tl-range";
  range.type = "range";
  range.min = "0";
  range.max = `${context.replay.duration}`;
  range.step = "0.01";
  range.value = "0";

  const handlePointerDown = (): void => {
    handlers.beginScrub();
  };
  const handleInput = (): void => {
    context.player.seek(context.player.projectTimelineTimeToReplay(Number(range.value)));
  };
  const handleWindowPointerUp = (): void => {
    handlers.endScrub();
  };

  range.addEventListener("pointerdown", handlePointerDown);
  range.addEventListener("input", handleInput);
  range.addEventListener("change", handleWindowPointerUp);
  window.addEventListener("pointerup", handleWindowPointerUp);
  window.addEventListener("pointercancel", handleWindowPointerUp);

  trackRail.append(mainRail, markers, range);
  trackWrap.append(rangesRoot, eventLanesRoot, toggleButton, trackRail);
  shell.append(topLine, trackWrap);
  root.append(shell);
  context.container.append(root);

  return {
    root,
    shell,
    rangesRoot,
    range,
    toggleButton,
    toggleButtonIcon,
    toggleButtonLabel,
    currentTimeText,
    remainingTimeText,
    eventLanesRoot,
    markers,
    removeWindowListeners() {
      range.removeEventListener("pointerdown", handlePointerDown);
      range.removeEventListener("input", handleInput);
      range.removeEventListener("change", handleWindowPointerUp);
      window.removeEventListener("pointerup", handleWindowPointerUp);
      window.removeEventListener("pointercancel", handleWindowPointerUp);
    },
  };
}
