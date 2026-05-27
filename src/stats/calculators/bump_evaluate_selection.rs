use super::*;

pub(super) type BumpSelection<'a> = (
    &'a PlayerSample,
    &'a PlayerSample,
    &'a boxcars::RigidBody,
    &'a boxcars::RigidBody,
    DirectionalBumpCandidate,
    f32,
);

pub(super) fn selected_bump_direction<'a>(
    left: &'a PlayerSample,
    left_body: &'a boxcars::RigidBody,
    right: &'a PlayerSample,
    right_body: &'a boxcars::RigidBody,
    left_to_right: DirectionalBumpCandidate,
    right_to_left: DirectionalBumpCandidate,
) -> BumpSelection<'a> {
    if left_to_right.score >= right_to_left.score {
        (
            left,
            right,
            left_body,
            right_body,
            left_to_right,
            right_to_left.score,
        )
    } else {
        (
            right,
            left,
            right_body,
            left_body,
            right_to_left,
            left_to_right.score,
        )
    }
}
