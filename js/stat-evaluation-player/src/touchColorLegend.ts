import { TOUCH_COLOR_LEGEND_GROUPS } from "./touchOverlay.ts";
import { normalizeTouchOverlayColorModes, TOUCH_OVERLAY_COLOR_MODE_ORDER } from "./touchOverlay.ts";
import type { TouchOverlayColorMode } from "./touchOverlay.ts";

export const TOUCH_COLOR_MODES_CHANGE_EVENT = "subtr-actor:touch-color-modes-change";

export const TOUCH_COLOR_LEGEND_MODE_LABELS: Record<TouchOverlayColorMode, string> = {
  team: "Team",
  intention: "Intention",
  kind: "Hit strength",
  height_band: "Height",
  surface: "Surface",
  dodge_state: "Dodge",
  flag: "Flags",
};

const TOUCH_COLOR_LEGEND_MODES: Array<{ title: string; mode: TouchOverlayColorMode }> = [
  { title: "Team", mode: "team" },
  { title: "Intention", mode: "intention" },
  { title: "Hit strength", mode: "kind" },
  { title: "Height", mode: "height_band" },
  { title: "Surface", mode: "surface" },
  { title: "Dodge", mode: "dodge_state" },
  { title: "Flags", mode: "flag" },
];

function modeForTitle(title: string): TouchOverlayColorMode {
  return TOUCH_COLOR_LEGEND_MODES.find((entry) => entry.title === title)?.mode ?? "team";
}

function colorHex(color: number): string {
  return `#${color.toString(16).padStart(6, "0")}`;
}

export function renderTouchColorLegend(
  root: HTMLElement,
  selectedModes: readonly TouchOverlayColorMode[],
): void {
  root.replaceChildren();
  const activeModes = normalizeTouchOverlayColorModes(selectedModes);
  const activeModeSet = new Set(activeModes);

  const explainer = document.createElement("div");
  explainer.className = "touch-color-legend-explainer";

  const inner = document.createElement("span");
  inner.textContent = "Toggle sections to add or remove rings";

  const outer = document.createElement("span");
  outer.textContent = "The outermost enabled ring sets the label tint";

  explainer.append(inner, outer);
  root.append(explainer);

  for (const group of TOUCH_COLOR_LEGEND_GROUPS) {
    const mode = modeForTitle(group.title);
    const selected = activeModeSet.has(mode);
    const section = document.createElement("section");
    section.className = "touch-color-legend-group";
    section.dataset.active = selected ? "true" : "false";

    const heading = document.createElement("button");
    heading.type = "button";
    heading.className = "touch-color-legend-heading";
    heading.dataset.colorMode = mode;
    heading.dataset.active = selected ? "true" : "false";
    heading.textContent = group.title;
    heading.addEventListener("click", () => {
      const nextModeSet = new Set(activeModeSet);
      if (nextModeSet.has(mode)) {
        nextModeSet.delete(mode);
      } else {
        nextModeSet.add(mode);
      }
      const nextModes = TOUCH_OVERLAY_COLOR_MODE_ORDER.filter((candidate) =>
        nextModeSet.has(candidate),
      );
      root.dispatchEvent(
        new CustomEvent<{ colorModes: TouchOverlayColorMode[] }>(TOUCH_COLOR_MODES_CHANGE_EVENT, {
          bubbles: true,
          detail: { colorModes: nextModes },
        }),
      );
      renderTouchColorLegend(root, nextModes);
    });

    const list = document.createElement("div");
    list.className = "touch-color-legend-list";

    for (const entry of group.entries) {
      const item = document.createElement("div");
      item.className = "touch-color-legend-item";

      const swatch = document.createElement("span");
      swatch.className = "touch-color-legend-swatch";
      swatch.style.background = colorHex(entry.color);

      const label = document.createElement("span");
      label.textContent = entry.label;

      item.append(swatch, label);
      list.append(item);
    }

    section.append(heading, list);
    root.append(section);
  }
}
