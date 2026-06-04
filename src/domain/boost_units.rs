/// The maximum raw boost value stored in replay data.
///
/// Rocket League replays represent boost on a `0..=255` scale rather than a
/// `0..=100` percentage scale.
pub const BOOST_MAX_AMOUNT: f32 = u8::MAX as f32;

/// The raw replay boost amount players spawn with at each standard kickoff.
///
/// Rocket League starts each kickoff with one third of a full tank, which maps
/// cleanly to `85.0` on the replay's `0..=255` boost scale.
pub const BOOST_KICKOFF_START_AMOUNT: f32 = BOOST_MAX_AMOUNT / 3.0;

/// The rate at which boost drains while active, in raw replay units per second.
pub const BOOST_USED_RAW_UNITS_PER_SECOND: f32 = 80.0 / 0.93;

/// The rate at which boost drains while active, in percentage points per second.
pub const BOOST_USED_PERCENT_PER_SECOND: f32 =
    BOOST_USED_RAW_UNITS_PER_SECOND * 100.0 / BOOST_MAX_AMOUNT;

/// Converts a raw replay boost amount (`0..=255`) to a percentage (`0..=100`).
pub fn boost_amount_to_percent(boost_amount: f32) -> f32 {
    boost_amount * 100.0 / BOOST_MAX_AMOUNT
}

/// Converts a boost percentage (`0..=100`) to a raw replay boost amount (`0..=255`).
pub fn boost_percent_to_amount(boost_percent: f32) -> f32 {
    boost_percent * BOOST_MAX_AMOUNT / 100.0
}

#[deprecated(
    note = "BOOST_USED_PER_SECOND is measured in raw replay units. Use BOOST_USED_RAW_UNITS_PER_SECOND or BOOST_USED_PERCENT_PER_SECOND instead."
)]
pub const BOOST_USED_PER_SECOND: f32 = BOOST_USED_RAW_UNITS_PER_SECOND;
