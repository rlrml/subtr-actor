# Scoring threat and touch expected goals

[Deployed v7 evaluation and provenance](EVALUATION.md).

Two models answer different questions:

- **Scoring threat**: probability this team scores in the next five seconds,
  before the live-play stretch ends or the other team scores. Evaluate on live frames.
- **Touch xG**: probability this team scores within ten seconds, before its next
  touch, the other team's goal, or the end of the live-play stretch. Evaluate
  only at detected touches, using a snapshot from 100–200 ms before contact.

`ExpectedGoalsTeamStats.xg` and player `xg` sum touch probabilities. Every
eligible goal has at most one positive touch example per team. There is no
shot-classifier gate, peak selection, or multiplier fitted to goal totals.
A subsequent teammate touch starts a new opportunity. Touch attribution is
accounting for these opportunities, not a causal allocation of contribution;
passes and carries can create threat without scoring on their own interval.
Simultaneous same-team contacts produce one opportunity, credited to the primary
contact selected by the touch detector; shared contact credit is not modeled.

`current_threat` is the five-second probability. `threat_integral` accumulates
`V * dt / 5` as duration-weighted pressure. An integral over overlapping windows
can approximate goal counts, but live-play boundaries, short stretches, and
prediction error prevent treating that as an exact identity per replay.
Episode peaks describe intensity only. Episode boundaries affect display and
pressure summaries; they cannot alter touch xG.

## Inputs and inference

Rust owns feature extraction for training and playback. `ThreatEntityFeatures`
exports the legacy 154-value state/history vector, six ball position/velocity
components, and four complete 17-value player vectors. The additional player
field records whether physics state is available, distinguishing missing cars
from stationary cars. Unavailable spatial fields use placeholders; replicated
boost and demo information remain available. `state_valid` marks the spatial
fields as unavailable. Legacy mean/spread features remain a baseline and context;
the new vectors retain which car has each position, velocity, facing, boost,
and dodge/demo state. History currently summarizes teams over 0.5 and 1 second;
it does not track individual players through time.

The entity network applies shared learned layers to each player together with
ball context, pools within each team, then combines those representations with
global context. Swapping either team's players leaves predictions unchanged.
It also averages the probabilities of the original state and its left/right
reflection. Small dense layers support native and WASM inference without a
runtime ML dependency. Torch is an optional **offline** dependency only.

## Reproduce

Fetch a corpus with `fetch_corpus.py` if needed. The existing manifest format
contains local replay paths, replay IDs, dates, playlist, team size, and rank
tiers. All training here is restricted to ranked doubles.

```sh
cargo run --release -p subtr-actor-tools --bin threat_dataset_dump -- \
  --manifest /path/to/manifest.jsonl --entity-features --sample-hz 2 --threads 4 \
  --out /path/to/frames.csv --touch-out /path/to/touches.csv \
  --episode-summary /path/to/goals.csv

cd scripts/threat_model
uv sync --locked --extra neural
uv run --locked --extra neural prepare_entity_splits.py \
  /path/to/manifest.jsonl /path/to/frames.audit.jsonl /path/to/splits.json
uv run --locked --extra neural train_entity_model.py /path/to/frames.csv \
  --task frame --split-file /path/to/splits.json --out-dir /path/to/frame-model
uv run --locked --extra neural train_entity_model.py /path/to/touches.csv \
  --task touch --split-file /path/to/splits.json --goal-counts /path/to/goals.csv \
  --epochs 80 --out-dir /path/to/touch-model
```

The exporter writes `frames.audit.jsonl` beside the frame CSV. It records every
observed goal and live segment, touch coverage, unavailable pre-contact history,
player IDs for overlap audits, and skipped replays. A goal-to-segment coverage
mismatch rejects that replay rather than silently generating false negatives.
Goal timestamps are capped at the associated live boundary when replicated
notifications arrive after play stops. Samples after an earlier recorded goal
are excluded. Unknown final stretches are censored; known stoppages resolve
negative examples. A future touch time is a **label boundary**, never a model input.

`prepare_entity_splits.py` excludes identical replay files and freezes common
60/15/10/15 partitions by replay date and ID for training, checkpoint/model
selection, probability calibration, and final evaluation. Both teams stay in
the same partition. It reports repeated-player overlap. This is a historical
replay holdout, not evidence of generalization to unseen players or new patches;
the corpus has been used in earlier experiments.

The trainer compares a refit eight-unit legacy model and the entity model,
selects using the tuning partition, then evaluates the test partition. The
exported weights are exactly the evaluated weights: no full-corpus refit.
Reports include probability scores, fixed-band and per-rank calibration,
replay-bootstrap confidence intervals for the paired model difference, and
(for touches) counts against **actual goals**, label coverage, and diagnostics
by subsequent touch interval. The future interval is used only for those
diagnostics. Artifacts include weights, parity examples, feature names, source
hashes, split membership, seed, and fit/calibration provenance.

For deployment, copy the selected `weights.rs` and `parity.rs` into the matching
`expected_goals_entity_{frame,touch}_{weights,parity}.rs` files, update the model
version and checked-in evaluation report, then run focused Rust tests, the
Python model tests, binding regeneration, and the repository checks.

## Interpretation and limitations

- Missing pre-contact history produces unavailable xG, not a fabricated zero
  probability. The cumulative sum includes only evaluated touches; report
  coverage when comparing it with actual goals. Team stats expose
  `evaluated_touch_count` and `unavailable_touch_count`; the scoreboard tooltip
  shows their ratio.
- Goals without an eligible preceding scoring-team touch, or after the ten-second
  horizon, cannot receive a positive label. Report the deficit rather than
  correcting it with a global multiplier.
- Frequent carry contacts split opportunities more finely. This is part of the
  next-touch target; interval diagnostics expose its behavior. Future contact
  spacing must not be used to improve a supposedly pre-contact prediction.
- `threat_added` retains signed detection-frame changes. It remains an observed
  state change, not a causal measurement of a complete multi-frame touch impulse.
- The original `train_threat_model.py` and v6 weights remain for legacy feature
  experiments and parity comparisons. Use `train_entity_model.py` for the new
  touch target and evaluated deployment artifacts.

For the frozen v7 corpus, `verify_entity_runtime.py CORPUS RUNTIME_SUMMARY --out
REPORT` compares native replay summaries with offline predictions on every
exported test touch. `summarize_entity_models.py CORPUS --output .
--runtime-report REPORT` rebuilds the checked-in evaluation and provenance.
