import type { StatDefinition, StatScopeKind } from "./statRegistry.ts";
import { el } from "./reportDom.ts";
import { createPageIntro } from "./reportLayout.ts";
import type { PlayerStatsSnapshot, StatsFrame, TeamStatsSnapshot } from "./statsTimeline.ts";

type StatsTarget = PlayerStatsSnapshot | TeamStatsSnapshot;

function targetName(target: StatsTarget, scope: StatScopeKind, index: number): string {
  if (scope === "player") {
    return (target as PlayerStatsSnapshot).name || `Player ${index + 1}`;
  }
  return index === 0 ? "Blue" : "Orange";
}

function getTargets(frame: StatsFrame, scope: StatScopeKind): StatsTarget[] {
  return scope === "player" ? frame.players : [frame.team_zero, frame.team_one];
}

function groupDefinitions(definitions: StatDefinition[]): Map<string, StatDefinition[]> {
  const groups = new Map<string, StatDefinition[]>();
  for (const definition of definitions) {
    const key = `${definition.scope}:${definition.category}`;
    const group = groups.get(key);
    if (group) {
      group.push(definition);
    } else {
      groups.set(key, [definition]);
    }
  }
  return new Map([...groups].sort(([left], [right]) => left.localeCompare(right)));
}

function formatSectionTitle(key: string): string {
  const [scope, category] = key.split(":");
  const prettyCategory = (category ?? "")
    .replace(/_/g, " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
  return `${scope === "player" ? "Player" : "Team"} ${prettyCategory}`;
}

function sectionId(key: string): string {
  return `stats-${key.replace(/[^a-z0-9]+/gi, "-").toLowerCase()}`;
}

function formatStatLabel(definition: StatDefinition): string {
  return definition.path.slice(1).join(".") || definition.label;
}

function renderStatsTable(
  key: string,
  definitions: StatDefinition[],
  finalFrame: StatsFrame,
): HTMLElement {
  const scope = definitions[0]?.scope ?? "player";
  const targets = getTargets(finalFrame, scope);
  const section = el("section", {
    className: "stats-report-section",
    id: sectionId(key),
  });
  const header = el("header");
  header.append(
    el("h2", { text: formatSectionTitle(key) }),
    el("span", { text: `${definitions.length} stats` }),
  );

  const wrap = el("div", { className: "stats-report-table-wrap" });
  const table = el("table", { className: "stats-report-table" });
  const thead = el("thead");
  const headerRow = el("tr");
  headerRow.append(el("th", { text: "Statistic" }));
  targets.forEach((target, index) => {
    headerRow.append(el("th", { text: targetName(target, scope, index) }));
  });
  thead.append(headerRow);

  const tbody = el("tbody");
  definitions.forEach((definition) => {
    const row = el("tr");
    row.append(el("td", { text: formatStatLabel(definition) }));
    targets.forEach((target) => {
      row.append(el("td", { text: definition.format(definition.read(target)) }));
    });
    tbody.append(row);
  });

  table.append(thead, tbody);
  wrap.append(table);
  section.append(header, wrap);
  return section;
}

export function renderDumpPage(definitions: StatDefinition[], finalFrame: StatsFrame): HTMLElement {
  const grouped = groupDefinitions(definitions);
  const page = el("div", { className: "stats-report-page" });
  page.append(
    createPageIntro(
      "All stats dump",
      "Everything emitted by the current stats timeline, including experimental mechanic counters and low-level breakdowns.",
    ),
  );

  const nav = el("nav", { className: "stats-report-jump-nav" });
  for (const key of grouped.keys()) {
    const link = el("a", { text: formatSectionTitle(key) });
    link.setAttribute("href", `#${sectionId(key)}`);
    nav.append(link);
  }
  page.append(nav);

  const grid = el("div", { className: "stats-report-grid" });
  for (const [key, group] of grouped) {
    grid.append(renderStatsTable(key, group, finalFrame));
  }
  page.append(grid);
  return page;
}
