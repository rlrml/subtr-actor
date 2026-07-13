# Threat model training pipeline

Offline pipeline that fits the expected-goals threat model embedded in
`src/stats/calculators/expected_goals_model.rs`. The model is
`V = sigmoid(bias + w · features)`: the probability that the attacking team
scores within `THREAT_HORIZON_SECONDS` (5s), evaluated per frame on the
`ThreatFeatures` vector. Feature extraction lives only in the Rust ndarray
feature layer (`ThreatFeatures` / `ThreatModelValues`); the runtime model and
training exporter consume that same analysis-backed row state, so train and
inference cannot diverge.

## Steps

1. **Fetch a corpus** (optional — a ranked-doubles manifest of local replays works):

   ```sh
   python3 fetch_corpus.py --seed 7   # stdlib only
   ```

   Downloads a rank-stratified sample of processed replays from a
   rocket-sense instance into a local cache and writes `manifest.jsonl`
   there. Configured entirely by environment variables (all optional):
   `ROCKET_SENSE_API_TOKEN` (or `ROCKET_SENSE_TOKEN_COMMAND`, defaulting to
   `pass show rocket-sense/token`), `ROCKET_SENSE_BASE_URL`,
   `THREAT_CORPUS_CACHE` (default `~/.cache/subtr-actor-threat-corpus`), and
   `PER_STRATUM` (default 150 per playlist × rank tier),
   `THREAT_CORPUS_SEED` (default 7), and `THREAT_CORPUS_PLAYLISTS` (fixed to
   `ranked-doubles`). The model and exporter intentionally reject 1v1 and 3v3;
   team formats are not mixed into one coefficient set. Selection shuffles
   each rank stratum with the recorded seed instead
   of biasing toward low replay IDs. The fetcher writes
   `manifest.provenance.json` with the seed, playlist set, and SHA-256 hashes of
   both the cached listing and resulting manifest.

2. **Export the dataset** through `NDArrayCollector`:

   ```sh
   cargo run --release -p subtr-actor-tools --bin threat_dataset_dump -- \
       --manifest ~/.cache/subtr-actor-threat-corpus/manifest.jsonl \
       --out threat_dataset.csv --sample-hz 4
   ```

   The collector evaluates its analysis graph on every replay frame, then an
   analysis-aware filter materializes one matrix row at the requested cadence
   during live play. Each matrix row contains both teams' attacking-normalized
   72-value feature vectors and their streaming model values; the exporter
   splits it into one CSV row per team and joins τ-agnostic goal-time columns
   for downstream labeling/censoring. The feature row has eight ball/shot
   values followed by permutation-invariant summaries of the perspective's
   own-team and opponent-team player sets. Every player first receives the
   same 16 position, velocity, facing, ball/goal distance, boost,
   goal-side/net, dodge-available, and demo inputs. Each two-player set is then
   represented by the component-wise mean and absolute spread, so swapping
   either pair cannot change the row and no near/far player role exists.

   This dataset path is separate from normal stats collection. Expected goals
   is an opt-in stats/timeline module and is not evaluated by default.

3. **Train and evaluate** (from the repository root):

   ```sh
   nix run .#train-threat-model -- scripts/threat_model/threat_dataset.csv \
       --tau 5.0 --gbt \
       --manifest ~/.cache/subtr-actor-threat-corpus/manifest.jsonl \
       --out-dir scripts/threat_model/threat_model_out
   ```

   Dependencies (numpy/pandas/scikit-learn) are isolated from the published
   Python bindings in this directory's `pyproject.toml` and pinned by
   `uv.lock`. The repository flake builds the same lock as
   `packages.threat-model-env` and exposes it through the command above and
   `nix develop .#threat-model`. Without Nix, run `uv sync --locked` followed
   by `uv run --locked train_threat_model.py ...` from this directory.

   The newest 20% of replays are held out by replay date for evaluation; after
   metrics are frozen, the publishable coefficients are refit on the complete
   corpus. `metrics.txt` reports log-loss, Brier score, AUC, equal-frequency
   calibration, feature-family knockouts, per-rank calibration, and integrated
   xG versus actual goals per held-out replay/team. This combination measures
   probability accuracy, ranking, forward generalization, feature usefulness,
   and count-scale behavior rather than relying on one headline score.
   The command also writes
   `training_provenance.json` (dataset/manifest and split hashes, seed, Python
   and package versions), `coefficients.json`, `model_coefficients.rs`, and
   `parity_fixture.rs`. `--gbt` also fits a
   gradient-boosted reference to quantify what the linear model leaves behind.

4. **Embed**: paste `model_coefficients.rs` into the GENERATED COEFFICIENTS
   section of `src/stats/calculators/expected_goals_model.rs`, bump
   `THREAT_MODEL_VERSION` (`trained-v<N>`), refresh the provenance comment and
   the parity fixture in `expected_goals_model_tests.rs` from
   `parity_fixture.rs`, and run `cargo test --lib expected_goals`.

## trained-v4

Fit 2026-07-12 on 5.22M team rows from 2,544 rank-stratified ranked-doubles
replays (rocket-sense production, tiers 3–22, ~150 per tier where available).
Every player receives the same 16 inputs, including boost, inferred dodge
availability, and demo state; each team pair is aggregated without ordering.
On the newest 509 replays (1.04M rows), logistic log-loss is 0.1355 versus
0.1964 for the constant-rate baseline, Brier score is 0.0359, AUC is 0.8837,
and 15-bin expected calibration error is 0.0015. The nonlinear GBT ceiling is
0.1287 log-loss, so the deployable linear model captures most—but not all—of
the available signal. Removing all player state worsens log-loss by 0.0180;
mean-substitution knockouts for boost and dodge availability worsen it by
0.00038 and 0.00031 respectively. Demo state is currently neutral after the
other physical inputs are present. On 1,018 held-out replay/team outcomes, the
time-integrated model averages 2.781 xG against 2.834 goals (0.981 ratio), with
0.594 per-team-game correlation. The aggregate count scale is good; individual
match totals remain noisy and should not be treated as precise forecasts.

## xG aggregation

`V` is calibrated per overlapping 5s window. Summing frame or episode peaks
would repeatedly count the same sustained chance, so the count-scale estimator
is the time integral `Σ V·dt/τ`. `ThreatEpisodeEvent.xg` is the within-episode
integral, team xg is the full-match integral, and the peak remains available as
`peak_value` for display/intensity.

The calculator also exposes an incident-based team total. An incident opens
above 15% V, remains open until V falls to 5%, and contributes one selected
peak. For goal-ending incidents, samples from 0.5 seconds before the scoring
team's final touch onward are excluded so nearly determined ball trajectories
do not leak into the total. The selected raw peak is multiplied by 0.629475,
fit on the oldest 80% of the ranked-doubles corpus. On the newest 509 held-out
replays (1,018 team-games), incident xG averages 2.905 against 2.797 goals
(3.9% high). This alternate total has weaker per-game correlation than the
continuous integral, so both remain available rather than conflating their
semantics. Revalidate either estimator with
`threat_dataset_dump --episode-summary`.
