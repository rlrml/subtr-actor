import * as THREE from "three";
import {
  BOOST_PICKUP_TEAM_ONE_COLOR,
  BOOST_PICKUP_TEAM_ZERO_COLOR,
} from "./boost-pickup-animation-constants";
import { createBoostPickupGroup } from "./boost-pickup-animation-group";
import type {
  BoostPickupAnimationContext,
  BoostPickupAnimationEvent,
} from "./boost-pickup-animation-types";
import type { ReplayBoostPad, ReplayBoostPadEvent, ReplayPlayerTrack } from "./types";

function teamColor(isTeamZero: boolean): string {
  return isTeamZero ? BOOST_PICKUP_TEAM_ZERO_COLOR : BOOST_PICKUP_TEAM_ONE_COLOR;
}

function padPickupEvents(pad: ReplayBoostPad): ReplayBoostPadEvent[] {
  return pad.events.filter((event) => !event.available && event.playerId);
}

function sortedPickupEvents(boostPads: ReplayBoostPad[]): Array<{
  pad: ReplayBoostPad;
  event: ReplayBoostPadEvent;
}> {
  const rawEvents = boostPads.flatMap((pad) => padPickupEvents(pad).map((event) => ({ pad, event })));
  rawEvents.sort((left, right) => {
    if (left.event.time !== right.event.time) {
      return left.event.time - right.event.time;
    }
    if (left.event.frame !== right.event.frame) {
      return left.event.frame - right.event.frame;
    }
    return left.pad.index - right.pad.index;
  });
  return rawEvents;
}

function playersById(players: ReplayPlayerTrack[]): Map<string, ReplayPlayerTrack> {
  return new Map(players.map((player) => [player.id, player]));
}

export function buildBoostPickupAnimationEvents(
  context: BoostPickupAnimationContext,
): BoostPickupAnimationEvent[] {
  const playerLookup = playersById(context.replay.players);
  const animationEvents: BoostPickupAnimationEvent[] = [];

  for (const { pad, event } of sortedPickupEvents(context.replay.boostPads)) {
    if (!event.playerId) {
      continue;
    }
    const player = playerLookup.get(event.playerId);
    if (!player) {
      continue;
    }

    const color = teamColor(player.isTeamZero);
    const { group, textMaterial, ringMaterial } = createBoostPickupGroup(color);
    group.position.copy(pad.position);
    context.scene.replayRoot.add(group);
    animationEvents.push({
      time: event.time,
      pad,
      event,
      player,
      color,
      currentCount: 1,
      position: new THREE.Vector3(pad.position.x, pad.position.y, pad.position.z),
      size: pad.size,
      group,
      textMaterial,
      ringMaterial,
    });
  }

  return animationEvents;
}
