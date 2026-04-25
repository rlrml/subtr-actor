# Rocket League replay format evolution

This is a working map of the replay-format changes that currently matter to
`subtr-actor`. It is not a complete replay-format specification. It focuses on
network fields that affect physical game state, because those fields feed replay
data, ndarray features, stats, and viewer playback.

The most important conclusion is that there is no single "old replay scale"
rule. Different fields changed at different `net_version` boundaries, and some
fields with similar Rust types encode different physical concepts.

## Version markers

The useful markers are:

- `major_version`
- `minor_version`
- `net_version`
- `BuildVersion` header property

`net_version` is the best discriminator observed so far for network-data
encoding. `major_version` and `minor_version` are too coarse on their own. For
example, both sampled 2018-04-02 and 2018-04-04 replays are `868.20`, but the
former has `net_version = 2` and the latter has `net_version = 5`, and their
rigid-body vector units differ.

Sampled timeline:

| Replay date | major.minor | net | BuildVersion | Notes |
| --- | --- | --- | --- | --- |
| 2016-08-01 | `868.12` | `None` | `160705.43783.134970` | old LAN/online era |
| 2016-11-01 | `868.12` | `None` | `160921.8478.141010` | old LAN/online era |
| 2017-06-01 | `868.17` | `None` | `170501.51736.158700` | old LAN/online era |
| 2017-12-01 | `868.20` | `2` | `171105.50789.177172` | pre-vector-scale transition |
| 2018-02-01 | `868.20` | `2` | `171122.50648.178784` | pre-vector-scale transition |
| 2018-03-01 | `868.20` | `2` | `180123.67440.183219` | pre-vector-scale transition |
| 2018-04-02 | `868.20` | `2` | `180215.52441.185728` | pre-vector-scale transition |
| 2018-04-04 | `868.20` | `5` | `180315.66224.188644` | vector scale changed, rotation still legacy |
| 2018-06-01 | `868.22` | `7` | `180517.71295.194805` | modern rigid-body rotation |

Checked-in fixture inspection links:

| Fixture | Replay version | Spatial rule | Format signal | Raw | Viewer |
| --- | --- | --- | --- | --- | --- |
| `rlcs.replay` | `868.14`, `net_version = None` | Legacy, `100x` | Older replay with no net version; exercises legacy rigid-body position scaling and 3v3 stats/touch extraction. | [raw](https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/rlcs.replay) | [viewer](https://rlrml.github.io/subtr-actor/?replayUrl=https%3A%2F%2Fraw.githubusercontent.com%2Frlrml%2Fsubtr-actor%2Fmaster%2Fassets%2Frlcs.replay) |
| `soccar-lan.replay` | `868.12`, `net_version = None` | Legacy, `100x` | Older LAN-style replay with no net version; useful for checking legacy player and ball positions. | [raw](https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/soccar-lan.replay) | [viewer](https://rlrml.github.io/subtr-actor/?replayUrl=https%3A%2F%2Fraw.githubusercontent.com%2Frlrml%2Fsubtr-actor%2Fmaster%2Fassets%2Fsoccar-lan.replay) |
| `old_boost_format.replay` | `868.32`, `net_version = 10` | Modern, native scale | Modern spatial scale with the older boost attribute shape. | [raw](https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/old_boost_format.replay) | [viewer](https://rlrml.github.io/subtr-actor/?replayUrl=https%3A%2F%2Fraw.githubusercontent.com%2Frlrml%2Fsubtr-actor%2Fmaster%2Fassets%2Fold_boost_format.replay) |
| `new_boost_format.replay` | `868.32`, `net_version = 10` | Modern, native scale | Modern spatial scale with the newer `ReplicatedBoost` format. | [raw](https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/new_boost_format.replay) | [viewer](https://rlrml.github.io/subtr-actor/?replayUrl=https%3A%2F%2Fraw.githubusercontent.com%2Frlrml%2Fsubtr-actor%2Fmaster%2Fassets%2Fnew_boost_format.replay) |
| `new_demolition_format.replay` | `868.32`, `net_version = 10` | Modern, native scale | Newer `ReplicatedDemolishExtended` demolition payload; regression coverage expects 10 demos and preserved victim locations. | [raw](https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/new_demolition_format.replay) | [viewer](https://rlrml.github.io/subtr-actor/?replayUrl=https%3A%2F%2Fraw.githubusercontent.com%2Frlrml%2Fsubtr-actor%2Fmaster%2Fassets%2Fnew_demolition_format.replay) |
| `tourny.replay` | `868.29`, `net_version = 10` | Modern, native scale | Tournament-style replay; useful for checking metadata/header handling around ordinary spatial interpretation. | [raw](https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/tourny.replay) | [viewer](https://rlrml.github.io/subtr-actor/?replayUrl=https%3A%2F%2Fraw.githubusercontent.com%2Frlrml%2Fsubtr-actor%2Fmaster%2Fassets%2Ftourny.replay) |
| `dodges_refreshed_counter.replay` | `868.32`, `net_version = 11` | Modern, native scale | Newer replay with `DodgeRefreshedCounter`; expected to expose 12 exact dodge refreshes. | [raw](https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/dodges_refreshed_counter.replay) | [viewer](https://rlrml.github.io/subtr-actor/?replayUrl=https%3A%2F%2Fraw.githubusercontent.com%2Frlrml%2Fsubtr-actor%2Fmaster%2Fassets%2Fdodges_refreshed_counter.replay) |

Use these links to visually check that cars and ball fit normal Rocket League
field dimensions, rotations track plausible car orientation, and replay events
line up with visible play. Legacy fixtures should appear in field units after
normalization, while modern fixtures should not be multiplied by `100`.

## Rigid-body state

`TAGame.RBActor_TA:ReplicatedRBState` is the main field where historical
versioning matters for `subtr-actor`.

### Location

Current normalized-output rule:

- `net_version` missing or `< 5`: multiply boxcars output by `100`
- `net_version >= 5`: passthrough

Evidence:

- Historical `net=None` and `net=2` samples have boxcars-decoded rigid-body
  location magnitudes around `60`; multiplying by `100` puts cars and ball in
  plausible Rocket League field units.
- `net=5` and `net=7` samples have boxcars-decoded magnitudes around `6000`
  already, so multiplying would overscale them.
- `RocketLeagueReplayParser` also has a position branch at `netVersion >= 5`:
  older replays use `Vector3D`, newer replays use `FixedPointVector3D`.

### Linear and angular velocity

Current normalized-output rule:

- `net_version` missing or `< 5`: multiply boxcars output by `10`
- `net_version >= 5`: passthrough

Evidence:

- Historical `net=None` and `net=2` samples have boxcars-decoded linear
  velocity magnitudes that are too small by about one order of magnitude.
- After multiplying by `10`, reported velocity agrees with frame-to-frame
  displacement in the replay-plausibility heuristic.
- `net=5` and `net=7` samples are already in plausible velocity units.

This boundary is the same as rigid-body location, but the multiplier is not.
That is why a blanket `*100` rule for old rigid-body vectors is wrong.

### Rotation

Current normalized-output rule:

- `net_version` missing or `< 7`: interpret boxcars' `Quaternion { x, y, z, w: 0 }`
  as a legacy fixed compressed `(pitch, yaw, roll)` rotator and convert it to a
  quaternion with `Quat::from_euler(EulerRot::ZYX, y * PI, x * PI, -z * PI)`
- `net_version >= 7`: passthrough modern quaternion

Evidence:

- `RocketLeagueReplayParser` reads `netVersion >= 7` rigid-body rotation as a
  quaternion and older rotation as `Vector3D.DeserializeFixed`.
- `Vector3D.DeserializeFixed` reads three fixed compressed floats with `max=1`
  and `bits=16`.
- `Rattletrap` uses the same version boundary: at least `868.22` with
  `net_version = 7` is a quaternion, older versions are a
  `CompressedWordVector`.
- Before conversion, legacy samples have non-unit "quaternion" norms and
  grounded car forward vectors point mostly opposite travel. After conversion,
  historical samples from 2016-08 through 2018-04 have unit-length quaternions
  and grounded forward alignment in the same range as a sampled `net=7` replay.
- On `assets/rlcs.replay`, the negated legacy roll sign makes frame-to-frame
  orientation deltas align with reported rigid-body angular velocity during
  high-spin aerial frames; direct `+roll` preserves grounded yaw but mirrors
  flip/roll motion more often.

Rotation changes later than rigid-body vector scale: `net=5` uses modern
location and velocity units, but still uses legacy rotation.

## Other Vector3-like fields

The other `Vector3f` fields should not inherit rigid-body rules by type alone.
Observed field ranges show that they are field-specific:

| Field | Observed boxcars-output pattern | Current interpretation |
| --- | --- | --- |
| `Demolish.attacker_velocity` / `victim_velocity` | about `23` across sampled versions | exported as physical velocity with `*100`; this is not the rigid-body velocity rule |
| `DemolishExtended.victim_location` | derived from normalized victim rigid-body location | field-scale output |
| `Explosion.location` / `ExtendedExplosion.explosion.location` | about `30-52` across sampled versions | appears to be encoded in `1/100` field units, but not currently central to `subtr-actor` output |
| `AppliedDamage.position` / `DamageState.ball_position` | about `40-46` in Dropshot samples | appears to be encoded in `1/100` field units, but should be validated before using as physical position |
| `Attribute::Location` | about `2.6` across sampled versions | likely not a world-space field position; do not normalize blindly |
| `Welded.offset` | about `1-2` in sampled Rumble replays | likely a local offset; do not normalize blindly |
| `NewActor.initial_trajectory.location` | `Vector3i`, not `Vector3f` | initial spawn position; currently unused by `subtr-actor` physical outputs |

The main practical rule is: normalize by semantic field, not by Rust type.
`Vector3f` is only the decoded shape.

## Validation heuristics

The local probe binary is:

`cargo run --bin replay_probe -- <metadata|plausibility|legacy-rotation|demolition|vector-ranges> <replay-path>`

The useful checks are:

- velocity/displacement consistency: compare reported velocity against
  frame-to-frame displacement
- field bounds: reject positions and speeds that are far outside plausible game
  ranges
- quaternion norms: rigid-body rotations should be unit quaternions after
  normalization
- grounded forward alignment: when a grounded car is moving fast, its local
  forward vector should usually align with planar velocity
- raw vector ranges: compare decoded magnitudes by field and by replay version

These checks do not prove every semantic detail, but they are strong enough to
detect the old vector-scale and legacy-rotation failures.

## Current confidence

High confidence:

- `RigidBody.location` scale boundary is `net_version < 5`.
- `RigidBody.linear_velocity` and `angular_velocity` scale boundary is
  `net_version < 5`, with a different multiplier from location.
- `RigidBody.rotation` format boundary is `net_version < 7`.
- `net_version`, not just `major_version` / `minor_version`, must drive these
  rules.
- `Vector3f` fields cannot be normalized uniformly by type.

Medium confidence:

- The legacy rotation conversion is operationally correct for player-car
  orientation in sampled historical replays. Grounded motion validates yaw,
  pitch/uprightness, and unit norms strongly, and the RLCS angular-velocity
  regression validates the roll sign during aerial spins.

Open areas:

- More `net_version` values should be sampled if we can find them, especially
  around the `4 -> 5` and `6 -> 7` boundaries.
- Non-standard modes can expose additional field semantics. Dropshot, Rumble,
  and demolition/explosion fields should be validated before promoting them
  into normalized public output.
- This document explains the replay fields `subtr-actor` depends on. It should
  not be treated as a complete parser spec.

## References

- `RocketLeagueReplayParser` rigid-body version branches:
  https://github.com/jjbott/RocketLeagueReplayParser/blob/master/RocketLeagueReplayParser/NetworkStream/RigidBodyState.cs
- `RocketLeagueReplayParser` vector and fixed-vector decoding:
  https://github.com/jjbott/RocketLeagueReplayParser/blob/master/RocketLeagueReplayParser/NetworkStream/Vector3D.cs
- `Rattletrap` rotation version branch:
  https://github.com/tfausak/rattletrap/blob/main/src/lib/Rattletrap/Type/Rotation.hs
- `Rattletrap` compressed-word vector type:
  https://github.com/tfausak/rattletrap/blob/main/src/lib/Rattletrap/Type/CompressedWordVector.hs
