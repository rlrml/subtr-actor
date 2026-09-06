# Expected goals v7 evaluation

The deployed entity models were selected on the tuning partition. Their exact
exported weights were then evaluated on 381 historical replay holdouts.
No full-corpus refit or goal-total multiplier was applied.

| Target | Model | Test log loss ↓ | Brier ↓ | AUC ↑ |
|---|---|---:|---:|---:|
| frame | legacy-refit | 0.12806 | 0.03404 | 0.8958 |
| frame | entity | 0.12102 | 0.03243 | 0.9093 |
| touch | legacy-refit | 0.09880 | 0.02637 | 0.9243 |
| touch | entity | 0.09385 | 0.02548 | 0.9360 |

Relative to the eight-unit legacy model refitted for each target, test log loss
improved by 5.01% for pre-contact touch xG and
5.50% for five-second scoring threat. Both models for a
target received the same epoch budget and early-stopping rule. See the JSON
report for complete training histories, replay-bootstrap confidence intervals,
fixed probability bands, and rank breakdowns. Both legacy fits exhausted their
epoch budgets (30 for frames, 80 for touches); their learning curves were still
improving. The comparison establishes an improvement under these training
budgets, not superiority to every fully optimized legacy fit.

On resolvable touch examples across 762 test team-games,
summed touch xG averaged
2.752 versus 2.730 actual goals.
Mean absolute count error was 1.111 and correlation was
0.571. Eligible touch labels cover
98.51% of actual goals; the remaining goals
lack an eligible touch interval or fall beyond its ten-second horizon.
Predictions exceed eligible positive labels by
2.33%.
That calibration drift partly offsets the missing goal coverage; the close
aggregate mean alone does not establish calibration.

## Data and interpretation

The audit accepted 2,540 ranked-doubles replays and rejected
22 with ambiguous goal attachment. Of
382,842 detected primary touches,
24,083 lacked a snapshot 100–200 ms
before contact. These receive unavailable xG. Unknown live-play endings are
censored when their label cannot be resolved. Full goal and coverage counts
are recorded in the JSON audit summary.

Replay-date partitions contain 1,524 training,
381 tuning, 254
calibration, and 381 test replays.
There were 0 identical replay files excluded.
Of 1,144 test players,
47 also occur in training. This corpus was used in earlier modeling
experiments: this is historical replay validation, not a prospective test or
evidence of generalization to new players, game modes, or patches.

Touch xG predicts conversion before the next own-team touch or live-play end,
within ten seconds. It assigns one opportunity to the primary contact when
same-team contacts coincide. It is not causal contribution credit. The separate
five-second threat model describes current danger; its integral measures
cumulative pressure. Neither episode peaks nor eventual goal outcomes change
a touch's probability.

The future-touch interval groups in the JSON are outcome-conditioned diagnostics,
not calibration tests: any interval ending at another touch necessarily has
label zero. Terminal intervals, included separately, contain the goals. Model
inputs never include future interval length.

[Machine-readable evaluation and hashes](entity-v7-evaluation.json) ·
[Frozen replay splits and exclusions](entity-v7-splits.json) ·
[Training contracts and reproduction](README.md)

## Native replay validation

All 381 held-out replays were processed by the deployed Rust
pipeline without rejection. Across 54,095
touches, native and offline per-team sums differ by at most
0.00001018 xG.
The native totals include predictions near unresolved replay endings whose
outcomes cannot enter the supervised evaluation above.

| Runtime quantity | Mean | Actual goals | MAE | Correlation |
|---|---:|---:|---:|---:|
| Touch xG | 2.781 | 2.730 | 1.110 | 0.571 |
| Cumulative threat | 2.775 | 2.730 | 1.011 | 0.656 |

The cumulative threat integral is approximately on a goal-count scale in this
sample. It also observes the ball after contact, so its count correlation is
not a comparison between predictors with the same information.
