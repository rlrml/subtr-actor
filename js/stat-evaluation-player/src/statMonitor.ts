import type {
  PlayerStatsSnapshot,
  StatsFrame,
  StatsTimeline,
  TeamStatsSnapshot,
} from "./statsTimeline.ts";

type StatScope = "player" | "team";

type PrimitiveStatValue = boolean | number | string | null;

export interface StatDescriptor {
  readonly id: string;
  readonly scope: StatScope;
  readonly path: readonly string[];
  readonly label: string;
  readonly selectorLabel: string;
  readonly searchText: string;
}

interface StatMonitorSlotElements {
  readonly root: HTMLElement;
  readonly input: HTMLInputElement;
  readonly suggestions: HTMLElement;
  readonly clearButton: HTMLButtonElement;
}

const PLAYER_METADATA_KEYS = new Set(["player_id", "name", "is_team_0"]);
const TEAM_LABELS = {
  team_zero: "Blue Team",
  team_one: "Orange Team",
} as const;

export class StatMonitor {
  private readonly overlayEl: HTMLElement;
  private slots: StatMonitorSlotElements[] = [];
  private readonly selections: Array<StatDescriptor | null> = [null, null];
  private descriptors: StatDescriptor[] = [];
  private currentFrame: StatsFrame | null = null;

  constructor(slotRoots: HTMLElement[], overlayEl: HTMLElement) {
    this.overlayEl = overlayEl;
    this.slots = slotRoots.map((root, index) => this.createSlot(root, index));
    this.renderSlots();
    this.renderOverlay();
  }

  setStatsTimeline(statsTimeline: StatsTimeline | null): void {
    const firstFrame = statsTimeline?.frames[0] ?? null;
    this.descriptors = firstFrame ? buildStatDescriptors(firstFrame) : [];
    const validIds = new Set(this.descriptors.map((descriptor) => descriptor.id));
    for (let index = 0; index < this.selections.length; index += 1) {
      const selected = this.selections[index];
      if (selected && !validIds.has(selected.id)) {
        this.selections[index] = null;
      }
    }
    this.renderSlots();
    this.renderOverlay();
  }

  renderFrame(frame: StatsFrame | null): void {
    this.currentFrame = frame;
    this.renderOverlay();
  }

  destroy(): void {
    this.overlayEl.replaceChildren();
    this.overlayEl.hidden = true;
    for (const slot of this.slots) {
      slot.root.replaceChildren();
    }
  }

  private createSlot(root: HTMLElement, index: number): StatMonitorSlotElements {
    root.replaceChildren();
    root.className = "stat-monitor-slot";

    const label = document.createElement("label");
    label.className = "stat-monitor-input-label";

    const labelText = document.createElement("span");
    labelText.className = "label";
    labelText.textContent = `Slot ${index + 1}`;

    const input = document.createElement("input");
    input.type = "search";
    input.autocomplete = "off";
    input.spellcheck = false;
    input.placeholder = "Search stats";
    input.addEventListener("input", () => {
      this.renderSuggestions(index, input.value);
    });
    input.addEventListener("focus", () => {
      this.renderSuggestions(index, input.value);
    });
    input.addEventListener("keydown", (event) => {
      if (event.key === "Escape") {
        this.hideSuggestions(index);
        input.blur();
      }
    });
    input.addEventListener("blur", () => {
      window.setTimeout(() => this.hideSuggestions(index), 120);
    });

    label.append(labelText, input);

    const suggestions = document.createElement("div");
    suggestions.className = "stat-monitor-suggestions";
    suggestions.hidden = true;

    const clearButton = document.createElement("button");
    clearButton.type = "button";
    clearButton.className = "stat-monitor-clear";
    clearButton.textContent = "Clear";
    clearButton.addEventListener("click", () => {
      this.selections[index] = null;
      input.value = "";
      this.renderSlots();
      this.renderOverlay();
    });

    root.append(label, suggestions, clearButton);
    return { root, input, suggestions, clearButton };
  }

  private renderSlots(): void {
    const disabled = this.descriptors.length === 0;
    this.slots.forEach((slot, index) => {
      const selected = this.selections[index];
      slot.root.dataset.empty = selected ? "false" : "true";
      slot.input.disabled = disabled;
      slot.input.value = selected?.selectorLabel ?? "";
      slot.clearButton.hidden = !selected;
      slot.clearButton.disabled = disabled;
      this.hideSuggestions(index);
    });
  }

  private renderSuggestions(index: number, query: string): void {
    const slot = this.slots[index];
    slot.suggestions.replaceChildren();

    const matches = getFuzzyStatMatches(this.descriptors, query, 8);
    if (matches.length === 0) {
      const empty = document.createElement("div");
      empty.className = "stat-monitor-suggestion-empty";
      empty.textContent = this.descriptors.length === 0
        ? "Load a replay first"
        : "No matching stats";
      slot.suggestions.append(empty);
      slot.suggestions.hidden = false;
      return;
    }

    for (const descriptor of matches) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "stat-monitor-suggestion";
      button.addEventListener("mousedown", (event) => event.preventDefault());
      button.addEventListener("click", () => {
        this.selections[index] = descriptor;
        this.renderSlots();
        this.renderOverlay();
      });

      const label = document.createElement("span");
      label.textContent = descriptor.selectorLabel;
      const scope = document.createElement("strong");
      scope.textContent = descriptor.scope === "player" ? "Players" : "Teams";
      button.append(label, scope);
      slot.suggestions.append(button);
    }

    slot.suggestions.hidden = false;
  }

  private hideSuggestions(index: number): void {
    const slot = this.slots[index];
    slot.suggestions.hidden = true;
    slot.suggestions.replaceChildren();
  }

  private renderOverlay(): void {
    this.overlayEl.replaceChildren();

    const selections = this.selections.filter(
      (selection): selection is StatDescriptor => selection !== null,
    );
    if (!this.currentFrame || selections.length === 0) {
      this.overlayEl.hidden = true;
      return;
    }

    for (const descriptor of selections) {
      this.overlayEl.append(this.renderOverlayCard(descriptor, this.currentFrame));
    }
    this.overlayEl.hidden = false;
  }

  private renderOverlayCard(
    descriptor: StatDescriptor,
    frame: StatsFrame,
  ): HTMLElement {
    const card = document.createElement("section");
    card.className = "stat-monitor-card";

    const header = document.createElement("div");
    header.className = "stat-monitor-card-header";

    const title = document.createElement("span");
    title.className = "stat-monitor-card-title";
    title.textContent = descriptor.label;

    const scope = document.createElement("strong");
    scope.className = "stat-monitor-card-scope";
    scope.textContent = descriptor.scope === "player" ? "Players" : "Teams";
    header.append(title, scope);

    const rows = document.createElement("div");
    rows.className = "stat-monitor-card-rows";

    if (descriptor.scope === "player") {
      for (const player of frame.players) {
        rows.append(renderStatMonitorRow(
          player.name,
          player.is_team_0,
          getValueAtPath(player, descriptor.path),
          descriptor.path,
        ));
      }
    } else {
      rows.append(
        renderStatMonitorRow(
          TEAM_LABELS.team_zero,
          true,
          getValueAtPath(frame.team_zero, descriptor.path),
          descriptor.path,
        ),
        renderStatMonitorRow(
          TEAM_LABELS.team_one,
          false,
          getValueAtPath(frame.team_one, descriptor.path),
          descriptor.path,
        ),
      );
    }

    card.append(header, rows);
    return card;
  }
}

export function buildStatDescriptors(frame: StatsFrame): StatDescriptor[] {
  const descriptors = [
    ...collectScopedDescriptors("player", frame.players[0] ?? null),
    ...collectScopedDescriptors("team", frame.team_zero),
  ];
  return descriptors.sort((left, right) =>
    left.selectorLabel.localeCompare(right.selectorLabel));
}

export function getFuzzyStatMatches(
  descriptors: readonly StatDescriptor[],
  query: string,
  limit: number,
): StatDescriptor[] {
  const normalizedQuery = normalize(query);
  if (!normalizedQuery) {
    return descriptors.slice(0, limit);
  }

  return descriptors
    .map((descriptor) => ({
      descriptor,
      score: scoreFuzzyMatch(descriptor.searchText, normalizedQuery),
    }))
    .filter((match): match is {
      descriptor: StatDescriptor;
      score: number;
    } => match.score !== null)
    .sort((left, right) => {
      if (left.score !== right.score) {
        return left.score - right.score;
      }
      return left.descriptor.selectorLabel.localeCompare(
        right.descriptor.selectorLabel,
      );
    })
    .slice(0, limit)
    .map((match) => match.descriptor);
}

export function formatStatMonitorValue(
  value: unknown,
  path: readonly string[],
): string {
  if (value === null || value === undefined) {
    return "--";
  }
  if (typeof value === "boolean") {
    return value ? "Yes" : "No";
  }
  if (typeof value === "string") {
    return value;
  }
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return "--";
  }

  const fieldName = path[path.length - 1] ?? "";
  if (fieldName.includes("percent") || fieldName.includes("confidence") ||
    fieldName.includes("quality")) {
    return `${formatNumericStat(value, value === 0 || Math.abs(value) >= 10 ? 0 : 1)}%`;
  }
  if (fieldName.includes("time") || fieldName.includes("duration") ||
    fieldName.includes("seconds")) {
    return `${formatNumericStat(value, Math.abs(value) >= 10 ? 1 : 2)}s`;
  }
  if (Number.isInteger(value) || /(^|_)count$/.test(fieldName) ||
    fieldName.endsWith("_count") || ["score", "goals", "assists", "saves", "shots"]
      .includes(fieldName)) {
    return `${Math.round(value)}`;
  }
  if (Math.abs(value) >= 100) {
    return value.toFixed(0);
  }
  if (Math.abs(value) >= 10) {
    return value.toFixed(1);
  }
  return value.toFixed(2);
}

function formatNumericStat(value: number, digits: number): string {
  return value.toFixed(digits);
}

function collectScopedDescriptors(
  scope: StatScope,
  snapshot: PlayerStatsSnapshot | TeamStatsSnapshot | null,
): StatDescriptor[] {
  if (!snapshot) {
    return [];
  }

  const paths: string[][] = [];
  collectPrimitivePaths(snapshot, [], paths);
  return paths
    .filter((path) => scope !== "player" || !PLAYER_METADATA_KEYS.has(path[0] ?? ""))
    .map((path) => {
      const label = humanizePath(path);
      const selectorLabel = `${scope === "player" ? "Players" : "Teams"} / ${label}`;
      return {
        id: `${scope}:${path.join(".")}`,
        scope,
        path,
        label,
        selectorLabel,
        searchText: normalize(`${scope} ${selectorLabel} ${path.join(" ")}`),
      };
    });
}

function collectPrimitivePaths(
  value: unknown,
  path: string[],
  paths: string[][],
): void {
  if (isPrimitiveStatValue(value)) {
    if (path.length > 0) {
      paths.push(path);
    }
    return;
  }
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return;
  }

  for (const [key, child] of Object.entries(value)) {
    if (Array.isArray(child)) {
      continue;
    }
    collectPrimitivePaths(child, [...path, key], paths);
  }
}

function isPrimitiveStatValue(value: unknown): value is PrimitiveStatValue {
  return value === null ||
    typeof value === "boolean" ||
    typeof value === "number" ||
    typeof value === "string";
}

function getValueAtPath(source: unknown, path: readonly string[]): unknown {
  let value = source;
  for (const part of path) {
    if (!value || typeof value !== "object" || !(part in value)) {
      return undefined;
    }
    value = (value as Record<string, unknown>)[part];
  }
  return value;
}

function renderStatMonitorRow(
  name: string,
  isTeamZero: boolean,
  value: unknown,
  path: readonly string[],
): HTMLElement {
  const row = document.createElement("div");
  row.className = `stat-monitor-row ${isTeamZero ? "team-blue" : "team-orange"}`;

  const label = document.createElement("span");
  label.className = "stat-monitor-row-label";
  label.textContent = name;

  const displayValue = document.createElement("strong");
  displayValue.className = "stat-monitor-row-value";
  displayValue.textContent = formatStatMonitorValue(value, path);

  row.append(label, displayValue);
  return row;
}

function humanizePath(path: readonly string[]): string {
  return path.map((part) => part.split("_").map((word) =>
    word.length === 0
      ? word
      : word[0].toUpperCase() + word.slice(1)
  ).join(" ")).join(" / ");
}

function normalize(value: string): string {
  return value.toLowerCase().replace(/[_/.-]+/g, " ").replace(/\s+/g, " ")
    .trim();
}

function scoreFuzzyMatch(text: string, query: string): number | null {
  const tokens = query.split(" ").filter(Boolean);
  let total = 0;
  for (const token of tokens) {
    const index = text.indexOf(token);
    if (index >= 0) {
      total += index;
      continue;
    }

    const fuzzyScore = scoreFuzzyToken(text, token);
    if (fuzzyScore === null) {
      return null;
    }
    total += fuzzyScore + 80;
  }
  return total + text.length / 1000;
}

function scoreFuzzyToken(text: string, token: string): number | null {
  let position = 0;
  let firstMatch = -1;
  let spread = 0;
  for (const character of token) {
    const next = text.indexOf(character, position);
    if (next < 0) {
      return null;
    }
    if (firstMatch < 0) {
      firstMatch = next;
    }
    spread += next - position;
    position = next + 1;
  }
  return firstMatch + spread;
}
