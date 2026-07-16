# Threat model training pipeline

Offline pipeline that fits the expected-goals threat model embedded in
`src/stats/calculators/expected_goals_model.rs`. The deployed model is a compact
eight-hidden-unit tanh MLP that estimates the probability that the attacking
team scores within `THREAT_HORIZON_SECONDS` (5s), evaluated per frame on the
`ThreatFeatures` vector. Feature extraction lives only in the Rust ndarray
feature layer (`ThreatFeatures` / `ThreatModelValues`); the runtime model and
training exporter consume that same analysis-backed row state, so train and
inference cannot diverge. A logistic model remains the transparent baseline,
and a gradient-boosted tree remains the nonlinear reference ceiling.

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
   `parity_fixture.rs`. The script evaluates the logistic baseline and smooth
   nonlinear model, then refits the MLP on the full corpus and emits its folded
   raw-feature weights. `--gbt` also fits a gradient-boosted reference ceiling.

4. **Embed**: replace
   `src/stats/calculators/expected_goals_model_weights.rs` with
   `model_coefficients.rs`, bump `THREAT_MODEL_VERSION` (`trained-v<N>`),
   refresh the provenance comment and the parity fixture in
   `expected_goals_model_tests.rs` from `parity_fixture.rs`, and run
   `cargo test --lib expected_goals`.

## trained-v5

Fit 2026-07-15 on 5.22M team rows from 2,544 rank-stratified ranked-doubles
replays (rocket-sense production, tiers 3–22, ~150 per tier where available).
Every player receives the same 16 inputs, including boost, inferred dodge
availability, and demo state; each team pair is aggregated without ordering.
On the newest 509 replays (1.04M rows), the eight-unit tanh MLP reaches 0.13005
log-loss, 0.03456 Brier score, 0.8936 AUC, and 0.00129 15-bin expected
calibration error. The logistic baseline is 0.13548 log-loss and the GBT
reference ceiling is 0.12816, so the compact smooth model captures roughly
three quarters of the measured nonlinear gap. At 4 Hz it crosses the 15%
incident threshold 6.740 times per live minute versus 6.544 for logistic.
On 1,018 held-out replay/team outcomes, the time-integrated MLP averages 2.795
xG against 2.834 goals (0.986 ratio), with 0.626 per-team-game correlation
versus 0.594 for logistic. Individual match totals remain noisy and should not
be treated as precise forecasts.

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
do not leak into the total. The selected raw peak is multiplied by 0.583503,
fit on the oldest 80% of the ranked-doubles corpus. On the newest 509 held-out
replays (1,018 team-games), incident xG averages 2.891 against 2.797 goals
(3.4% high), with 0.356 per-team-game correlation. The continuous integral
averages 2.827 xG with 0.641 correlation on the same full-replay evaluation.
Both remain available rather than conflating their semantics. Revalidate with
`threat_dataset_dump --episode-summary`.
