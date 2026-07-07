import test from "node:test";
import assert from "node:assert/strict";

import { momentumLossWarning } from "@rlrml/player";
import { captureStatusMessage } from "./trainingPackMessages.ts";

test("capture status without a momentum warning", () => {
  assert.equal(
    captureStatusMessage(2, "calemacar", "3:46", null),
    "Captured shot 2 (calemacar at 3:46).",
  );
});

test("capture status appends the momentum-loss warning with the plugin's wording", () => {
  // Identity rotation faces +X; a fast sideways drift loses everything to
  // the scalar spawn-momentum encoding.
  const warning = momentumLossWarning({
    position: { x: 0, y: 0, z: 17 },
    linearVelocity: { x: 0, y: 900, z: 0 },
  });
  assert.equal(
    captureStatusMessage(3, "zen", "0:12", warning),
    "Captured shot 3 (zen at 0:12); warning: car moving 900 uu/s at 90\u{b0} off facing; " +
      "only 0 uu/s representable as spawn momentum.",
  );
});
