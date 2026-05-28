export function formatInteger(value: number | undefined): string {
  return value === undefined || Number.isNaN(value) ? "?" : `${Math.round(value)}`;
}

export function formatNumber(value: number | undefined, digits = 1, suffix = ""): string {
  return value === undefined || Number.isNaN(value) ? "?" : `${value.toFixed(digits)}${suffix}`;
}

export function formatPercentage(value: number | undefined, digits = 0): string {
  return formatNumber(value, digits, "%");
}

export function formatTimeShare(
  value: number | undefined,
  percentage: number | undefined,
  timeDigits = 1,
  percentageDigits = 0,
): string {
  if (value === undefined || Number.isNaN(value)) {
    return formatPercentage(percentage, percentageDigits);
  }

  const timeDisplay = formatNumber(value, timeDigits, "s");
  if (percentage === undefined || Number.isNaN(percentage)) {
    return timeDisplay;
  }

  return `${timeDisplay} (${formatPercentage(percentage, percentageDigits)})`;
}

export function formatTimeShareFromTrackedTime(
  value: number | undefined,
  trackedTime: number | undefined,
  timeDigits = 1,
  percentageDigits = 0,
): string {
  const percentage =
    value !== undefined &&
    trackedTime !== undefined &&
    !Number.isNaN(value) &&
    !Number.isNaN(trackedTime) &&
    trackedTime > 0
      ? (value * 100) / trackedTime
      : undefined;

  return formatTimeShare(value, percentage, timeDigits, percentageDigits);
}

export function asNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

export function percentageFromUnit(value: unknown): number | undefined {
  const number = asNumber(value);
  return number === undefined ? undefined : number * 100;
}
