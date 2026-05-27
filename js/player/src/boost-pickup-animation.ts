import type {
  ReplayPlayerPlugin,
  ReplayPlayerRenderContext,
} from "./types";
import { DEFAULT_BOOST_PICKUP_ANIMATION_DURATION_SECONDS } from "./boost-pickup-animation-constants";
import { buildBoostPickupAnimationEvents } from "./boost-pickup-animation-events";
import { updateBoostPickupAnimationEvent } from "./boost-pickup-animation-frame";
import {
  disposeBoostPickupAnimationEvent,
  syncBoostPickupCountTexture,
} from "./boost-pickup-animation-group";
import type {
  BoostPickupAnimationEvent,
  BoostPickupAnimationPluginOptions,
} from "./boost-pickup-animation-types";

export type {
  BoostPickupAnimationFilter,
  BoostPickupAnimationPickup,
  BoostPickupAnimationPluginOptions,
} from "./boost-pickup-animation-types";

export function createBoostPickupAnimationPlugin(
  options: BoostPickupAnimationPluginOptions = {},
): ReplayPlayerPlugin {
  const durationSeconds = Math.max(
    0.1,
    options.durationSeconds ?? DEFAULT_BOOST_PICKUP_ANIMATION_DURATION_SECONDS,
  );
  let events: BoostPickupAnimationEvent[] = [];

  function includeEvent(event: BoostPickupAnimationEvent): boolean {
    return (
      options.includePickup?.({
        pad: event.pad,
        event: event.event,
        player: event.player,
      }) ?? true
    );
  }

  function hideAll(): void {
    for (const event of events) {
      event.group.visible = false;
    }
  }

  return {
    id: "boost-pickup-animation",
    setup(context): void {
      events = buildBoostPickupAnimationEvents(context);
    },
    beforeRender(context: ReplayPlayerRenderContext): void {
      if (!context.state.boostPickupAnimationEnabled) {
        hideAll();
        return;
      }

      const startTime = context.currentTime - durationSeconds;
      const countsByPlayer = new Map<string, number>();
      for (const event of events) {
        if (event.time > context.currentTime) {
          event.group.visible = false;
          continue;
        }
        if (!includeEvent(event)) {
          event.group.visible = false;
          continue;
        }

        const pickupCount = (countsByPlayer.get(event.player.id) ?? 0) + 1;
        countsByPlayer.set(event.player.id, pickupCount);
        if (event.time < startTime) {
          event.group.visible = false;
          continue;
        }

        syncBoostPickupCountTexture(event, pickupCount);
        updateBoostPickupAnimationEvent(event, context.currentTime - event.time, durationSeconds);
      }
    },
    teardown(): void {
      for (const event of events) {
        disposeBoostPickupAnimationEvent(event);
      }
      events = [];
    },
  };
}
