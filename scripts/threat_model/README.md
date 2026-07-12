# Threat model training pipeline

Offline pipeline that fits the expected-goals threat model embedded in
`src/stats/calculators/expected_goals_model.rs`. The model is
`V = sigmoid(bias + w · features)`: the probability that the attacking team
scores within `THREAT_HORIZON_SECONDS` (5s), evaluated per frame on the
`ThreatFeatures` vector. Feature extraction lives only in Rust
(`compute_threat_features`); training consumes rows exported through that
exact code path, so train and inference can never diverge.

## Steps

1. **Fetch a corpus** (optional — any manifest of local replays works):

   ```sh
   python3 fetch_corpus.py
   ```

   Downloads a rank-stratified sample of processed replays from the
   rocket-sense production API (JWT read from `pass show rocket-sense/token`)
   into `~/.cache/subtr-actor-threat-corpus/`, and writes `manifest.jsonl`
   there. Tune `PER_STRATUM` (default 150 per playlist × rank tier) via env.

2. **Export the dataset** through the shared Rust feature path:

   ```sh
   cargo run --release -p subtr-actor-tools --bin threat_dataset_dump -- \
       --manifest ~/.cache/subtr-actor-threat-corpus/manifest.jsonl \
       --out threat_dataset.csv --sample-hz 4
   ```

   Two attacking-normalized rows per sampled live-play frame (one per team),
   with τ-agnostic goal-time columns for downstream labeling/censoring.

3. **Train and evaluate** (needs numpy/pandas/scikit-learn):

   ```sh
   python3 train_threat_model.py threat_dataset.csv --tau 5.0 --gbt \
       --out-dir threat_model_out
   ```

   Grouped train/test split by replay. Writes `metrics.txt` (log-loss, Brier,
   AUC, calibration table, per-rank-tier calibration), `coefficients.json`,
   `model_coefficients.rs`, and `parity_fixture.rs`. `--gbt` also fits a
   gradient-boosted reference to quantify what the linear model leaves behind.

4. **Embed**: paste `model_coefficients.rs` into the GENERATED COEFFICIENTS
   section of `src/stats/calculators/expected_goals_model.rs`, bump
   `THREAT_MODEL_VERSION` (`trained-v<N>`), refresh the provenance comment and
   the parity fixture in `expected_goals_model_tests.rs` from
   `parity_fixture.rs`, and run `cargo test --lib expected_goals`.

## trained-v1

Fit 2026-07-12 on 10.3M rows from 5,280 rank-stratified ranked-duels/-doubles
replays (rocket-sense production, tiers 1–22, ~150 per playlist × tier where
available). Held-out: log-loss 0.169 / Brier 0.0468 / AUC 0.885 vs 0.252
baseline log-loss; GBT ceiling 0.154. Calibration tracks observed frequency
across all 15 prediction-quantile bins and across rank tiers (drift ≲10%
relative for mid/high tiers), supporting a single rank-blind model as v1.
Aggregate sanity: integrated V·dt/τ recovers 6.72 mean goals/game vs 6.67
actual; per-replay corr(xG, goals) = 0.78.
