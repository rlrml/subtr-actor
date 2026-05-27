use super::bump_evaluate_event::bump_event_from_selection;
use super::bump_evaluate_selection::selected_bump_direction;
use super::bump_geometry::{contact_normal, directional_candidate, swept_horizontal_distance};
use super::*;

impl BumpCalculator {
    pub(super) fn evaluate_pair(
        frame: &FrameInfo,
        left: &PlayerSample,
        left_body: &boxcars::RigidBody,
        previous_left_body: &boxcars::RigidBody,
        right: &PlayerSample,
        right_body: &boxcars::RigidBody,
        previous_right_body: &boxcars::RigidBody,
    ) -> Option<BumpEvent> {
        let left_previous_position = vec_to_glam(&previous_left_body.location);
        let right_previous_position = vec_to_glam(&previous_right_body.location);
        let left_position = vec_to_glam(&left_body.location);
        let right_position = vec_to_glam(&right_body.location);

        let contact_distance = swept_horizontal_distance(
            left_previous_position,
            left_position,
            right_previous_position,
            right_position,
        );
        if contact_distance > BUMP_MAX_CONTACT_DISTANCE {
            return None;
        }

        let vertical_gap = (left_position.z - right_position.z)
            .abs()
            .min((left_previous_position.z - right_previous_position.z).abs());
        if vertical_gap > BUMP_MAX_VERTICAL_GAP {
            return None;
        }

        let normal_left_to_right = contact_normal(
            left_previous_position,
            left_position,
            right_previous_position,
            right_position,
        )?;
        let left_to_right = directional_candidate(
            previous_left_body,
            left_body,
            previous_right_body,
            right_body,
            normal_left_to_right,
        )?;
        let right_to_left = directional_candidate(
            previous_right_body,
            right_body,
            previous_left_body,
            left_body,
            -normal_left_to_right,
        )?;

        let selected = selected_bump_direction(
            left,
            left_body,
            right,
            right_body,
            left_to_right,
            right_to_left,
        );
        bump_event_from_selection(frame, contact_distance, selected)
    }
}
