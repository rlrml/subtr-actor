# Positioning Depth Proposal

## Current State

Today `PositioningReducer` treats these as binary or winner-take-all:

- `behind / in front of ball`: compare normalized player `y` to normalized ball `y`
- `most back / most forward`: give the whole sample to the single extreme teammate

That is simple, but it collapses two different situations into the same stat:

- barely ahead of the ball vs clearly cheating forward
- barely last back vs clearly isolated as last back

## Proposal

Keep the existing stats unchanged for compatibility.

Add a new additive stat family that measures **depth separation**, not just side/rank.

Recommended public name:

- `positioning.depth_layers`

Alternative names:

- `positioning.depth_margin`
- `positioning.role_separation`

`depth_layers` is the best fit because these are time buckets, not a single scalar margin.

## New Ball-Relative Stat

Add a five-bucket ball-depth stat based on attack-axis distance from the ball:

- `time_clearly_behind_ball`
- `time_slightly_behind_ball`
- `time_level_with_ball`
- `time_slightly_in_front_of_ball`
- `time_clearly_in_front_of_ball`

Computation:

```text
ball_depth_delta = normalized_player_y - normalized_ball_y
```

Bucket logic:

- `ball_depth_delta <= -clear_ball_depth_margin` => `clearly_behind`
- `-clear_ball_depth_margin < ball_depth_delta <= -level_ball_depth_margin` => `slightly_behind`
- `abs(ball_depth_delta) < level_ball_depth_margin` => `level_with_ball`
- `level_ball_depth_margin <= ball_depth_delta < clear_ball_depth_margin` => `slightly_in_front`
- `ball_depth_delta >= clear_ball_depth_margin` => `clearly_in_front`

Why this helps:

- preserves the current "ahead/behind" concept
- stops tiny jitter around the ball line from looking meaningful
- distinguishes normal support distance from an aggressive overcommit

## New Team-Relative Extreme Stat

Add a second stat for how separated the player is when they are the most-back or most-forward teammate.

Recommended field names:

- `time_essentially_level_most_back`
- `time_clear_most_back`
- `time_isolated_most_back`
- `time_essentially_level_most_forward`
- `time_clear_most_forward`
- `time_isolated_most_forward`

Computation:

- sort teammates by normalized attack-axis `y`
- for the most-back player, compute:

```text
most_back_gap = second_most_back_y - most_back_y
```

- for the most-forward player, compute:

```text
most_forward_gap = most_forward_y - second_most_forward_y
```

Bucket logic:

- gap `< level_extreme_gap` => `essentially_level`
- `level_extreme_gap <= gap < isolated_extreme_gap` => `clear`
- gap `>= isolated_extreme_gap` => `isolated`

Why this helps:

- separates "technically last back" from "clearly anchoring"
- separates "technically furthest up" from "fully stretched upfield"
- preserves current `most_back` / `most_forward` for parity and compatibility

This is the team-relative analog of `level_with_ball`:

- if the extreme player is only a little ahead/behind the next teammate, they are "essentially level"
- if the gap opens up, they become `clear`
- if the gap is large, they become `isolated`

## Defaults

These should be configurable on the reducer, but the defaults below are reasonable starting points:

```rust
pub struct PositioningReducerConfig {
    pub level_ball_depth_margin: f32,
    pub clear_ball_depth_margin: f32,
    pub level_extreme_gap: f32,
    pub isolated_extreme_gap: f32,
}

impl Default for PositioningReducerConfig {
    fn default() -> Self {
        Self {
            level_ball_depth_margin: 150.0,
            clear_ball_depth_margin: 600.0,
            level_extreme_gap: 250.0,
            isolated_extreme_gap: 900.0,
        }
    }
}
```

Interpretation of the defaults:

- `150uu`: effectively "same line" once replay jitter and car center-vs-front simplification are considered
- `600uu`: clearly different layer from the ball, but not so large that only extreme cases register
- `250uu`: teammate ordering exists but is still close enough to be effectively level
- `900uu`: teammate is meaningfully detached as anchor or outlet

## API Shape

Keep this additive and explicit.

Option A:

- add these fields directly onto `PositioningStats`

Option B:

- add a nested struct:

```rust
pub struct DepthLayerStats {
    pub time_clearly_behind_ball: f32,
    pub time_slightly_behind_ball: f32,
    pub time_level_with_ball: f32,
    pub time_slightly_in_front_of_ball: f32,
    pub time_clearly_in_front_of_ball: f32,
    pub time_essentially_level_most_back: f32,
    pub time_clear_most_back: f32,
    pub time_isolated_most_back: f32,
    pub time_essentially_level_most_forward: f32,
    pub time_clear_most_forward: f32,
    pub time_isolated_most_forward: f32,
}
```

I would prefer Option A if we want Ballchasing-style flat output, and Option B if we expect this to grow.

## Recommended Naming

Recommended external naming:

- family: `depth_layers`
- ball buckets: `clearly_behind`, `slightly_behind`, `level`, `slightly_in_front`, `clearly_in_front`
- team-extreme buckets: `essentially_level`, `clear`, `isolated`

Recommended serialized flat field names if kept in `PositioningStats`:

- `time_ball_depth_clearly_behind`
- `time_ball_depth_slightly_behind`
- `time_ball_depth_level`
- `time_ball_depth_slightly_in_front`
- `time_ball_depth_clearly_in_front`
- `time_most_back_essentially_level`
- `time_most_back_clear`
- `time_most_back_isolated`
- `time_most_forward_essentially_level`
- `time_most_forward_clear`
- `time_most_forward_isolated`

This reads more consistently than embedding adverbs in mixed positions.

## Non-Goals

This proposal does not replace:

- `time_behind_ball`
- `time_in_front_of_ball`
- `time_most_back`
- `time_most_forward`

Those remain useful coarse stats and remain better for Ballchasing comparisons.

## Suggested Rollout

1. Add `PositioningReducerConfig` with defaults.
2. Implement the ball-depth buckets first.
3. Implement the extreme-gap buckets second.
4. Expose percentages derived from `tracked_time` only if the raw time buckets look useful in practice.

If we want to stay minimal, the best first cut is:

- ball depth: `slightly_behind / level / slightly_in_front / clearly_in_front / clearly_behind`
- extremes: `essentially_level / clear / isolated`

That gives materially better definitions without changing the current stats contract.
