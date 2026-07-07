import {
  mapWindowPlacementToViewport,
  type ConfigViewportSize,
  type SingletonWindowConfig,
  type SingletonWindowId,
  type WindowPlacementConfig,
} from "./playerConfig.ts";

const SINGLETON_WINDOW_IDS: SingletonWindowId[] = [
  "camera",
  "scoreboard",
  "playback",
  "recording",
  "training-pack",
  "mechanics",
  "event-playlist",
  "mechanics-review",
  "replay-loading",
  "boost-pickups",
  "touch-controls",
  "touch-legend",
  "shot-visualization",
  "missed-events",
];

export interface FloatingWindowControllerOptions {
  getRoot(): HTMLElement | Document;
  requestConfigSync(): void;
}

export class FloatingWindowController {
  private nextZIndex = 30;

  constructor(private readonly options: FloatingWindowControllerOptions) {}

  reset(): void {
    this.nextZIndex = 30;
  }

  bringToFront(windowEl: HTMLElement): void {
    windowEl.style.zIndex = `${this.nextZIndex++}`;
  }

  show(id: SingletonWindowId): void {
    const windowEl = this.mustWindow(id);
    windowEl.hidden = false;
    this.bringToFront(windowEl);
    this.options.requestConfigSync();
  }

  toggle(id: SingletonWindowId): void {
    const windowEl = this.mustWindow(id);
    windowEl.hidden = !windowEl.hidden;
    if (!windowEl.hidden) {
      this.bringToFront(windowEl);
    }
    this.options.requestConfigSync();
  }

  hide(id: string): void {
    const windowEl = this.mustWindow(id);
    windowEl.hidden = true;
    this.options.requestConfigSync();
  }

  readPlacement(windowEl: HTMLElement): WindowPlacementConfig {
    const zIndex = Number.parseInt(windowEl.style.zIndex, 10);
    return {
      x: this.readCoordinate(windowEl, "--window-x"),
      y: this.readCoordinate(windowEl, "--window-y"),
      viewport: getCurrentViewportSize(),
      zIndex: Number.isFinite(zIndex) ? zIndex : undefined,
      visible: !windowEl.hidden,
    };
  }

  applyPlacement(windowEl: HTMLElement, placement: WindowPlacementConfig): void {
    const mapped = mapWindowPlacementToViewport(placement, getCurrentViewportSize());
    windowEl.style.setProperty("--window-x", `${mapped.x}px`);
    windowEl.style.setProperty("--window-y", `${mapped.y}px`);
    windowEl.hidden = !placement.visible;
    if (placement.zIndex !== undefined) {
      windowEl.style.zIndex = `${placement.zIndex}`;
      this.nextZIndex = Math.max(this.nextZIndex, placement.zIndex + 1);
    }
  }

  getSingletonConfigs(): SingletonWindowConfig[] {
    const configs: SingletonWindowConfig[] = [];
    const root = this.options.getRoot();
    for (const id of SINGLETON_WINDOW_IDS) {
      const element = root.querySelector<HTMLElement>(`[data-window-id="${id}"]`);
      if (element) {
        configs.push({
          id,
          placement: this.readPlacement(element),
        });
      }
    }
    return configs;
  }

  applySingletonConfigs(configs: readonly SingletonWindowConfig[]): void {
    const root = this.options.getRoot();
    for (const windowConfig of configs) {
      const element = root.querySelector<HTMLElement>(`[data-window-id="${windowConfig.id}"]`);
      if (element) {
        this.applyPlacement(element, windowConfig.placement);
      }
    }
  }

  installDragging(root: HTMLElement, signal: AbortSignal): void {
    root.addEventListener(
      "pointerdown",
      (event) => {
        if (!(event.target instanceof HTMLElement) || isInteractiveDragTarget(event.target)) {
          return;
        }

        const windowEl = event.target.closest<HTMLElement>("[data-window-id]");
        if (!windowEl || windowEl.hidden) {
          return;
        }

        this.bringToFront(windowEl);
        const startX = event.clientX;
        const startY = event.clientY;
        const rect = windowEl.getBoundingClientRect();
        const pointerId = event.pointerId;

        windowEl.setPointerCapture(pointerId);
        event.preventDefault();

        const onPointerMove = (moveEvent: PointerEvent) => {
          const nextX = Math.max(
            8,
            Math.min(window.innerWidth - 120, rect.left + moveEvent.clientX - startX),
          );
          const nextY = Math.max(
            8,
            Math.min(window.innerHeight - 100, rect.top + moveEvent.clientY - startY),
          );
          windowEl.style.setProperty("--window-x", `${nextX}px`);
          windowEl.style.setProperty("--window-y", `${nextY}px`);
        };

        const onPointerUp = () => {
          windowEl.releasePointerCapture(pointerId);
          windowEl.removeEventListener("pointermove", onPointerMove);
          windowEl.removeEventListener("pointerup", onPointerUp);
          windowEl.removeEventListener("pointercancel", onPointerUp);
          this.options.requestConfigSync();
        };

        windowEl.addEventListener("pointermove", onPointerMove);
        windowEl.addEventListener("pointerup", onPointerUp);
        windowEl.addEventListener("pointercancel", onPointerUp);
      },
      { signal },
    );
  }

  private mustWindow(id: string): HTMLElement {
    const windowEl = this.options.getRoot().querySelector<HTMLElement>(`[data-window-id="${id}"]`);
    if (!windowEl) {
      throw new Error(`Missing window for id: ${id}`);
    }
    return windowEl;
  }

  private readCoordinate(windowEl: HTMLElement, propertyName: string): number {
    const inlineValue = windowEl.style.getPropertyValue(propertyName).trim();
    const computedValue = getComputedStyle(windowEl).getPropertyValue(propertyName).trim();
    const rawValue = inlineValue || computedValue;
    const parsed = Number.parseFloat(rawValue);
    if (Number.isFinite(parsed)) {
      return parsed;
    }

    const rect = windowEl.getBoundingClientRect();
    return propertyName === "--window-y" ? rect.top : rect.left;
  }
}

function getCurrentViewportSize(): ConfigViewportSize {
  return {
    width: Math.max(1, window.innerWidth),
    height: Math.max(1, window.innerHeight),
  };
}

function isInteractiveDragTarget(target: EventTarget | null): boolean {
  return (
    target instanceof Element &&
    Boolean(target.closest("button, input, select, textarea, option, label, a, [data-no-drag]"))
  );
}

export function createFloatingWindowController(
  options: FloatingWindowControllerOptions,
): FloatingWindowController {
  return new FloatingWindowController(options);
}
