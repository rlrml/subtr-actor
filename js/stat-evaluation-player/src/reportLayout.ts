import { el } from "./reportDom.ts";
import type { PlayerStatsSnapshot } from "./statsTimeline.ts";

export function createSummaryCard(label: string, value: string, detail?: string): HTMLElement {
  const card = el("section", { className: "stats-report-summary-card" });
  card.append(el("span", { text: label }), el("strong", { text: value }));
  if (detail) {
    card.append(el("small", { text: detail }));
  }
  return card;
}

export function createPageIntro(title: string, text: string): HTMLElement {
  const intro = el("section", { className: "stats-report-page-intro" });
  intro.append(el("h2", { text: title }), el("p", { text }));
  return intro;
}

export function getLeader(
  players: PlayerStatsSnapshot[],
  read: (player: PlayerStatsSnapshot) => number,
  format: (value: number) => string,
): HTMLElement {
  const leader = [...players].sort((left, right) => read(right) - read(left))[0];
  const value = leader ? read(leader) : 0;
  return createSummaryCard(leader?.name ?? "--", format(value));
}
