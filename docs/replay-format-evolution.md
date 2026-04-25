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

Checked-in coverage timeline:

The fixture filenames use the `BuildVersion` date because that is the stable
format marker available in the replay header. Older short fixture names are
still checked in for compatibility with existing tests, but the table below uses
the clearer replay-format fixture names.

| Fixture | Replay version | BuildVersion | Rigid-body rule | Format signal | Raw | Viewer |
| --- | --- | --- | --- | --- | --- | --- |
| `replay-format-2016-07-21-v868-12-net-none-lan.replay` | `868.12`, `net_version = None` | `160721.58730.135786` | Legacy vectors and legacy rotation | Old LAN-style replay with no `GameEvent_Soccar` archetype; useful for checking metadata fallback plus legacy player and ball positions. | [raw][raw-2016-07-lan] | [viewer][viewer-2016-07-lan] |
| `replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay` | `868.14`, `net_version = None` | `161109.39595.145160` | Legacy vectors and legacy rotation | Older RLCS LAN replay with no net version; exercises legacy rigid-body position scaling and 3v3 stats/touch extraction. | [raw][raw-2016-11-rlcs] | [viewer][viewer-2016-11-rlcs] |
| `replay-format-2017-03-16-v868-17-net-none-online.replay` | `868.17`, `net_version = None` | `170316.47017.154572` | Legacy vectors and legacy rotation | Later no-net online replay; keeps the old-era coverage from being only 2016 LAN-style samples. | [raw][raw-2017-03-net-none] | [viewer][viewer-2017-03-net-none] |
| `replay-format-2017-11-22-v868-20-net2-legacy-vectors.replay` | `868.20`, `net_version = 2` | `171122.50648.178784` | Legacy vectors and legacy rotation | Pre-vector-scale transition sample; validates that `net=2` still uses old location, velocity, and rotation interpretation. | [raw][raw-2017-11-net2] | [viewer][viewer-2017-11-net2] |
| `replay-format-2018-03-15-v868-20-net5-modern-vectors-legacy-rotation.replay` | `868.20`, `net_version = 5` | `180315.66224.188644` | Modern vectors, legacy rotation | Boundary sample where rigid-body location and velocity are native scale but rotation is still legacy compressed rotator data. | [raw][raw-2018-03-net5] | [viewer][viewer-2018-03-net5] |
| `replay-format-2018-05-17-v868-22-net7-modern-rigidbody.replay` | `868.22`, `net_version = 7` | `180517.71295.194805` | Modern vectors and modern quaternion rotation | First checked-in boundary sample where rigid-body rotation is already a modern quaternion. | [raw][raw-2018-05-net7] | [viewer][viewer-2018-05-net7] |
| `replay-format-2019-04-19-v868-24-net10-modern-rigidbody.replay` | `868.24`, `net_version = 10` | `190419.41693.231343` | Modern rigid body | Early `net=10` sample before later boost and demolition payload changes. | [raw][raw-2019-04-net10] | [viewer][viewer-2019-04-net10] |
| `replay-format-2020-09-25-v868-29-net10-tournament.replay` | `868.29`, `net_version = 10` | `200925.55985.293168` | Modern rigid body | Tournament-style replay; useful for checking metadata/header handling around ordinary spatial interpretation. | [raw][raw-2020-09-tournament] | [viewer][viewer-2020-09-tournament] |
| `replay-format-2022-09-29-v868-32-net10-legacy-boost.replay` | `868.32`, `net_version = 10` | `220929.397994` | Modern rigid body | Modern spatial scale with the older boost attribute shape. | [raw][raw-2022-09-legacy-boost] | [viewer][viewer-2022-09-legacy-boost] |
| `replay-format-2025-06-10-v868-32-net10-replicated-boost.replay` | `868.32`, `net_version = 10` | `250610.60392.487806` | Modern rigid body | Modern spatial scale with the newer `ReplicatedBoost` format. | [raw][raw-2025-06-replicated-boost] | [viewer][viewer-2025-06-replicated-boost] |
| `replay-format-2026-01-14-v868-32-net10-demolish-extended.replay` | `868.32`, `net_version = 10` | `260114.55864.507183` | Modern rigid body | Newer `ReplicatedDemolishExtended` demolition payload; regression coverage expects 10 demos and preserved victim locations. | [raw][raw-2026-01-demolish-extended] | [viewer][viewer-2026-01-demolish-extended] |
| `replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay` | `868.32`, `net_version = 11` | `260303.78181.511382` | Modern rigid body | Newer replay with `DodgeRefreshedCounter`; expected to expose 12 exact dodge refreshes. | [raw][raw-2026-03-dodge-refresh] | [viewer][viewer-2026-03-dodge-refresh] |

Known coverage gaps:

- The checked-in fixtures cover the boundaries currently known to affect
  `subtr-actor`: missing/old `net_version`, `net=2`, `net=5`, `net=7`,
  `net=10`, and `net=11`.
- We do not currently have checked-in fixtures for every intermediate
  `net_version` value such as `0`, `1`, `3`, `4`, `6`, `8`, or `9`. Add those
  if a parser behavior or public output depends on them.

[raw-2016-07-lan]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2016-07-21-v868-12-net-none-lan.replay
[viewer-2016-07-lan]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2016-07-21-v868-12-net-none-lan.replay
[raw-2016-11-rlcs]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay
[viewer-2016-11-rlcs]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay
[raw-2017-03-net-none]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2017-03-16-v868-17-net-none-online.replay
[viewer-2017-03-net-none]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2017-03-16-v868-17-net-none-online.replay
[raw-2017-11-net2]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2017-11-22-v868-20-net2-legacy-vectors.replay
[viewer-2017-11-net2]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2017-11-22-v868-20-net2-legacy-vectors.replay
[raw-2018-03-net5]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2018-03-15-v868-20-net5-modern-vectors-legacy-rotation.replay
[viewer-2018-03-net5]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2018-03-15-v868-20-net5-modern-vectors-legacy-rotation.replay
[raw-2018-05-net7]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2018-05-17-v868-22-net7-modern-rigidbody.replay
[viewer-2018-05-net7]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2018-05-17-v868-22-net7-modern-rigidbody.replay
[raw-2019-04-net10]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2019-04-19-v868-24-net10-modern-rigidbody.replay
[viewer-2019-04-net10]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2019-04-19-v868-24-net10-modern-rigidbody.replay
[raw-2020-09-tournament]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2020-09-25-v868-29-net10-tournament.replay
[viewer-2020-09-tournament]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2020-09-25-v868-29-net10-tournament.replay
[raw-2022-09-legacy-boost]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2022-09-29-v868-32-net10-legacy-boost.replay
[viewer-2022-09-legacy-boost]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2022-09-29-v868-32-net10-legacy-boost.replay
[raw-2025-06-replicated-boost]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay
[viewer-2025-06-replicated-boost]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay
[raw-2026-01-demolish-extended]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2026-01-14-v868-32-net10-demolish-extended.replay
[viewer-2026-01-demolish-extended]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2026-01-14-v868-32-net10-demolish-extended.replay
[raw-2026-03-dodge-refresh]: https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay
[viewer-2026-03-dodge-refresh]: https://rlrml.github.io/subtr-actor/?replayUrl=https://raw.githubusercontent.com/rlrml/subtr-actor/master/assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay

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
- On `assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay`, the
  negated legacy roll sign makes frame-to-frame orientation deltas align with
  reported rigid-body angular velocity during high-spin aerial frames; direct
  `+roll` preserves grounded yaw but mirrors flip/roll motion more often.

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
