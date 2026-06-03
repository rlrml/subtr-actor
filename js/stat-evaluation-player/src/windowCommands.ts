import type { FloatingWindowController } from "./floatingWindows.ts";
import type { SingletonWindowId } from "./playerConfig.ts";

export interface WindowCommandController {
  bringWindowToFront(windowEl: HTMLElement): void;
  showWindow(id: SingletonWindowId): void;
  toggleWindow(id: SingletonWindowId): void;
  hideWindow(id: string): void;
  setLauncherOpen(open: boolean): void;
  openReplayFilePicker(): void;
  installWindowDragging(root: HTMLElement, signal: AbortSignal): void;
  getElementWindowId(element: HTMLElement): string | null;
}

export interface WindowCommandControllerOptions {
  getFloatingWindowController(): FloatingWindowController | null;
  getLauncherMenu(): HTMLDivElement;
  getLauncherToggle(): HTMLButtonElement;
  getFileInput(): HTMLInputElement;
}

export function createWindowCommandController(
  options: WindowCommandControllerOptions,
): WindowCommandController {
  const setLauncherOpen = (open: boolean) => {
    const launcherToggle = options.getLauncherToggle();
    options.getLauncherMenu().hidden = !open;
    launcherToggle.setAttribute("aria-label", open ? "Close menu" : "Open menu");
    launcherToggle.setAttribute("aria-expanded", open ? "true" : "false");
  };

  return {
    bringWindowToFront(windowEl) {
      options.getFloatingWindowController()?.bringToFront(windowEl);
    },

    showWindow(id) {
      options.getFloatingWindowController()?.show(id);
    },

    toggleWindow(id) {
      options.getFloatingWindowController()?.toggle(id);
    },

    hideWindow(id) {
      options.getFloatingWindowController()?.hide(id);
    },

    setLauncherOpen,

    openReplayFilePicker() {
      options.getFileInput().click();
      setLauncherOpen(false);
    },

    installWindowDragging(root, signal) {
      options.getFloatingWindowController()?.installDragging(root, signal);
    },

    getElementWindowId(element) {
      return element.closest<HTMLElement>("[data-window-id]")?.dataset.windowId ?? null;
    },
  };
}
