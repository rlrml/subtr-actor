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
   python3 fetch_corpus.py   # stdlib only
   ```

   Downloads a rank-stratified sample of processed replays from a
   rocket-sense instance into a local cache and writes `manifest.jsonl`
   there. Configured entirely by environment variables (all optional):
   `ROCKET_SENSE_API_TOKEN` (or `ROCKET_SENSE_TOKEN_COMMAND`, defaulting to
   `pass show rocket-sense/token`), `ROCKET_SENSE_BASE_URL`,
   `THREAT_CORPUS_CACHE` (default `~/.cache/subtr-actor-threat-corpus`), and
   `PER_STRATUM` (default 150 per playlist × rank tier).

2. **Export the dataset** through the shared Rust feature path:

   ```sh
   cargo run --release -p subtr-actor-tools --bin threat_dataset_dump -- \
       --manifest ~/.cache/subtr-actor-threat-corpus/manifest.jsonl \
       --out threat_dataset.csv --sample-hz 4
   ```

   Two attacking-normalized rows per sampled live-play frame (one per team),
   with τ-agnostic goal-time columns for downstream labeling/censoring.

3. **Train and evaluate**:

   ```sh
   uv run --script train_threat_model.py threat_dataset.csv --tau 5.0 --gbt \
       --out-dir threat_model_out
   ```

   Dependencies (numpy/pandas/scikit-learn) are declared in the script's
   PEP 723 block and pinned by `train_threat_model.py.lock`, so training runs
   in a reproducible environment and coefficient provenance is auditable.

   Grouped train/test split by replay. Writes `metrics.txt` (log-loss, Brier,
   AUC, calibration table, per-rank-tier calibration), `coefficients.json`,
   `model_coefficients.rs`, and `parity_fixture.rs`. `--gbt` also fits a
   gradient-boosted reference to quantify what the linear model leaves behind.

4. **Embed**: paste `model_coefficients.rs` into the GENERATED COEFFICIENTS
   section of `src/stats/calculators/expected_goals_model.rs`, bump
   `THREAT_MODEL_VERSION` (`trained-v<N>`), refresh the provenance comment and
   the parity fixture in `expected_goals_model_tests.rs` from
   `parity_fixture.rs`, and run `cargo test --lib expected_goals`.

## trained-v2

Fit 2026-07-12 on 10.3M rows from 5,280 rank-stratified ranked-duels/-doubles
replays (rocket-sense production, tiers 1–22, ~150 per playlist × tier where
available), after fixing `defenders_goalside` to normalize by the defending
roster. Held-out: log-loss 0.169 / Brier 0.0468 / AUC 0.885 vs 0.252
baseline log-loss; GBT ceiling 0.154. Calibration tracks observed frequency
across all 15 prediction-quantile bins and across rank tiers (drift ≲10%
relative for mid/high tiers), supporting a single rank-blind model as v1.

## xG aggregation (why the integral)

`V` is calibrated per 5s-window, so summing episode *peaks* over-counts goals
badly (measured 2.7× on this corpus: 9.87 peak-sum vs 3.68 goals per
team-game). The calibrated estimator is the time integral `Σ V·dt/τ`: the
full-match integral recovers 3.37 mean goals per team-game vs 3.33 actual
(within 1%, per-replay corr 0.75), and the within-episode portion of that
integral (what gets player-attributed) captures ~62% of it. This is why
`ThreatEpisodeEvent.xg` is the within-episode integral, team xg is the
full-match integral, and the old peak lives in `peak_value`. Validate any
estimator change with `threat_dataset_dump --episode-summary`.
