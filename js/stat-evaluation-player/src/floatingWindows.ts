import {
  mapWindowPlacementToViewport,
  type ConfigViewportSize,
  type SingletonWindowConfig,
  type SingletonWindowId,
  type WindowPlacementConfig,
} from "./playerConfig.ts";

export function mustElement<T extends HTMLElement>(root: ParentNode, selector: string): T {
  const element = root.querySelector(selector);
  if (!(element instanceof HTMLElement)) {
    throw new Error(`Missing element for selector: ${selector}`);
  }

  return element as T;
}

export class FloatingWindowController {
  private nextWindowZIndex = 30;

  constructor(
    private readonly getRoot: () => ParentNode,
    private readonly onPlacementChange: () => void,
  ) {}

  resetZIndex(): void {
    this.nextWindowZIndex = 30;
  }

  getElementWindowId(element: HTMLElement): string | null {
    return element.closest<HTMLElement>("[data-window-id]")?.dataset.windowId ?? null;
  }

  getSingletonWindowConfigs(ids: SingletonWindowId[]): SingletonWindowConfig[] {
    const configs: SingletonWindowConfig[] = [];
    const root = this.getRoot();
    for (const id of ids) {
      const element = root.querySelector<HTMLElement>(`[data-window-id="${id}"]`);
      if (element) {
        configs.push({
          id,
          placement: this.readWindowPlacement(element),
        });
      }
    }
    return configs;
  }

  applyWindowConfigs(configs: SingletonWindowConfig[]): void {
    const root = this.getRoot();
    for (const windowConfig of configs) {
      const element = root.querySelector<HTMLElement>(`[data-window-id="${windowConfig.id}"]`);
      if (element) {
        this.applyWindowPlacement(element, windowConfig.placement);
      }
    }
  }

  showWindow(id: SingletonWindowId): void {
    const windowEl = mustElement<HTMLElement>(this.getRoot(), `[data-window-id="${id}"]`);
    windowEl.hidden = false;
    this.bringWindowToFront(windowEl);
    this.onPlacementChange();
  }

  toggleWindow(id: SingletonWindowId): void {
    const windowEl = mustElement<HTMLElement>(this.getRoot(), `[data-window-id="${id}"]`);
    windowEl.hidden = !windowEl.hidden;
    if (!windowEl.hidden) {
      this.bringWindowToFront(windowEl);
    }
    this.onPlacementChange();
  }

  hideWindow(id: string): void {
    const windowEl = mustElement<HTMLElement>(this.getRoot(), `[data-window-id="${id}"]`);
    windowEl.hidden = true;
    this.onPlacementChange();
  }

  installDragging(root: HTMLElement, signal: AbortSignal): void {
    root.addEventListener(
      "pointerdown",
      (event) => {
        if (!(event.target instanceof HTMLElement) || this.isInteractiveDragTarget(event.target)) {
          return;
        }

        const windowEl = event.target.closest<HTMLElement>("[data-window-id]");
        if (!windowEl || windowEl.hidden) {
          return;
        }

        this.bringWindowToFront(windowEl);
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
          this.onPlacementChange();
        };

        windowEl.addEventListener("pointermove", onPointerMove);
        windowEl.addEventListener("pointerup", onPointerUp);
        windowEl.addEventListener("pointercancel", onPointerUp);
      },
      { signal },
    );
  }

  private getCurrentViewportSize(): ConfigViewportSize {
    return {
      width: Math.max(1, window.innerWidth),
      height: Math.max(1, window.innerHeight),
    };
  }

  private readWindowCoordinate(windowEl: HTMLElement, propertyName: string): number {
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

  readWindowPlacement(windowEl: HTMLElement): WindowPlacementConfig {
    const zIndex = Number.parseInt(windowEl.style.zIndex, 10);
    return {
      x: this.readWindowCoordinate(windowEl, "--window-x"),
      y: this.readWindowCoordinate(windowEl, "--window-y"),
      viewport: this.getCurrentViewportSize(),
      zIndex: Number.isFinite(zIndex) ? zIndex : undefined,
      visible: !windowEl.hidden,
    };
  }

  applyWindowPlacement(windowEl: HTMLElement, placement: WindowPlacementConfig): void {
    const mapped = mapWindowPlacementToViewport(placement, this.getCurrentViewportSize());
    windowEl.style.setProperty("--window-x", `${mapped.x}px`);
    windowEl.style.setProperty("--window-y", `${mapped.y}px`);
    windowEl.hidden = !placement.visible;
    if (placement.zIndex !== undefined) {
      windowEl.style.zIndex = `${placement.zIndex}`;
      this.nextWindowZIndex = Math.max(this.nextWindowZIndex, placement.zIndex + 1);
    }
  }

  bringWindowToFront(windowEl: HTMLElement): void {
    windowEl.style.zIndex = `${this.nextWindowZIndex++}`;
  }

  private isInteractiveDragTarget(target: EventTarget | null): boolean {
    return (
      target instanceof Element &&
      Boolean(target.closest("button, input, select, textarea, option, label, a, [data-no-drag]"))
    );
  }
}
