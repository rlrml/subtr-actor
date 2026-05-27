import type { StatsTimeline } from "./statsTimeline.ts";

const MECHANIC_SHORT_LABELS: Record<string, string> = {
  air_dribble: "AD",
  ball_carry: "BC",
  ceiling_shot: "CS",
  double_tap: "DT",
  flick: "F",
  flip_reset: "FR",
  half_flip: "HF",
  half_volley: "HV",
  musty_flick: "M",
  one_timer: "OT",
  pass: "P",
  speed_flip: "SF",
  wall_aerial: "WA",
  wall_aerial_shot: "WS",
  wavedash: "WD",
};

const HIDDEN_MECHANIC_KINDS = new Set(["wavedash"]);

export function formatMechanicKind(kind: string): string {
  return kind
    .split(/[_-]+/)
    .filter((part) => part.length > 0)
    .map((part) => `${part.slice(0, 1).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

export function mechanicShortLabel(kind: string): string {
  return (
    MECHANIC_SHORT_LABELS[kind] ??
    (kind
      .split(/[_-]+/)
      .filter((part) => part.length > 0)
      .map((part) => part.slice(0, 1).toUpperCase())
      .join("")
      .slice(0, 3) ||
      "M")
  );
}

export function getMechanicKinds(statsTimeline: StatsTimeline | null): string[] {
  return [
    ...new Set(
      (statsTimeline?.events.mechanics ?? [])
        .filter((event) => isVisibleMechanicKind(event.kind))
        .map((event) => event.kind),
    ),
  ].sort((left, right) => formatMechanicKind(left).localeCompare(formatMechanicKind(right)));
}

export function isVisibleMechanicKind(kind: string): boolean {
  return !HIDDEN_MECHANIC_KINDS.has(kind);
}

export function mechanicKindToModuleId(kind: string): string {
  return kind.replaceAll("_", "-");
}
