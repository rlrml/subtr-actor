use super::*;

pub(super) fn vec3_json(value: &Vector3f) -> Value {
    json!({
        "x": value.x,
        "y": value.y,
        "z": value.z,
    })
}

fn quat_json(value: &Quaternion) -> Value {
    json!({
        "x": value.x,
        "y": value.y,
        "z": value.z,
        "w": value.w,
    })
}

fn rigid_body_json(value: &RigidBody) -> Value {
    json!({
        "location": vec3_json(&value.location),
        "rotation": quat_json(&value.rotation),
        "sleeping": value.sleeping,
        "linear_velocity": value.linear_velocity.as_ref().map(vec3_json),
        "angular_velocity": value.angular_velocity.as_ref().map(vec3_json),
    })
}

pub(super) fn ball_frame_state_json(state: &BallFrameState) -> Value {
    match state {
        BallFrameState::Missing => json!({
            "kind": "Missing",
            "ball": Value::Null,
        }),
        BallFrameState::Present(ball) => json!({
            "kind": "Present",
            "ball": ball_sample_json(ball),
        }),
    }
}

fn ball_sample_json(sample: &BallSample) -> Value {
    json!({
        "rigid_body": rigid_body_json(&sample.rigid_body),
    })
}

pub(super) fn player_sample_json(sample: &PlayerSample) -> Value {
    json!({
        "player_id": sample.player_id,
        "is_team_0": sample.is_team_0,
        "rigid_body": sample.rigid_body.as_ref().map(rigid_body_json),
        "boost_amount": sample.boost_amount,
        "last_boost_amount": sample.last_boost_amount,
        "boost_active": sample.boost_active,
        "dodge_active": sample.dodge_active,
        "powerslide_active": sample.powerslide_active,
        "match_goals": sample.match_goals,
        "match_assists": sample.match_assists,
        "match_saves": sample.match_saves,
        "match_shots": sample.match_shots,
        "match_score": sample.match_score,
    })
}
