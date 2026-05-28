import type {
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginDefinition,
  ReplayPlayerPluginStateContext,
  ReplayPlayerState,
} from "./types";

type InstalledReplayPlayerPlugin = {
  definition: ReplayPlayerPluginDefinition;
  plugin: ReplayPlayerPlugin;
};

interface ReplayPlayerPluginHostOptions {
  createContext(): ReplayPlayerPluginContext;
  getState(): ReplayPlayerState;
  render(): void;
}

export class ReplayPlayerPluginHost {
  private readonly plugins: InstalledReplayPlayerPlugin[] = [];

  constructor(private readonly options: ReplayPlayerPluginHostOptions) {}

  install(definition: ReplayPlayerPluginDefinition, renderAfterSetup: boolean): () => void {
    const plugin = typeof definition === "function" ? definition() : definition;

    if (this.plugins.some((entry) => entry.plugin.id === plugin.id)) {
      throw new Error(`Replay player plugin "${plugin.id}" is already installed`);
    }

    const entry = { definition, plugin };
    this.plugins.push(entry);
    plugin.setup?.(this.options.createContext());
    plugin.onStateChange?.(this.createStateContext(this.options.getState()));

    if (renderAfterSetup) {
      this.options.render();
    }

    return () => {
      const index = this.plugins.indexOf(entry);
      if (index < 0) {
        return;
      }
      this.plugins.splice(index, 1);
      plugin.teardown?.(this.options.createContext());
      this.options.render();
    };
  }

  remove(id: string): boolean {
    const index = this.plugins.findIndex((entry) => entry.plugin.id === id);
    if (index < 0) {
      return false;
    }

    const [entry] = this.plugins.splice(index, 1);
    entry.plugin.teardown?.(this.options.createContext());
    this.options.render();
    return true;
  }

  getPlugins(): ReplayPlayerPlugin[] {
    return this.plugins.map((entry) => entry.plugin);
  }

  teardownAll(): void {
    while (this.plugins.length > 0) {
      const entry = this.plugins.pop();
      entry?.plugin.teardown?.(this.options.createContext());
    }
  }

  notifyStateChange(state: ReplayPlayerState): void {
    const pluginStateContext = this.createStateContext(state);
    for (const entry of this.plugins) {
      entry.plugin.onStateChange?.(pluginStateContext);
    }
  }

  private createStateContext(state: ReplayPlayerState): ReplayPlayerPluginStateContext {
    return {
      ...this.options.createContext(),
      state,
    };
  }
}
