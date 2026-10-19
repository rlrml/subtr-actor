# Vision-assisted whiff labels and a telemetry whiff model

[Evaluation, decision, and limitations](EVALUATION.md).

This workflow bootstraps whiff labels from LLM vision judgments of rendered
replay evidence, then trains the small telemetry classifier that the
production `WhiffCalculator` runs (no LLM at inference time).

## Label policy

Each review window is 6 s of replay with a highlighted 2 s core interval. A
labeler lists every interaction in the core where the focal car approaches,
jumps or dodges at, or passes close to the ball, and answers three separate
questions: was an attempt observable, did the focal car touch the ball, and
did another player's touch or bump disrupt the opportunity? The derived
classes are kept separate in labels, training targets, metrics, and counts:

- `clear_whiff`: attempted, no contact, not disrupted, not a plausible fake.
- `disrupted_attempt`: attempted, no contact, another player took the ball or
  the car away first. Never merged into whiffs or negatives.
- `successful_contact`: the focal car touched the ball.
- `no_observable_attempt`: proximity without visible intent, including fakes.
- `uncertain`: unresolvable; excluded from training and from every metric.

Prompts never show detector verdicts, proposal sources, or player names;
windows carry neutral IDs in random order.

## Pipeline

```sh
# Telemetry dump (per-frame detector inputs, touches, current detector events)
cargo run --release -p subtr-actor-tools --bin whiff_dataset_dump -- \
  --manifest ~/.cache/subtr-actor-whiff/manifest.jsonl --out-dir ~/.cache/subtr-actor-whiff/dumps

just whiff-model mine                       # proposals + random + tiled windows, replay-level splits
just whiff-model freeze --name NAME ...     # immutable stratified manifest
just whiff-model render --manifest M        # replay sheets, trajectory plots, telemetry strip
just whiff-model label --manifest M --representations compact                     # Claude
just whiff-model label --manifest M --representations combined --transport codex  # Codex
just whiff-model adjudicate --manifest M    # match judges, <=2 blinded re-reviews, telemetry checks
just whiff-model train --name N --labels ... --test-labels ...
just whiff-model select --name NAME --model N   # active-learning batch from train replays

# Deployed model: train on features computed by the core detector itself
cargo run --release -p subtr-actor-tools --bin whiff_candidate_dump -- \
  --manifest REPLAYS.jsonl --out-dir ~/.cache/subtr-actor-whiff/work/v2/core_cands
just whiff-model train-core --name whiff-gbt-vN --refit-all \
  --candidates ~/.cache/subtr-actor-whiff/work/v2/core_cands \
  --labels <train/dev final label files> --test-labels <test final label files>
# writes src/stats/calculators/whiff_model_weights.rs and its parity fixture

# Before/after evaluation on replays never used for training
just whiff-model freeze-fresh --name NAME --replay-manifest FRESH.jsonl \
  --candidates DIR --quotas '{"both": 90, "new_only": 80, "legacy_only": 90, "near_unflagged": 100}'
just whiff-model score-fresh --manifest ~/.cache/subtr-actor-whiff/work/manifests/NAME.jsonl
```

Bulk artifacts (`dumps/`, `renders/`, `labels/attempts.jsonl`, models) live
under `~/.cache/subtr-actor-whiff`. The attempt ledger is append-only and
content-keyed (images, prompt, transport, model, settings), so interrupted runs
resume and repeated reviews keep earlier provenance. `--max-calls` and
`--max-cost` bound every labeling run. Transports are the `claude -p` and
`codex exec` CLIs; both read images from disk.

Replays are split 60/20/20 by content hash before any windows are mined, so
every window and player of a match stays in one split. Model and threshold
selection use only `dev`; the frozen `test` manifests were labeled with the
final rubric and evaluated once.

## Inference

The production detector lives in `src/stats/calculators/whiff*.rs`: candidate
closest approaches (local contact-gap minima during live play), 66 causal
features read up to the first frame 0.6 s after the anchor, and the generated
tree weights. `train-core` fits on vectors dumped by that same code
(`whiff_candidate_dump`), so there is no second feature implementation to keep
in parity; `whiff_model_tests.rs` checks Rust inference against scikit-learn
probabilities on exported rows. `fresh_eval.py` scores the new and legacy
detectors with Horvitz-Thompson weights over a stratified sample; the legacy
events come from `whiff_dataset_dump` built from the pre-model commit.

The v1 experiment's shadow implementation remains in
`crates/subtr-actor-tools/src/whiff_model/`: it re-implements candidate generation
(local minima of the ball contact gap during live play), the 44 causal
features (nothing after anchor + 0.6 s), and tree evaluation. `whiff_model_shadow`
scores replays with an exported `model.json`;
`tests/whiff_model_parity_test.rs` requires exact feature and probability
parity with Python on a tracked fixture replay.
