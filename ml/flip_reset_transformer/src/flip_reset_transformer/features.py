from __future__ import annotations

LABEL_PLAYER_FEATURE_ADDER = "PlayerDodgeRefreshed"
LABEL_PLAYER_HEADER = "dodge refresh count"
TIME_HEADER = "current time"

DEFAULT_GLOBAL_FEATURE_ADDERS: tuple[str, ...] = (
    "BallRigidBody",
    "CurrentTime",
    "ReplicatedStateName",
    "BallHasBeenHit",
)

DEFAULT_PLAYER_FEATURE_ADDERS: tuple[str, ...] = (
    "PlayerRigidBody",
    "PlayerRelativeBallPosition",
    "PlayerRelativeBallVelocity",
    "PlayerBoost",
    "PlayerJump",
    "PlayerAnyJump",
)


def ensure_label_player_feature(player_feature_adders: tuple[str, ...]) -> tuple[str, ...]:
    if LABEL_PLAYER_FEATURE_ADDER in player_feature_adders:
        return player_feature_adders
    return player_feature_adders + (LABEL_PLAYER_FEATURE_ADDER,)
