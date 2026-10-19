# Whiff detector v2 evaluation (`whiff-gbt-v2`, deployed)

**Decision: promoted.** The production `WhiffCalculator` now classifies every
closest approach with `whiff-gbt-v2` and emits clean whiffs and disrupted
attempts as separate event kinds and counts (`whiff_count`,
`disrupted_attempt_count`). All reference labels come from two vision-model
judges (Claude Opus 5.5 on trajectory + telemetry, Codex `gpt-5.6-sol` on the
full packet) with blinded re-review of disagreements; they are not expert human
labels, and both judges see the same heuristic touch markers.

## What changed

- Candidates are local minima of each player's ball contact gap during live
  play (radius 600 uu), resolved exactly 0.6 s later. A touch no longer clears
  every pending candidate, so a renewed attempt after the player's own touch or
  a second miss one second after the first are judged individually.
- 66 causal features, computed by the core crate itself: v1's gap history,
  orientation, dodge/boost and touch timing, plus disruption and commitment
  evidence: time and ball impulse of the latest other-player touch, the car's
  closest distance to the ball's *pre-touch* ballistic path versus the real one,
  dodge start relative to that touch, predicted miss distance and time of a
  constant-velocity intercept at -0.6/-0.3/-0.15 s, acceleration alignment
  toward the ball, forward-alignment mean and trend, airborne time, and the
  time and separation since the player's previous touch.
- Training reads vectors dumped by the production code
  (`whiff_candidate_dump`), so inference cannot drift from training features.
  Rust inference matches scikit-learn to 1e-9 (`whiff_model_tests.rs`).

## Selection and held-out sets

Replay-grouped 5-fold CV over the non-test labels (3 seeds) selected 100
depth-2 stages, thresholds 0.5 (clean) / 0.4 (disrupted); CV macro event F1
0.842 vs about 0.80 for v1 features. CV clean whiff P 0.78 / R 0.74; aerial (car z >
150) clean-whiff recall 31/39, grounded 17/23, so aerial misses are no longer
the dominant error. The v1 test files were then scored once with a model fit on
train + dev (they had been scored once by v1 but not used for v2 tuning):

| set | system | clean P | clean R | disrupted P | disrupted R |
| --- | --- | --- | --- | --- | --- |
| stratified (280 windows) | v2 | 18/19 [0.82, 1.0] | 0.90 [0.75, 1.0] | 0.80 [0.59, 0.96] | 0.83 [0.70, 0.96] |
| | legacy | 0.24 [0.14, 0.36] | 0.50 | 0.52 | 0.58 |
| continuous (400 windows) | v2 | 5/7 | 5/9 | 3/6 | 3/8 |
| | legacy | 4/10 | 4/9 | 2/7 | 2/8 |

The deployed weights were refit on every label file (91 clean whiffs, 222
disrupted, 699 contacts, 467 no attempt; 254 uncertain excluded).

## Fresh before/after evaluation (`fresh-eval-v1`)

18 rank-stratified ranked-doubles replays never used for training or tuning
(about 300 live player-minutes). Legacy events come from the pre-model commit.
Population: interactions flagged by both systems (199), only v2 (155), only
legacy (308), and unflagged closest approaches within 300 uu (2,241); 90 / 80 /
90 / 100 sampled, labeled blind with the same rubric. Estimates are
Horvitz-Thompson weighted, 95 % replay-bootstrap intervals. "Strict" counts
anchors the judges marked `uncertain` (mostly possible fakes / unclear intent)
as errors.

| system | class | events | precision | strict precision | recall |
| --- | --- | --- | --- | --- | --- |
| v2 | clean whiff | 129 | 0.84 [0.70, 0.95] | 0.41 [0.32, 0.50] | 0.96 [0.86, 1.0] |
| legacy | clean whiff | 233 | 0.18 [0.08, 0.33] | 0.11 [0.05, 0.21] | 0.48 [0.29, 0.70] |
| v2 | disrupted | 225 | 0.79 [0.69, 0.89] | 0.49 [0.41, 0.56] | 0.71 [0.55, 0.90] |
| legacy (beaten to ball) | disrupted | 274 | 0.27 [0.16, 0.40] | 0.24 [0.14, 0.36] | 0.42 [0.28, 0.57] |

Estimated confirmed clean whiffs in these replays: 56 (v2 finds about 53 of
them with 129 events; legacy about 27 with 233). Of 90 sampled legacy-only
interactions, 48 were successful contacts and 25 no-attempt.

## Errors and limits

- Uncertain anchors are the main residual: 31 of 62 sampled v2 clean-whiff
  windows and 42 of 108 disrupted ones were labeled `uncertain`, so counts
  include many possible fakes or intent-ambiguous plays. `confidence` on each
  event is the model probability; it is not calibrated agreement.
- Disrupted-attempt recall (0.71) is the weaker class.
- Recall is relative to interactions the population covers: misses that never
  come within 600 uu and are not flagged by either detector are not measured.
  No clean whiffs were found among 100 sampled unflagged near approaches.
- Feature importance is led by touch timing (`focal_touch_dt`,
  `other_touch_rel`), so errors of the heuristic touch detector propagate.
- Both judges share the same telemetry markers; agreement is not independent
  confirmation.

---

# Whiff model v1 evaluation (superseded) (`whiff-gbt-v1`)

Machine-readable report: [`whiff-gbt-v1-evaluation.json`](whiff-gbt-v1-evaluation.json).
Exported model and parity fixture: `crates/subtr-actor-tools/assets/whiff_model/`.

**Decision: not promoted.** The model stays an experimental shadow detector;
the production `WhiffCalculator` is unchanged. Clean-whiff recall (0.65) and
disrupted-attempt recall on continuous play (2/8) miss the provisional
90 % precision / 80 % recall development target. The reference labels come from
two vision models plus blinded re-review, not from expert humans, so all
numbers below measure agreement with a frozen model-generated reference.

## Corpus and labels

- 127 replays: 115 rank-stratified ranked-doubles replays (tiers 3–22) from
  rocket-sense plus 12 repository fixtures (duels, standard, RLCS). Split
  60/20/20 by replay content before mining.
- 1,780 labeled windows. Final interactions: 724 `successful_contact`,
  347 `no_observable_attempt`, 230 `disrupted_attempt`, 91 `clear_whiff`,
  262 `uncertain` (excluded). Tiers: 925 agreed, 715 adjudicated by blinded
  re-review, 14 unresolved (kept `uncertain`).
- Judges: Claude Opus 5.5 (`claude -p`) on trajectory + telemetry views and
  Codex (`gpt-5.6-sol`) on replay sheets + trajectory + telemetry. 2,968 Claude
  calls (nominal list price $228, subscription usage) and 2,101 Codex calls,
  including pilot arms and re-reviews.

## Pilot: which evidence is labelable (150 windows, 84 replays)

| Claude arm | uncertain | agreement with final | κ vs Codex combined | $ / usable label |
| --- | --- | --- | --- | --- |
| pseudo-3-D replay sheets | 50 % | 0.42 | 0.24 | 0.125 |
| trajectory plots | 34 % | 0.49 | 0.26 | 0.074 |
| trajectory + telemetry | 20 % | 0.87 | 0.51 | 0.062 |
| all views | 18 % | 0.93 (biased: a final judge) | 0.52 | 0.104 |

The telemetry strip makes contact and disruption decidable; rendered replay
frames alone are mostly ambiguous at contact scale. Trajectory + telemetry
matched the full packet at about 60 % of the cost and was used for scale-up.
Both judges see the same heuristic touch markers, so their agreement is partly
shared evidence rather than independent confirmation.

## Current detector

Of 155 labeled windows anchored on current `WhiffEvent`s: 23 clean whiffs,
27 disrupted attempts, 57 successful contacts, 3 no attempt, 45 uncertain.
Two mechanisms explain most false positives, both visible directly in
telemetry: the candidate re-arms immediately after the player's own touch
while the car is still pushing or carrying the ball (contact gap 0, touch on
the same frames), and it re-arms after another player's touch 0.1–0.3 s before
the focal car arrives. Large aerial misses outside the 220 uu entry gate are
missed entirely.

## Held-out results

Systems are matched one-to-one to labeled events per window (same focal player,
0.4 s tolerance); `uncertain` labels neither reward nor penalize. Intervals are
95 % replay-level bootstrap intervals. The current detector's disrupted-attempt
column uses `BeatenToBallEvent`.

Continuous tiled segments (40 segments, 21 test replays, 13.3 player-minutes;
prevalence-representative but small):

| system | clean P | clean R | clean pred / labeled | disrupted P | disrupted R | FP / player-min (both classes) |
| --- | --- | --- | --- | --- | --- | --- |
| boosted trees | 6/6 | 0.67 [0.43, 1.0] | 6 / 9 | 2/3 | 0.25 [0, 0.75] | 0.07 |
| geometric rule | 0.38 | 0.56 | 13 / 9 | 0.20 | 0.25 | 1.2 |
| current detector | 0.40 | 0.44 | 10 / 9 | 0.29 | 0.25 | 0.83 |

Stratified proposals (280 windows, 22 test replays; oversamples hard cases, so
use for conditional comparison, not prevalence):

| system | clean P | clean R | disrupted P | disrupted R |
| --- | --- | --- | --- | --- |
| boosted trees | 13/13 | 0.65 [0.46, 0.83] | 0.83 [0.61, 1.0] | 0.79 [0.65, 0.93] |
| geometric rule | 0.64 | 0.70 | 0.69 | 0.75 |
| current detector | 0.24 [0.14, 0.36] | 0.50 | 0.52 | 0.58 |

Candidate recall (labeled events with a causal candidate within 0.4 s, test +
dev): 36/36 clean whiffs, 42/47 disrupted attempts. Median timing error for
matched clean whiffs is about 12 ms.

## Errors and limits

- 9 of 10 missed test clean whiffs are aerial (car z 180–580 uu) and are
  predicted `no_observable_attempt`: telemetry alone does not separate an aerial
  miss from an aerial pass or a non-committal jump. That is the same intent
  ambiguity that produces most `uncertain` labels.
- The dev learning curve flattens after about half of the 852 supervised
  examples (macro event F1 0.75 → 0.83 → 0.81 → 0.83), so more labels of the
  same kind are unlikely to fix recall. Better aerial-intent evidence (input
  state, boost/orientation history toward the ball) is the likelier lever.
- Continuous-play counts are small (9 clean, 8 disrupted); its intervals
  are wide. Thresholds were chosen on a dev set with only 7 clean whiffs.
- `focal_touch_dt` and `opponent_touch_dt` dominate feature importance, so the
  model inherits errors of the heuristic touch detector.

## Inference parity

The Rust implementation reproduces the Python candidates, all 44 features
(exact), and class probabilities (|Δ| < 1e-12) on 285 candidates of
`assets/post-eac-ranked-doubles-2026-04-28.replay`
(`cargo test -p subtr-actor-tools --test whiff_model_parity_test`). Features
only read frames up to anchor + 0.6 s, the emission delay a live integration
would need.
