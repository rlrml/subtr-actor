import type { BoostStateEvent } from "./generated/BoostStateEvent.ts";
import { addF32, divF32, f32, mulF32, subF32 } from "./boostLedgerFloat.ts";
import type { EventDerivedBoostStats } from "./boostLedgerStats.ts";

const BOOST_MAX_AMOUNT = 255;
const BOOST_ZERO_BAND_RAW = 1;
const BOOST_FULL_BAND_MIN_RAW = BOOST_MAX_AMOUNT - 1;
const F32_EPSILON = 1.1920928955078125e-7;

interface ContinuousBoostAccumulator {
  stats: EventDerivedBoostStats;
  currentBoostAmount: number | null;
  currentBoostBefore: number | null;
  currentBoostFrame: number | null;
  previousBoostAmount: number | null;
}

export function applyBoostStateEvent(
  accumulator: ContinuousBoostAccumulator,
  event: BoostStateEvent,
): void {
  accumulator.currentBoostAmount = f32(event.boost_amount);
  accumulator.currentBoostBefore = event.boost_before == null ? null : f32(event.boost_before);
  accumulator.currentBoostFrame = event.frame;
}

export function addContinuousBoostSample(
  stats: EventDerivedBoostStats,
  previousBoostAmount: number,
  boostAmount: number,
  dt: number,
): void {
  const previous = f32(previousBoostAmount);
  const current = f32(boostAmount);
  const sampleDt = f32(dt);
  const averageBoostAmount = mulF32(addF32(previous, current), 0.5);

  stats.tracked_time = addF32(stats.tracked_time, sampleDt);
  stats.boost_integral = addF32(stats.boost_integral, mulF32(averageBoostAmount, sampleDt));
  stats.time_zero_boost = addF32(
    stats.time_zero_boost,
    mulF32(sampleDt, intervalFractionInBoostRange(previous, current, 0, BOOST_ZERO_BAND_RAW)),
  );
  stats.time_hundred_boost = addF32(
    stats.time_hundred_boost,
    mulF32(
      sampleDt,
      intervalFractionInBoostRange(
        previous,
        current,
        BOOST_FULL_BAND_MIN_RAW,
        BOOST_MAX_AMOUNT + 1,
      ),
    ),
  );
  stats.time_boost_0_25 = addF32(
    stats.time_boost_0_25,
    mulF32(sampleDt, intervalFractionInBoostRange(previous, current, 0, boostPercentToAmount(25))),
  );
  stats.time_boost_25_50 = addF32(
    stats.time_boost_25_50,
    mulF32(
      sampleDt,
      intervalFractionInBoostRange(
        previous,
        current,
        boostPercentToAmount(25),
        boostPercentToAmount(50),
      ),
    ),
  );
  stats.time_boost_50_75 = addF32(
    stats.time_boost_50_75,
    mulF32(
      sampleDt,
      intervalFractionInBoostRange(
        previous,
        current,
        boostPercentToAmount(50),
        boostPercentToAmount(75),
      ),
    ),
  );
  stats.time_boost_75_100 = addF32(
    stats.time_boost_75_100,
    mulF32(
      sampleDt,
      intervalFractionInBoostRange(
        previous,
        current,
        boostPercentToAmount(75),
        BOOST_MAX_AMOUNT + 1,
      ),
    ),
  );
}

export function applyContinuousBoostSample(
  accumulator: ContinuousBoostAccumulator,
  dt: number,
  frameNumber: number,
): [number, number] | null {
  if (accumulator.currentBoostFrame !== frameNumber) {
    return null;
  }
  const boostAmount = accumulator.currentBoostAmount;
  if (boostAmount == null) {
    return null;
  }
  const previousBoostAmount = accumulator.currentBoostBefore ?? boostAmount;
  addContinuousBoostSample(accumulator.stats, previousBoostAmount, boostAmount, dt);
  accumulator.previousBoostAmount = boostAmount;
  return [previousBoostAmount, boostAmount];
}

function boostPercentToAmount(boostPercent: number): number {
  return divF32(mulF32(boostPercent, BOOST_MAX_AMOUNT), 100);
}

function intervalFractionInBoostRange(
  startBoost: number,
  endBoost: number,
  minBoost: number,
  maxBoost: number,
): number {
  const boostDelta = subF32(endBoost, startBoost);
  if (Math.abs(boostDelta) <= F32_EPSILON) {
    return startBoost >= minBoost && startBoost < maxBoost ? 1 : 0;
  }

  const tAtMin = divF32(subF32(minBoost, startBoost), boostDelta);
  const tAtMax = divF32(subF32(maxBoost, startBoost), boostDelta);
  const intervalStart = Math.max(Math.min(tAtMin, tAtMax), 0);
  const intervalEnd = Math.min(Math.max(tAtMin, tAtMax), 1);
  return Math.max(subF32(intervalEnd, intervalStart), 0);
}
