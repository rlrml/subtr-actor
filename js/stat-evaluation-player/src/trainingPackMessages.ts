/**
 * Status line for a completed training-pack capture, with the
 * momentum-loss warning (see `momentumLossWarning` in `@rlrml/player`)
 * appended when the shooter's velocity is meaningfully unrepresentable as
 * spawn momentum — the same diagnostic the BakkesMod plugin surfaces at
 * capture time.
 *
 * Kept in its own module (rather than `trainingPackWindow.ts`) so node
 * tests can exercise it without importing the DOM-coupled window
 * controller.
 */
export function captureStatusMessage(
  shotNumber: number,
  shooterName: string,
  formattedTime: string,
  momentumWarning: string | null,
): string {
  const base = `Captured shot ${shotNumber} (${shooterName} at ${formattedTime})`;
  return momentumWarning === null ? `${base}.` : `${base}; warning: ${momentumWarning}.`;
}
