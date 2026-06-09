export const BLUE_TIMELINE_COLOR = "#3b82f6";
export const ORANGE_TIMELINE_COLOR = "#f59e0b";

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
const VISIBLE_MECHANIC_KINDS = new Set([
  "air_dribble",
  "ball_carry",
  "ceiling_shot",
  "center",
  "double_tap",
  "flick",
  "flip_reset",
  "half_flip",
  "half_volley",
  "musty_flick",
  "one_timer",
  "pass",
  "speed_flip",
  "wall_aerial",
  "wall_aerial_shot",
  "wavedash",
]);

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

export function isVisibleMechanicKind(kind: string): boolean {
  return VISIBLE_MECHANIC_KINDS.has(kind) && !HIDDEN_MECHANIC_KINDS.has(kind);
}

export function teamTimelineColor(isTeamZero: boolean | null | undefined): string | null {
  if (isTeamZero === true) {
    return BLUE_TIMELINE_COLOR;
  }
  if (isTeamZero === false) {
    return ORANGE_TIMELINE_COLOR;
  }

  return null;
}
