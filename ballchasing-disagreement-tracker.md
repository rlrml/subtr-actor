# Ballchasing Disagreement Tracker

## Purpose

This document tracks where `subtr-actor` currently disagrees with Ballchasing stats, what implementation choices are relevant, and what the likely causes are.

This is not a proof that Ballchasing is correct. It is a debugging map:

- some disagreements may be bugs in `subtr-actor`
- some may be Ballchasing heuristics or omissions
- some may be pure semantic mismatches

## Evidence Base

Current notes are based on:

- the shared comparison code in [`src/ballchasing.rs`](src/ballchasing.rs)
- recent live comparisons run with [`src/bin/ballchasing_compare.rs`](src/bin/ballchasing_compare.rs)
- a small sample of 4 recent ranked-doubles replays from Ballchasing

Sample replay ids used for the qualitative mismatch patterns below:

- `32741e71-2410-4d77-a9b1-e9d386ed315e`
- `75c7e699-e881-475a-a905-0e9c044c4889`
- `61fa97e6-d817-4bce-8843-8e1b53eefa67`
- `32b623fa-e7ff-452f-a6ab-179482b44cbd`

Mismatch frequency counts below are counts of mismatching comparison targets across those sampled scopes, not replay-level pass/fail counts.

## Thresholds And Heuristics In Our Implementation

These are the important places where we are making an explicit semantic choice.

| Area | Current `subtr-actor` behavior | Code | Do we know Ballchasing does the same? |
| --- | --- | --- | --- |
| Live-play gating | Exclude kickoff countdown and goal-scored replay states from time-based stats | [`StatsSample::is_live_play`](src/stats.rs) | No |
| Boost pickup gating | Ignore non-live `PickedUp` events by default; configurable opt-in exists | [`BoostReducerConfig`](src/stats.rs) | No |
| Boost scale | Comparison normalizes raw replay boost `0..255` into Ballchasing-style `0..100` units | [`src/ballchasing.rs`](src/ballchasing.rs) | Ballchasing appears to use `0..100`-style reporting |
| Small pad amount | `12%` of full boost | [`SMALL_PAD_AMOUNT_RAW`](src/stats.rs) | Probably yes |
| Pad size inference | Infer size from observed respawn cadence using `>= 7s` => big | [`BoostReducer`](src/stats.rs) | Unknown |
| Canonical pad matching radius | Match standard soccar pads within `400uu` | [`STANDARD_PAD_MATCH_RADIUS`](src/stats.rs) | Unknown |
| Midfield stolen/neutral tolerance | Treat near-midfield fallback pads as neutral with `128uu` tolerance on `y` | [`BOOST_PAD_MIDFIELD_TOLERANCE_Y`](src/stats.rs) | Unknown |
| Supersonic threshold | `2200uu/s` | [`SUPERSONIC_SPEED_THRESHOLD`](src/stats.rs) | Probably, but not confirmed |
| Boost-speed threshold | `1410uu/s` | [`BOOST_SPEED_THRESHOLD`](src/stats.rs) | Probably, but not confirmed |
| Ground threshold | `z <= 20` | [`GROUND_Z_THRESHOLD`](src/stats.rs) | Unknown |
| High-air threshold | `z >= 300` | [`HIGH_AIR_Z_THRESHOLD`](src/stats.rs) | Unknown |
| Field halves | Normalize to team attack direction; `y < 0` is defensive half | [`PositioningReducer`](src/stats.rs) | Probably similar, not confirmed |
| Field zones | Normalize to team attack direction; split using `FIELD_ZONE_BOUNDARY_Y = BOOST_PAD_SIDE_LANE_Y` | [`PositioningReducer`](src/stats.rs) | Unknown |
| Behind / in front of ball | Compare normalized player `y` to normalized ball `y` | [`PositioningReducer`](src/stats.rs) | Unknown |
| Closest / farthest / most back / most forward | Pure per-sample ranking among teammates | [`PositioningReducer`](src/stats.rs) | Unknown |
| Possession owner | Use team last-touch signal, not replay-native player possession | [`PositioningReducer`](src/stats.rs), [`PossessionReducer`](src/stats.rs) | No |
| Powerslide count | Count rising edges of `bReplicatedHandbrake` | [`PowerslideReducer`](src/stats.rs) | Unknown |

## Directional Patterns Seen So Far

### Boost pickup direction

Across the sampled replays:

- `count_collected_small` is consistently higher on our side
- `amount_collected_small` is consistently higher on our side
- total `amount_collected` is usually somewhat higher on our side
- big-pad counts and big-pad collected amounts are mixed, not one-directional

This suggests the current disagreement is not just scale conversion. The strongest visible bias is extra small-pad pickups or Ballchasing suppressing some small-pad pickups that we count.

### Movement direction

Across the sampled replays:

- `time_high_air` is usually much higher on our side
- `time_low_air` is usually lower on our side
- `time_ground` is usually a bit lower on our side
- `time_powerslide` and `count_powerslide` are usually much higher on our side
- `total_distance` is usually higher on our side

This is exactly the shape you would expect if our vertical and handbrake semantics do not match Ballchasing's.

### Positioning direction

Across the sampled replays:

- Ballchasing's `time_neutral_third` is often much higher than our internal neutral-zone time
- Ballchasing's `time_defensive_third` and `time_offensive_third` are often lower than our internal zone totals
- the possession-conditioned distance stats disagree in a way that looks heuristic rather than threshold-driven

This strongly suggests semantic mismatch in zone boundaries and possession labeling.

## Stats With No Strong Evidence Of Disagreement Yet

In the sampled runs, there was no major systematic evidence yet against:

- core scoreboard stats: `score`, `goals`, `assists`, `saves`, `shots`
- `shooting_percentage` after current comparison tolerance
- demo counts as a major current concern

These should still be checked again after the larger movement / boost issues are reduced.

## Non-Stat Comparison Problems

### Player name matching

Observed issue:

- some comparisons fail with `missing actual player`

Likely causes:

- Ballchasing normalizes or truncates names differently
- punctuation / Unicode / platform-specific formatting differs
- our comparison is exact-string matching on player names inside a team

Impact:

- this can hide stat agreement for those players
- it is a comparison-layer problem, not necessarily a reducer problem

## Disagreement Tracker By Stat Family

### Boost

| Stat key | Mismatch frequency in sample | Current direction / pattern | Threshold or heuristic involved | Ballchasing semantics known? | Likely causes / notes |
| --- | ---: | --- | --- | --- | --- |
| `boost.amount_collected` | 21 | Usually somewhat high on our side | Exact pad events, pad-size inference, live-play pickup inclusion | Partially | Most likely driven by small-pad overcount or different pickup suppression rules. Non-live pickup gating did not explain the sampled replay gap by itself. |
| `boost.amount_stolen` | 20 | Mixed, often somewhat high | Canonical pad side classification, midfield tolerance | No | Could be pad-side attribution mismatch, especially for fallback pad-position estimation or Ballchasing side logic near midfield. |
| `boost.amount_collected_big` | 19 | Mixed | Pad-size inference from respawn cadence | No | Mixed direction suggests size or pad identity resolution differences, not a simple scale problem. |
| `boost.amount_stolen_big` | 14 | Mixed | Pad-size inference and side classification | No | Same as above, plus enemy-side classification on matched vs fallback pad positions. |
| `boost.amount_collected_small` | 21 | Consistently high on our side | Pad-size inference, pickup dedupe | No | Strongest current signal. Either we are overcounting small-pad pickups or Ballchasing intentionally ignores some of them. |
| `boost.amount_stolen_small` | 19 | Usually high on our side | Pad-size inference, side classification | No | Likely downstream of the extra small-pad pickup pattern. |
| `boost.count_collected_big` | 18 | Mixed | Pad-size inference | No | Not the dominant directional issue. |
| `boost.count_stolen_big` | 6 | Mixed | Pad-size inference, side classification | No | Same note as above. |
| `boost.count_collected_small` | 21 | Consistently high on our side | Pickup dedupe, pad-size inference | No | Best candidate stat family for raw-event debugging. |
| `boost.count_stolen_small` | 18 | Usually high on our side | Pickup dedupe, side classification | No | Probably follows the same root cause as `count_collected_small`. |
| `boost.amount_overfill` | 20 | Usually somewhat high on our side | Pre-pickup boost amount, pad gain, pickup counting | No | Likely inherits any collection overcount or pre-pickup amount timing mismatch. |
| `boost.amount_overfill_stolen` | 12 | Mixed | Same as overfill plus stolen classification | No | Secondary effect of pickup / side disagreement. |
| `boost.amount_used_while_supersonic` | 21 | Usually high on our side | `boost_active && speed >= 2200` | No | Ballchasing may gate usage differently, use different supersonic semantics, or require actual boost consumption rather than active-boost state. |
| `boost.bpm` | 20 | Often low-to-slightly-low after comparison normalization fix | Team aggregation and pickup amounts | Partially | Team aggregation was fixed in comparison; remaining gap likely comes from real collection-count disagreement and/or tracked-time semantics. |
| `boost.avg_amount` | 19 | Often low on our side after normalization fix | Live-play tracked time and boost integral | No | May mean Ballchasing includes some non-live windows we exclude, or uses different boost interpolation / smoothing. |
| `boost.time_zero_boost` | 21 | Usually high on our side | Live-play gating, exact zero threshold | No | Could be Ballchasing smoothing/interpolating boost across frames or using a nonzero epsilon instead of strict `<= 0`. |
| `boost.percent_zero_boost` | 13 | Usually high on our side | Same as above, plus denominator choice | No | Same root causes as `time_zero_boost`; percent also depends on tracked-time semantics. |
| `boost.time_full_boost` | 21 | Usually low on our side | Live-play gating, strict `>= 255` threshold | No | Ballchasing may use a near-full threshold rather than exact max, or interpolate differently. |
| `boost.percent_full_boost` | 12 | Usually low on our side | Same as above, plus denominator choice | No | Same root causes as `time_full_boost`. |
| `boost.time_boost_0_25` | 21 | Often high | Bucket boundaries at `<25` | No | Likely affected by boost interpolation differences, especially if Ballchasing smooths between sparse updates. |
| `boost.time_boost_25_50` | 18 | Mixed | Bucket boundaries | No | Distribution mismatch rather than one-directional offset. |
| `boost.time_boost_50_75` | 21 | Mixed, often low | Bucket boundaries | No | Same bucket-distribution issue. |
| `boost.time_boost_75_100` | 21 | Mixed | Bucket boundaries | No | Same bucket-distribution issue. |
| `boost.percent_boost_0_25` | 12 | Often high | Bucket boundaries plus denominator | No | Same as `time_boost_0_25`. |
| `boost.percent_boost_25_50` | 11 | Mixed | Bucket boundaries plus denominator | No | Same as `time_boost_25_50`. |
| `boost.percent_boost_50_75` | 10 | Mixed | Bucket boundaries plus denominator | No | Same as `time_boost_50_75`. |
| `boost.percent_boost_75_100` | 11 | Mixed | Bucket boundaries plus denominator | No | Same as `time_boost_75_100`. |

### Movement

| Stat key | Mismatch frequency in sample | Current direction / pattern | Threshold or heuristic involved | Ballchasing semantics known? | Likely causes / notes |
| --- | ---: | --- | --- | --- | --- |
| `movement.total_distance` | 21 | Usually high on our side | Frame-to-frame position deltas during live play | No | Could be interpolation differences, teleport / spawn handling, or Ballchasing smoothing / dead-time suppression. |
| `movement.avg_speed` | 13 | Often low-to-mixed | Speed integral / tracked time | No | If tracked-time semantics differ from Ballchasing, average speed will shift even when distance is close. |
| `movement.avg_speed_percentage` | 13 | Often low-to-mixed | Normalize by `2300uu/s` | Probably | Likely derivative of `avg_speed`; less likely a Ballchasing threshold issue by itself. |
| `movement.time_slow_speed` | 5 | Slightly mixed | `speed < 1410` | No | Could move if Ballchasing uses a slightly different boost-speed boundary or interpolation. |
| `movement.time_boost_speed` | not prominent in sample top counts | Not yet enough signal | `1410 <= speed < 2200` | No | Still needs tracking after other movement mismatches shrink. |
| `movement.time_supersonic_speed` | not prominent in sample top counts | Not yet enough signal | `speed >= 2200` | Probably | Same note as above. |
| `movement.time_ground` | 20 | Usually somewhat low on our side | `z <= 20` | No | Strong sign that Ballchasing's ground threshold / grounded semantics differ. |
| `movement.time_low_air` | 21 | Usually low on our side | `20 < z < 300` | No | Very likely because our `high_air` threshold is too permissive compared with Ballchasing. |
| `movement.time_high_air` | 21 | Usually much higher on our side | `z >= 300` | No | Best candidate threshold mismatch in movement. Ballchasing may use a meaningfully higher high-air threshold. |
| `movement.percent_ground` | 3 | Slightly low | Same as `time_ground` | No | Same root cause, but showed up less frequently in this small sample. |
| `movement.percent_low_air` | 13 | Usually low | Same as `time_low_air` | No | Same root cause. |
| `movement.percent_high_air` | 13 | Usually high | Same as `time_high_air` | No | Same root cause. |
| `movement.time_powerslide` | 18 | Usually much higher on our side | Handbrake-active duration | No | Ballchasing may require wheel-ground contact, minimum duration, or a more restricted powerslide signal than `bReplicatedHandbrake`. |
| `movement.count_powerslide` | 20 | Usually much higher on our side | Rising edges of `bReplicatedHandbrake` | No | Strong sign that our powerslide event semantics are broader than Ballchasing's. |
| `movement.avg_powerslide_duration` | 7 | Usually higher | Duration divided by rising-edge count | No | Downstream of the powerslide-definition mismatch. |

### Positioning

| Stat key | Mismatch frequency in sample | Current direction / pattern | Threshold or heuristic involved | Ballchasing semantics known? | Likely causes / notes |
| --- | ---: | --- | --- | --- | --- |
| `positioning.avg_distance_to_ball` | 5 | Moderate mismatch | Per-sample ball distance over live play | No | Could be sample timing differences, ball/player interpolation, or live-play denominator mismatch. |
| `positioning.avg_distance_to_ball_possession` | 5 | Moderate mismatch | Team possession owner heuristic from last touch | No | Very likely possession semantics differ; Ballchasing may use player-level possession rather than team last-touch. |
| `positioning.avg_distance_to_ball_no_possession` | 5 | Moderate mismatch | Same as above | No | Same note. |
| `positioning.avg_distance_to_mates` | 2 | Small sample of disagreement | Per-sample teammate distance | No | Lower priority; likely derivative of sampling / interpolation differences. |
| `positioning.time_defensive_third` | 13 | Usually low on our side | Normalized `y < -FIELD_ZONE_BOUNDARY_Y` | No | Suggests Ballchasing uses different zone geometry or a different live-play window. |
| `positioning.time_neutral_third` | 13 | Usually high on our side | Center band between our zone thresholds | No | Strong sign that our neutral zone is narrower than Ballchasing's or that Ballchasing uses a different field reference. |
| `positioning.time_offensive_third` | 13 | Usually low on our side | Normalized `y > FIELD_ZONE_BOUNDARY_Y` | No | Same as defensive / neutral zones. |
| `positioning.percent_defensive_third` | 13 | Usually low | Same as above plus denominator | No | Same root cause. |
| `positioning.percent_neutral_third` | 13 | Usually high | Same as above plus denominator | No | Same root cause. |
| `positioning.percent_offensive_third` | 13 | Usually low | Same as above plus denominator | No | Same root cause. |
| `positioning.time_defensive_half` | 13 | Moderate mismatch | Normalized `y < 0` | Probably | Less likely a raw threshold bug than thirds; could be live-play or side-normalization semantics. |
| `positioning.time_offensive_half` | 11 | Moderate mismatch | Normalized `y >= 0` | Probably | Same note. |
| `positioning.percent_defensive_half` | 12 | Moderate mismatch | Same as above plus denominator | Probably | Same note. |
| `positioning.percent_offensive_half` | 12 | Moderate mismatch | Same as above plus denominator | Probably | Same note. |
| `positioning.time_behind_ball` | 13 | Often low on our side | Compare normalized player `y` vs ball `y` | No | Ballchasing may use a ball-radius offset, car-front rather than center, or different normalization around possessions. |
| `positioning.time_infront_ball` | 11 | Moderate mismatch | Same as above | No | Same root cause as `time_behind_ball`. |
| `positioning.percent_behind_ball` | 8 | Moderate mismatch | Same as above plus denominator | No | Same root cause. |
| `positioning.percent_infront_ball` | 8 | Moderate mismatch | Same as above plus denominator | No | Same root cause. |
| `positioning.time_most_back` | 13 | Moderate mismatch | Per-sample team rank by normalized `y` | No | Could differ if Ballchasing requires all teammates to be live/grounded or uses smoothing / dead-time suppression. |
| `positioning.time_most_forward` | 13 | Moderate mismatch | Same as above | No | Same note. |
| `positioning.percent_most_back` | 5 | Moderate mismatch | Same as above plus denominator | No | Same root cause. |
| `positioning.percent_most_forward` | 6 | Moderate mismatch | Same as above plus denominator | No | Same root cause. |
| `positioning.time_closest_to_ball` | 13 | Moderate mismatch | Per-sample team rank by ball distance | No | Could differ with interpolation timing or missing-player treatment. |
| `positioning.time_farthest_from_ball` | 13 | Moderate mismatch | Same as above | No | Same note. |
| `positioning.percent_closest_to_ball` | 8 | Moderate mismatch | Same as above plus denominator | No | Same root cause. |
| `positioning.percent_farthest_from_ball` | 7 | Moderate mismatch | Same as above plus denominator | No | Same root cause. |

## Working Hypotheses Worth Testing Next

### Highest priority

- Small-pad pickups are probably the biggest remaining boost disagreement.
  - Inspect raw `BoostPadEvent` sequences per pad id on one replay with modest deltas.
  - Check whether the same pad id is producing duplicate `PickedUp` events.
  - Check whether inferred small pads later resolve as big pads or vice versa.

- Powerslide semantics are probably too permissive.
  - Compare raw `bReplicatedHandbrake` traces with Ballchasing's reported powerslide counts on a short replay segment.
  - Test whether requiring ground contact meaningfully reduces the gap.

- High-air threshold is probably too low.
  - Try a comparison experiment with a larger high-air threshold and see whether `time_high_air` / `time_low_air` move toward Ballchasing.

### Medium priority

- Possession-conditioned positioning is likely using a weaker semantic than Ballchasing.
  - Ballchasing may use player possession rather than team last-touch.

- Thirds geometry may differ.
  - Ballchasing may not split the field at exactly `5120 * 2 / 3`.
  - There may be a different reference frame or a ball-relative / play-relative heuristic.

- Team boost tracked-time semantics may still differ.
  - Ballchasing may include or exclude windows differently around kickoff, goals, or other replay states.

## Recommendations For Future Updates To This Document

Whenever a disagreement changes materially:

- update the directional note
- note whether the change came from comparison normalization or reducer logic
- add at least one concrete replay id where the issue was confirmed
- if a threshold was changed, record the old and new values here
