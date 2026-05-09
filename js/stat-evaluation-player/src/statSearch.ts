import type { StatDefinition } from "./statRegistry.ts";

function normalizeStatSearchText(value: string): string {
  return value.toLowerCase().replace(/[_/.-]+/g, " ").replace(/\s+/g, " ")
    .trim();
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

export function scoreStatDefinitionSearchMatch(
  definition: StatDefinition,
  query: string,
): number | null {
  const tokens = normalizeStatSearchText(query).split(" ").filter(Boolean);
  if (tokens.length === 0) {
    return 0;
  }

  const searchText = normalizeStatSearchText([
    definition.scope,
    definition.category,
    definition.label,
    definition.id,
    ...definition.path,
  ].join(" "));

  let total = 0;
  for (const token of tokens) {
    const index = searchText.indexOf(token);
    if (index >= 0) {
      total += index;
      continue;
    }

    const fuzzyScore = scoreFuzzyToken(searchText, token);
    if (fuzzyScore === null) {
      return null;
    }
    total += fuzzyScore + 80;
  }

  return total + searchText.length / 1000;
}

export function getStatDefinitionSearchMatches(
  definitions: StatDefinition[],
  query: string,
): StatDefinition[] {
  return definitions
    .map((definition, index) => ({
      definition,
      index,
      score: scoreStatDefinitionSearchMatch(definition, query),
    }))
    .filter((match): match is {
      definition: StatDefinition;
      index: number;
      score: number;
    } => match.score !== null)
    .sort((a, b) => a.score - b.score || a.index - b.index)
    .map((match) => match.definition);
}
