# Flip Reset Transformer

This subproject is a first pass at training a transformer to detect flip resets
from `subtr-actor` replay features.

The current baseline is deliberately narrow:

- Training labels come from the exact `PlayerDodgeRefreshed` signal.
- That label channel is appended to the ndarray request only for supervision.
- The label channel is stripped before model inputs are built.
- Training examples are per-player temporal windows centered on a candidate frame.

## Why This Shape

Flip reset detection is a local temporal classification problem. The thing that
matters is not "what frame number is this in the replay", but "where is this
token relative to the center of the decision window".

The baseline therefore uses:

- Fixed-width windows around a center frame
- Learned relative position embeddings for token offsets within that window
- A continuous relative-time scalar for each token

That combination is a better fit than whole-replay absolute positional encoding
for the initial model. If we later move to variable-rate inputs or longer
contexts, rotary or continuous Fourier time encodings are the next things to
try.

## Data Contract

Inputs currently use:

- Global adders: `BallRigidBody`, `CurrentTime`, `ReplicatedStateName`, `BallHasBeenHit`
- Player adders: `PlayerRigidBody`, `PlayerBoost`, `PlayerJump`, `PlayerAnyJump`

The supervision channel is:

- `PlayerDodgeRefreshed`

The dataset builder also derives relative ball-minus-player position and
velocity features when the requested headers expose those channels.

## Leakage Rules

- `PlayerDodgeRefreshed` is always requested for label construction.
- `"dodge refresh count"` is never included in model inputs.
- `"current time"` is used to compute relative token times and is then removed
  from model inputs.
- The existing heuristic flip-reset events are not used as inputs or labels in
  this baseline.

## Workflow

1. Build or install the local Python bindings for `subtr-actor`.
2. Point the trainer at one or more replay directories.
3. Train on fixed-FPS windows with replay-level train/validation splits.

Example setup:

```bash
cd python
uv sync --group dev
uv run maturin develop

cd ../ml/flip_reset_transformer
uv sync
uv run python -m flip_reset_transformer.train \
  --replay-dir ../../data/flip-reset-ground-truth-exact/replays \
  --output-dir artifacts/baseline
```

Pretraining can use the same direct replay-directory flow:

```bash
uv run python -m flip_reset_transformer.pretrain \
  --replay-dir ../../data/flip-reset-ground-truth-exact/replays \
  --output-dir artifacts/pretrain
```

## Current Limitations

- Negatives are sampled from non-event frames within positive replays.
- The baseline focuses on the player plus ball, not all cars on the field.
- Labels are binary event/no-event at the center frame, not a full sequence tagger.
- No inference CLI yet. The first goal is a defensible training and evaluation loop.
