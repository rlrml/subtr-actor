from __future__ import annotations

LABEL_PLAYER_FEATURE_ADDER = "PlayerDodgeRefreshed"
LABEL_PLAYER_HEADER = "dodge refresh count"
TIME_HEADER = "current time"

RIGID_BODY_GLOBAL_FEATURE_ADDER_BY_ENCODING: dict[str, str] = {
    "euler": "BallRigidBody",
    "quaternion": "BallRigidBodyQuaternionVelocities",
    "basis": "BallRigidBodyBasis",
}

RIGID_BODY_PLAYER_FEATURE_ADDER_BY_ENCODING: dict[str, str] = {
    "euler": "PlayerRigidBody",
    "quaternion": "PlayerRigidBodyQuaternionVelocities",
    "basis": "PlayerRigidBodyBasis",
}

BASE_GLOBAL_FEATURE_ADDERS: tuple[str, ...] = (
    "CurrentTime",
    "ReplicatedStateName",
    "BallHasBeenHit",
)

BASE_PLAYER_FEATURE_ADDERS: tuple[str, ...] = (
    "PlayerRelativeBallPosition",
    "PlayerRelativeBallVelocity",
    "PlayerLocalRelativeBallPosition",
    "PlayerLocalRelativeBallVelocity",
    "PlayerBoost",
    "PlayerJump",
    "PlayerAnyJump",
)


def global_feature_adders_for_orientation(orientation_encoding: str) -> tuple[str, ...]:
    rigid_body_adder = RIGID_BODY_GLOBAL_FEATURE_ADDER_BY_ENCODING[orientation_encoding]
    return (rigid_body_adder, *BASE_GLOBAL_FEATURE_ADDERS)


def player_feature_adders_for_orientation(orientation_encoding: str) -> tuple[str, ...]:
    rigid_body_adder = RIGID_BODY_PLAYER_FEATURE_ADDER_BY_ENCODING[orientation_encoding]
    return (rigid_body_adder, *BASE_PLAYER_FEATURE_ADDERS)


DEFAULT_GLOBAL_FEATURE_ADDERS: tuple[str, ...] = global_feature_adders_for_orientation("euler")
DEFAULT_PLAYER_FEATURE_ADDERS: tuple[str, ...] = player_feature_adders_for_orientation("euler")


def ensure_label_player_feature(player_feature_adders: tuple[str, ...]) -> tuple[str, ...]:
    if LABEL_PLAYER_FEATURE_ADDER in player_feature_adders:
        return player_feature_adders
    return player_feature_adders + (LABEL_PLAYER_FEATURE_ADDER,)
