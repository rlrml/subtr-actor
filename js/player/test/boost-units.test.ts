import assert from "node:assert/strict";
import { test } from "node:test";

import { BOOST_RAW_MAX, boostAmountToPercent, boostPercentToAmount } from "../src/boost-units";

test("converts raw replay boost amounts to a 0-100 percentage", () => {
  assert.equal(boostAmountToPercent(BOOST_RAW_MAX), 100);
  assert.equal(boostAmountToPercent(0), 0);
  assert.equal(boostAmountToPercent(BOOST_RAW_MAX / 3), 100 / 3);
});

test("passes nullish boost amounts through unchanged", () => {
  assert.equal(boostAmountToPercent(null), null);
  assert.equal(boostAmountToPercent(undefined), null);
});

test("round-trips percentages back to raw boost amounts", () => {
  assert.equal(boostPercentToAmount(100), BOOST_RAW_MAX);
  assert.equal(boostPercentToAmount(0), 0);
  assert.equal(boostPercentToAmount(null), null);
});
