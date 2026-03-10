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

## Implemented Reducers

- `MatchStatsReducer`: score, goals, assists, saves, shots, shooting percentage, exact goal timeline, exact shot/save/assist timeline, goals conceded while last defender
- `DemoReducer`: demos inflicted by team and player, demos taken by player, kill timeline, death timeline
- `PressureReducer`: time and percentage of ball-side pressure
- `PossessionReducer`: team possession time and percentage using exact team-touch boundaries
- `BoostReducer`: boost amount buckets, BPM, exact pad pickup counts and pad-size classification, collected/stolen amounts, overfill, supersonic boost usage
- `PositioningReducer`: teammate distance, ball distance, back/forward role percentages, thirds/halves, closest/farthest to ball, behind/in front of ball
- `MovementReducer`: distance, average speed, speed buckets, ground/air buckets, team aggregates
- `PowerslideReducer`: total powerslide duration, count, average duration, team aggregates
- `SettingsReducer`: camera settings and steering sensitivity from replay metadata

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
| Team touch events | `TAGame.Ball_TA:HitTeamNum` updates | Extract exact touch timestamps from `HitTeamNum` attribute updates; these include same-team consecutive touches, not just possession changes | None | `Available` |
| Player touch attribution | Exact team touch events plus motion-aware car ranking | Use velocity-applied ball/car states at the touch frame and rank touching-team players by short-window closest approach to the ball; this is still heuristic, but more faithful than raw nearest-car distance | None | `Partial` |
| Ball has been hit | `get_ball_has_been_hit` | Useful kickoff-phase signal, but not enough for possession/touches by itself | None | `Available` |

## Existing Time-Integrated Signals

| Stat(s) | Source | Computation plan | Sampling | Processor status |
| --- | --- | --- | --- | --- |
| Average boost amount, BPM | Current boost value from processor | Integrate boost amount over `dt`; derive average and per-minute usage/collection stats | Prefer full frame deltas | `Available` |
| Time at 0 boost, Time at 100 boost | Current boost value from processor | Integrate `dt` while boost equals threshold | Prefer full frame deltas | `Available` |
| Time in boost ranges 0-25 / 25-50 / 50-75 / 75-100 | Current boost value from processor | Integrate `dt` into range buckets; report seconds and percentages | Prefer full frame deltas | `Available` |
| Average distance to teammates | Player rigid bodies | Integrate pairwise teammate distance over `dt` and average per player | Prefer full frame deltas or deterministic resampling | `Available` |
| Percent most back / most forward | Player positions projected along team attack axis | At each sample choose relative ordering within team, then integrate `dt` | Needs a stable sample policy | `Available` |
| Percent in defensive / neutral / offensive third | Player positions | Bucket field zone per sample, integrate `dt` | Needs a stable sample policy | `Available` |
| Percent in defensive / offensive half | Player positions | Bucket field half per sample, integrate `dt` | Needs a stable sample policy | `Available` |
| Percent closest to ball / farthest from ball | Player-ball distances | Rank teammates by ball distance per sample, integrate `dt` | Needs a stable sample policy | `Available` |
| Percent behind ball / in front of ball | Player and ball positions | Classify by attack-direction-relative position, integrate `dt` | Needs a stable sample policy | `Available` |
| Average distance to ball / has possession / no possession | Player-ball distances plus possession owner | Integrate distance over `dt` in global and possession-conditioned buckets using exact team-touch boundaries; player-level possession ownership is still heuristic | Needs replay-native player touch ownership for exact player parity | `Partial` |
| Total distance traveled | Player rigid bodies | Sum distance deltas between successive positions | Prefer full frame deltas | `Available` |
| Average speed (uu/s), Average speed (% of max) | Player velocities or distance over time | Integrate speed over `dt`; normalize by max car speed for percent | Prefer full frame deltas | `Available` |
| Time / percent at slow, boost, supersonic speed | Player speed buckets | Bucket speed per sample, integrate `dt`; report seconds or percentages | Prefer full frame deltas | `Available` |
| Time / percent on ground, low air, high air | Player `z` / grounded heuristics | Bucket vertical state per sample, integrate `dt` | Needs consistent thresholds | `Available` |
| Team movement aggregates | Per-player movement reducers | Sum or average player reducers by team after accumulation | Same as player metrics | `Available` |

## Newly Supported Processor Signals

| Stat(s) | Source | Computation plan | Sampling | Processor status |
| --- | --- | --- | --- | --- |
| Powerslide active | `TAGame.Vehicle_TA:bReplicatedHandbrake` on car actor | Read as boolean input signal for downstream reducers | Full frame deltas preferred | `Available` |
| Powerslide total duration | Powerslide active boolean | Integrate `dt` while active | Prefer full frame deltas | `Available` |
| Powerslide average duration | Powerslide active boolean | Detect active segments, divide total active time by segment count | Prefer full frame deltas | `Available` |
| Powerslide count / presses | Powerslide active boolean | Count rising edges `false -> true` | None beyond frame traversal | `Available` |

## Missing Event Sources

| Stat(s) | Source needed | Computation plan | Sampling | Processor status |
| --- | --- | --- | --- | --- |
| Possession | Ball touch ownership per player/team | Exact team-touch timestamps are available from `HitTeamNum` updates and reducers now buffer those events across sampled frames. Player ownership remains heuristic rather than replay-native | Event-driven plus `dt` for exact team parity | `Partial` |
| Pressure (time ball is in each side) | Ball position | Classify ball side per sample and integrate `dt` | Prefer full frame deltas | `Available` |
| Shots on timeline | Exact `PRI_TA:MatchShots` counter increment events | Emit shot events directly from replay counter increments and buffer them across sampled frames | Exact event time from replay frame updates | `Available` |
| Saves on timeline | Exact `PRI_TA:MatchSaves` counter increment events | Emit save events directly from replay counter increments and buffer them across sampled frames | Exact event time from replay frame updates | `Available` |
| Assists on timeline | Exact `PRI_TA:MatchAssists` counter increment events | Emit assist events directly from replay counter increments and buffer them across sampled frames | Exact event time from replay frame updates | `Available` |
| Goals on timeline | Exact ball goal explosion events plus `PRI_TA:MatchGoals` frame updates | Extract exact goal timestamps from `Ball_TA:ReplicatedExplosionData`, derive scorer identity from same-frame `PRI_TA:MatchGoals` updates when present, and synthesize cumulative score tuples when live team scores are absent | Exact event time plus replay-frame scorer derivation | `Available` |
| Goals conceded while last defender | Team score deltas plus positional role at concession time | Determine defender ordering at the scoring frame and attribute conceded goals to the most-back defender | Event-driven plus sampled state | `Available` |
| Amount collected / amount stolen | Exact boost pad pickup events plus canonical standard-soccar pad layout matching | Pad pickups are extracted exactly from `VehiclePickup_TA:NewReplicatedPickupData`; for standard soccar, side classification is derived from the matched canonical pad position instead of raw player position. Outside canonical matches, we fall back to per-pad observed pickup centroids with a midfield neutral tolerance | Event-driven | `Available` |
| Big pads collected / Small pads collected | Exact boost pad pickup events plus replay-observed pad cooldown | Pad size is inferred exactly from that pad's observed respawn cadence (`~4s` vs `~10s`) and then cached per pad id | Event-driven | `Available` |
| Stolen big pads / Stolen small pads | Exact boost pad pickup events plus canonical standard-soccar pad layout matching | Pad size is exact; for standard soccar, enemy-side classification comes from the matched canonical pad position. Outside canonical matches, the fallback uses per-pad observed pickup centroids with a midfield neutral tolerance | Event-driven | `Available` |
| Amount collected from big/small pads | Exact boost pad pickup events, inferred pad size, and pre-pickup boost amount | Collected amount is computed from nominal pad gain and pre-pickup boost instead of from noisy observed boost deltas | Event-driven | `Available` |
| Amount stolen from big/small pads | Exact boost pad pickup events, inferred pad size, canonical pad position, and pre-pickup boost amount | Collected amount and pad size are exact; for standard soccar, enemy-side classification comes from the matched canonical pad position. Outside canonical matches, the fallback uses per-pad observed pickup centroids with a midfield neutral tolerance | Event-driven | `Available` |
| Overfill total / from stolen | Exact boost pad pickup events, inferred pad size, canonical pad position, and pre-pickup boost amount | Overfill is computed from nominal pad gain minus exact capped collection; for standard soccar, enemy-side classification comes from the matched canonical pad position. Outside canonical matches, the fallback uses per-pad observed pickup centroids with a midfield neutral tolerance | Event-driven | `Available` |

## Processor Work Queue

- Add a dedicated stats module with reducer traits instead of making every stat a top-level `Collector`.
- Keep `ReplayProcessor` responsible for exposing raw normalized signals, not final Ballchasing stats.
- Add touch extraction:
  - per-touch player
  - touch timestamp
  - team in possession after touch
- Improve goal event extraction further:
  - replace current replay-mode dedupe window with a stronger replay-state signal if we find one
- Continue making reducer inputs sampling-invariant by buffering exact replay events between sampled collector invocations
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
