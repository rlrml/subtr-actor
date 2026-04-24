## Legacy replay investigation

This note captures the current evidence behind the legacy rigid-body
normalization rules and the remaining legacy rotation issue.

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

### Rotation issue

Legacy rigid-body rotation is still wrong for sampled `net < 7` replays.

Observed plausibility pattern:

- `net=None` and `net=2` replays:
  quaternion norm error is about `0.73`, and grounded forward alignment is
  strongly negative.
- `net=5` replays:
  same failure pattern as `net=2`.
- `net=7` replays:
  quaternion norms are near unit length and forward alignment is correct.

This means the remaining rotation bug is not specific to the oldest replays.
It appears to affect the whole legacy compressed-quaternion path used by
`boxcars` for `net_version < 7`.

### Current conclusion

- The old `net_version < 7 => *100 everything` rule was wrong.
- Legacy rigid-body location and legacy rigid-body velocity need different
  scale factors.
- `net_version` is the best normalization discriminator we have observed so
  far; `major_version` / `minor_version` are too coarse.
- `BuildVersion` is still useful supporting evidence when comparing samples
  within the same `major.minor`.
- The remaining unresolved issue is legacy compressed rigid-body rotation for
  `net_version < 7`.
