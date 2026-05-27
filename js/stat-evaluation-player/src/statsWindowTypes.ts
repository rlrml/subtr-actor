import type { StatsWindowKind, TeamScope } from "./playerConfig.ts";

export interface SelectedStatEntry {
  key: string;
  statId: string;
  targetId?: string;
}

export interface StatsWindowState {
  readonly id: string;
  readonly kind: StatsWindowKind;
  readonly entries: SelectedStatEntry[];
  playerId: string | null;
  team: TeamScope | null;
  pickerOpen: boolean;
  query: string;
  element: HTMLElement;
  body: HTMLElement;
}
