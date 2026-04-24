## Legacy replay investigation

This note captures the current evidence behind the legacy rigid-body
normalization rules.

The local probe entrypoint for these experiments is:

`cargo run --bin replay_probe -- <metadata|plausibility|legacy-rotation|demolition|vector-ranges> <replay-path>`

### Version fields

Replays expose several useful version markers:

- `major_version`
- `minor_version`
- `net_version`
- `BuildVersion` header property

`major_version` and `minor_version` are not enough on their own to describe the
2018 transition. For example, both of these online replays are `868.20`, but
they differ materially:

| Replay date | major.minor | net | BuildVersion |
| --- | --- | --- | --- |
| 2018-04-02 | `868.20` | `2` | `180215.52441.185728` |
| 2018-04-04 | `868.20` | `5` | `180315.66224.188644` |

The sampled timeline currently looks like this:

| Replay date | major.minor | net | BuildVersion |
| --- | --- | --- | --- |
| 2016-08-01 | `868.12` | `None` | `160705.43783.134970` |
| 2016-11-01 | `868.12` | `None` | `160921.8478.141010` |
| 2017-06-01 | `868.17` | `None` | `170501.51736.158700` |
| 2017-12-01 | `868.20` | `2` | `171105.50789.177172` |
| 2018-02-01 | `868.20` | `2` | `171122.50648.178784` |
| 2018-03-01 | `868.20` | `2` | `180123.67440.183219` |
| 2018-04-02 | `868.20` | `2` | `180215.52441.185728` |
| 2018-04-04 | `868.20` | `5` | `180315.66224.188644` |
| 2018-06-01 | `868.22` | `7` | `180517.71295.194805` |

### Vector normalization

The current processor rules are:

- `RigidBody.location`: `*100` when `net_version` is missing or `< 5`
- `RigidBody.linear_velocity`: `*10` when `net_version` is missing or `< 5`
- `RigidBody.angular_velocity`: `*10` when `net_version` is missing or `< 5`
- `RigidBody.rotation`: legacy compressed rotator conversion when
  `net_version` is missing or `< 7`; modern quaternion passthrough for
  `net_version >= 7`
- `Demolish.attacker_velocity` / `victim_velocity`: `*100` across versions

These rules are backed by replay plausibility checks and historical samples:

- `net=None` legacy LAN replays (`assets/rlcs.replay`, `assets/soccar-lan.replay`)
  have sane motion under the current rules.
- `net=2` online replays from late 2017 through 2018-04-02 also have sane
  motion under the current rules.
- `net=5` and `net=7` replays look correct without the legacy rigid-body scale.

This supports the current rigid-body boundary at `< 5`, not `< 7`.

Raw vector range checks provide another field-specific signal:

| Field | Observed raw magnitude pattern | Current interpretation |
| --- | --- | --- |
| `RigidBody.location` | about `60` for `net=None/2`, about `6000` for `net>=5` | legacy location needs `*100`; `net>=5` is already field-scale |
| `RigidBody.linear_velocity` | about `300` for old LAN samples, about `2300-3300` for `net>=5` | legacy velocity needs `*10`; `net>=5` is already velocity-scale |
| `RigidBody.angular_velocity` | about `60` for old LAN samples, about `600` for `net>=5` | legacy angular velocity needs `*10`; `net>=5` is already angular-velocity-scale |
| `Demolish.*velocity` | about `23` across sampled versions | demo velocities are not rigid-body velocities as decoded; current export scales them by `*100` |
| `Explosion.location` / `ExtendedExplosion.explosion.location` | about `30-52` across sampled versions and modes | explosion positions appear to be encoded in `1/100` field units across versions |
| `AppliedDamage.position` / `DamageState.ball_position` | about `40-46` in Dropshot samples | damage positions appear to be encoded in `1/100` field units across versions |
| `Attribute::Location` | about `2.6` across sampled versions | likely not a field position; do not apply rigid-body normalization blindly |
| `Welded.offset` | about `1-2` in sampled Rumble replays | likely a local offset; do not apply rigid-body normalization blindly |

### Rotation normalization

Legacy rigid-body rotation uses a different encoding for sampled `net < 7`
replays. `boxcars` decodes that path as three compressed fixed floats and
stores them in `Quaternion { x, y, z, w: 0 }`, but those values are not a
modern quaternion.

Other parsers agree on the version boundary:

- `jjbott/RocketLeagueReplayParser` reads `netVersion >= 7` as `Quaternion`
  and older versions as a fixed compressed `Vector3D`.
- `Rattletrap` reads `net_version >= 7` as `Quaternion` and older versions as a
  `CompressedWordVector`.

The processor converts the legacy vector into a quaternion with:

`Quat::from_euler(EulerRot::ZYX, y * PI, x * PI, z * PI)`

This treats the raw legacy vector as `(pitch, yaw, roll)` and converts it to a
quaternion using the usual yaw/pitch/roll axis order. The sign and order were
checked against historical replay plausibility checks.
Before this conversion, `net=None`, `net=2`, and `net=5` samples had quaternion
norm errors around `0.73` and grounded player forward vectors usually pointed
opposite travel. After conversion, those samples have unit-length rotations and
grounded forward alignment consistent with modern `net=7` replays.

The remaining uncertainty is semantic rather than operational: grounded motion
strongly validates yaw and uprightness, while roll sign is less directly
observable from grounded samples. The selected roll sign is the direct parser
order and remains plausible across the historical samples checked so far.

### Current conclusion

- The old `net_version < 7 => *100 everything` rule was wrong.
- Legacy rigid-body location and legacy rigid-body velocity need different
  scale factors.
- `net_version` is the best normalization discriminator we have observed so
  far; `major_version` / `minor_version` are too coarse.
- `BuildVersion` is still useful supporting evidence when comparing samples
  within the same `major.minor`.
- Rigid-body rotation has a different boundary from rigid-body vector scale:
  vector scale changes at `net_version >= 5`, while rotation changes at
  `net_version >= 7`.
