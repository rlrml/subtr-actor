import type { StatDefinition } from "./statRegistry.ts";

function normalizeStatSearchText(value: string): string {
  return value
    .toLowerCase()
    .replace(/[_/.-]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function getStatSearchTokens(query: string): string[] {
  return normalizeStatSearchText(query).split(" ").filter(Boolean);
}

export function scoreStatDefinitionSearchMatch(
  definition: StatDefinition,
  query: string,
): number | null {
  const tokens = getStatSearchTokens(query);
  if (tokens.length === 0) {
    return 0;
  }

  const searchText = normalizeStatSearchText(
    [
      definition.scope,
      definition.category,
      definition.label,
      definition.id,
      ...definition.path,
    ].join(" "),
  );

  let total = 0;
  for (const token of tokens) {
    const index = searchText.indexOf(token);
    if (index < 0) {
      return null;
    }
    total += index;
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
    .filter(
      (
        match,
      ): match is {
        definition: StatDefinition;
        index: number;
        score: number;
      } => match.score !== null,
    )
    .sort((a, b) => a.score - b.score || a.index - b.index)
    .map((match) => match.definition);
}
