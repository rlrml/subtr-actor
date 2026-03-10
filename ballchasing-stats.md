# Ballchasing Stats Implementation Matrix

## Design Direction

- Prefer one outer stats collector that implements `Collector` and owns sampling policy.
- Compute stats with small composable reducers rather than one monolithic accumulator.
- Separate reducer inputs into:
  - Replay/header metadata
  - Discrete events
  - Time-integrated samples with explicit `dt`
- Do not make fixed-FPS sampling the canonical calculation path.
  - Exact stats should use replay frame deltas and event boundaries when possible.
  - Resampling should remain an optional collection policy for approximate or ML-oriented consumers.

## Status Key

- `Available`: already exposed well enough to build reducers now
- `Partial`: some source data exists, but reducer or normalization work is still needed
- `Missing`: processor support is not exposed yet or not identified yet

## Header / Replay Metadata

| Stat(s) | Source | Computation plan | Sampling | Processor status |
| --- | --- | --- | --- | --- |
| Score, Goals, Assists, Saves, Shots | Replay headers / per-player stats | Read from `ReplayMeta` and normalize into team/player totals | None | `Available` |
| Camera settings: FOV, Height, Pitch, Distance, Stiffness, Swivel speed, Transition speed | Replay headers / existing camera settings data | Read once per player; expose as settings payload, not time-series stats | None | `Available` |
| Steering sensitivity | Replay headers / existing player settings | Read once per player | None | `Available` |

## Existing Event-Like Signals

| Stat(s) | Source | Computation plan | Sampling | Processor status |
| --- | --- | --- | --- | --- |
| Demos inflicted by team / player | `ReplayProcessor.demolishes`, `get_active_demos` | Count attacker occurrences by player and by team | None | `Available` |
| Demos taken by player | `ReplayProcessor.demolishes`, `get_active_demos` | Count victim occurrences by player | None | `Available` |
| Kills, Deaths on timeline | Same demolish events | Emit timeline events from demolish timestamps | None | `Available` |
| Ball has been hit | `get_ball_has_been_hit` | Useful kickoff-phase signal, but not enough for possession/touches by itself | None | `Available` |

## Existing Time-Integrated Signals

| Stat(s) | Source | Computation plan | Sampling | Processor status |
| --- | --- | --- | --- | --- |
| Average boost amount, BPM | Current boost value from processor | Integrate boost amount over `dt`; derive average and per-minute usage/collection stats | Prefer full frame deltas | `Available` |
| Time at 0 boost, Time at 100 boost | Current boost value from processor | Integrate `dt` while boost equals threshold | Prefer full frame deltas | `Available` |
| Time in boost ranges 0-25 / 25-50 / 50-75 / 75-100 | Current boost value from processor | Integrate `dt` into range buckets; report seconds and percentages | Prefer full frame deltas | `Available` |
| Average distance to teammates | Player rigid bodies | Integrate pairwise teammate distance over `dt` and average per player | Prefer full frame deltas or deterministic resampling | `Available` |
| Percent most back / most forward | Player positions projected along team attack axis | At each sample choose relative ordering within team, then integrate `dt` | Needs a stable sample policy | `Partial` |
| Percent in defensive / neutral / offensive third | Player positions | Bucket field zone per sample, integrate `dt` | Needs a stable sample policy | `Available` |
| Percent in defensive / offensive half | Player positions | Bucket field half per sample, integrate `dt` | Needs a stable sample policy | `Available` |
| Percent closest to ball / farthest from ball | Player-ball distances | Rank teammates by ball distance per sample, integrate `dt` | Needs a stable sample policy | `Available` |
| Percent behind ball / in front of ball | Player and ball positions | Classify by attack-direction-relative position, integrate `dt` | Needs a stable sample policy | `Available` |
| Average distance to ball / has possession / no possession | Player-ball distances plus possession owner | Integrate distance over `dt` in global and possession-conditioned buckets | Needs possession event/source | `Partial` |
| Total distance traveled | Player rigid bodies | Sum distance deltas between successive positions | Prefer full frame deltas | `Available` |
| Average speed (uu/s), Average speed (% of max) | Player velocities or distance over time | Integrate speed over `dt`; normalize by max car speed for percent | Prefer full frame deltas | `Available` |
| Time / percent at slow, boost, supersonic speed | Player speed buckets | Bucket speed per sample, integrate `dt`; report seconds or percentages | Prefer full frame deltas | `Available` |
| Time / percent on ground, low air, high air | Player `z` / grounded heuristics | Bucket vertical state per sample, integrate `dt` | Needs consistent thresholds | `Available` |
| Team movement aggregates | Per-player movement reducers | Sum or average player reducers by team after accumulation | Same as player metrics | `Available` |

## Newly Supported Processor Signals

| Stat(s) | Source | Computation plan | Sampling | Processor status |
| --- | --- | --- | --- | --- |
| Powerslide active | `TAGame.Vehicle_TA:bReplicatedHandbrake` on car actor | Read as boolean input signal for downstream reducers | Full frame deltas preferred | `Available` |
| Powerslide total duration | Powerslide active boolean | Integrate `dt` while active | Prefer full frame deltas | `Partial` |
| Powerslide average duration | Powerslide active boolean | Detect active segments, divide total active time by segment count | Prefer full frame deltas | `Partial` |
| Powerslide count / presses | Powerslide active boolean | Count rising edges `false -> true` | None beyond frame traversal | `Partial` |

## Missing Event Sources

| Stat(s) | Source needed | Computation plan | Sampling | Processor status |
| --- | --- | --- | --- | --- |
| Possession | Ball touch ownership per player/team | Track last touching side/player until touch changes; integrate possession time | Event-driven plus `dt` | `Missing` |
| Pressure (time ball is in each side) | Ball position | Classify ball side per sample and integrate `dt` | Prefer full frame deltas | `Available` |
| Shots on timeline | Shot event stream or reconstruction heuristic | Prefer explicit event extraction; fallback heuristic from touches + goal/saves is risky | Event-driven | `Missing` |
| Saves on timeline | Save event stream or reconstruction heuristic | Prefer explicit event extraction | Event-driven | `Missing` |
| Assists on timeline | Assist event stream or reconstruction heuristic | Prefer explicit event extraction | Event-driven | `Missing` |
| Goals on timeline | Goal event timestamps | Detect from game-state transitions or score deltas | Event-driven | `Partial` |
| Goals conceded while last defender | Goal event timestamps plus positional role at concession time | Determine defender ordering immediately before goal | Event-driven plus sampled state | `Missing` |
| Amount collected / amount stolen | Boost pad pickup events with pad identity/team side | Sum boost gained from pickup events; classify home vs enemy side | Event-driven | `Missing` |
| Big pads collected / Small pads collected | Boost pad pickup events with pad size | Count pickup events by pad size | Event-driven | `Missing` |
| Stolen big pads / Stolen small pads | Boost pad pickup events with pad size and side | Count enemy-side pickups by pad size | Event-driven | `Missing` |
| Amount collected from big/small pads | Boost pad pickup events with exact gain value | Sum boost gain by pad size | Event-driven | `Missing` |
| Amount stolen from big/small pads | Boost pad pickup events with exact gain value and side | Sum enemy-side boost gain by pad size | Event-driven | `Missing` |
| Overfill total / from stolen | Pickup gain plus current boost before pickup | Compute wasted gain at pickup time, bucket by home/enemy side | Event-driven | `Missing` |

## Processor Work Queue

- Add a dedicated stats module with reducer traits instead of making every stat a top-level `Collector`.
- Keep `ReplayProcessor` responsible for exposing raw normalized signals, not final Ballchasing stats.
- Add touch extraction:
  - per-touch player
  - touch timestamp
  - team in possession after touch
- Add boost pad pickup extraction:
  - pad actor identity
  - pad size
  - world position / team-side classification
  - pickup timestamp
  - player responsible
  - boost before/after pickup if derivable
- Add score / goal event extraction:
  - score delta timestamp
  - scoring team
  - scorer if derivable
- Consider a normalized sample struct for reducers:
  - time
  - dt
  - ball state
  - per-player kinematics
  - boost amount
  - powerslide active
  - current possession owner if known

## Recommended First Stats Reducers

- Boost bucket reducer
- Movement speed bucket reducer
- Ground/air reducer
- Distance-to-ball reducer
- Distance-to-teammates reducer
- Powerslide reducer
- Demo counter reducer
- Pressure reducer
