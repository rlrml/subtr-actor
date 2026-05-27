import type { BoostStats } from "./generated/BoostStats.ts";

const CONTINUOUS_BOOST_FIELDS = [
  "tracked_time",
  "boost_integral",
  "time_zero_boost",
  "time_hundred_boost",
  "time_boost_0_25",
  "time_boost_25_50",
  "time_boost_50_75",
  "time_boost_75_100",
] as const;

const LEDGER_BOOST_FIELDS = [
  "amount_collected",
  "amount_collected_inactive",
  "big_pads_collected_inactive",
  "small_pads_collected_inactive",
  "amount_stolen",
  "big_pads_collected",
  "small_pads_collected",
  "big_pads_stolen",
  "small_pads_stolen",
  "amount_collected_big",
  "amount_stolen_big",
  "amount_collected_small",
  "amount_stolen_small",
  "amount_respawned",
  "overfill_total",
  "overfill_from_stolen",
  "amount_used",
  "amount_used_while_grounded",
  "amount_used_while_airborne",
  "amount_used_while_supersonic",
] as const;

export const EVENT_DERIVED_BOOST_FIELDS = [
  ...CONTINUOUS_BOOST_FIELDS,
  ...LEDGER_BOOST_FIELDS,
] as const;

export type EventDerivedBoostField = (typeof EVENT_DERIVED_BOOST_FIELDS)[number];
export type EventDerivedBoostStats = Pick<BoostStats, EventDerivedBoostField> &
  Pick<BoostStats, "labeled_amounts" | "labeled_counts">;

export function createLedgerBoostStats(): EventDerivedBoostStats {
  return {
    tracked_time: 0,
    boost_integral: 0,
    time_zero_boost: 0,
    time_hundred_boost: 0,
    time_boost_0_25: 0,
    time_boost_25_50: 0,
    time_boost_50_75: 0,
    time_boost_75_100: 0,
    amount_collected: 0,
    amount_collected_inactive: 0,
    big_pads_collected_inactive: 0,
    small_pads_collected_inactive: 0,
    amount_stolen: 0,
    big_pads_collected: 0,
    small_pads_collected: 0,
    big_pads_stolen: 0,
    small_pads_stolen: 0,
    amount_collected_big: 0,
    amount_stolen_big: 0,
    amount_collected_small: 0,
    amount_stolen_small: 0,
    amount_respawned: 0,
    overfill_total: 0,
    overfill_from_stolen: 0,
    amount_used: 0,
    amount_used_while_grounded: 0,
    amount_used_while_airborne: 0,
    amount_used_while_supersonic: 0,
  };
}

export const EMPTY_LEDGER_BOOST_STATS = createLedgerBoostStats();
