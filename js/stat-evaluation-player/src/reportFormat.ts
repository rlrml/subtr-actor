import { toBoostDisplayUnits } from "./boostFormatting.ts";

interface FieldPosition {
  x: number;
  y: number;
  z: number;
}

export function formatSeconds(value: number | null | undefined): string {
  return value == null || !Number.isFinite(value) ? "--" : `${Number(value.toFixed(1))}s`;
}

export function formatPercent(value: number | null | undefined): string {
  return value == null || !Number.isFinite(value) ? "--" : `${Number(value.toFixed(1))}%`;
}

export function formatShare(value: number, total: number): string {
  return total > 0 ? `${formatSeconds(value)} (${formatPercent((value / total) * 100)})` : "--";
}

export function formatFieldPosition(position: FieldPosition | null | undefined): string {
  if (!position) return "--";
  return `x ${Math.round(position.x)}, y ${Math.round(position.y)}, z ${Math.round(position.z)}`;
}

export function formatBoostAmount(raw: number | null | undefined): string {
  return raw == null || !Number.isFinite(raw)
    ? "--"
    : `${Number(toBoostDisplayUnits(raw).toFixed(0))}`;
}

export function formatTime(seconds: number | null | undefined): string {
  if (seconds == null || !Number.isFinite(seconds)) return "--";
  const clamped = Math.max(0, seconds);
  const minutes = Math.floor(clamped / 60);
  const remainingSeconds = clamped - minutes * 60;
  return `${minutes}:${remainingSeconds.toFixed(1).padStart(4, "0")}`;
}
