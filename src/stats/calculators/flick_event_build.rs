use super::flick_confidence::flick_event_confidence;
use super::*;

impl FlickCalculator {
    pub(super) fn candidate_event(
        &self,
        ball: &BallFrameState,
        player: &PlayerSample,
        touch_event: &TouchEvent,
        dodge_start: &RecentDodgeStart,
        ball_impulse: glam::Vec3,
    ) -> Option<FlickEvent> {
        let ball = ball.sample()?;
        let player_position = player.position()?;
        let time_since_dodge = touch_event.time - dodge_start.time;
        if !(0.0..=FLICK_MAX_DODGE_TO_TOUCH_SECONDS).contains(&time_since_dodge) {
            return None;
        }

        let ball_speed_change = ball_impulse.length();
        if ball_speed_change < FLICK_MIN_BALL_SPEED_CHANGE {
            return None;
        }

        let to_ball = (ball.position() - player_position).normalize_or_zero();
        let impulse_direction = ball_impulse.normalize_or_zero();
        if to_ball.length_squared() <= f32::EPSILON
            || impulse_direction.length_squared() <= f32::EPSILON
        {
            return None;
        }

        let impulse_away_alignment = impulse_direction.dot(to_ball);
        if impulse_away_alignment < FLICK_MIN_IMPULSE_AWAY_ALIGNMENT {
            return None;
        }

        let event = flick_event_from_parts(
            player,
            touch_event,
            dodge_start,
            ball_impulse,
            ball_speed_change,
            impulse_away_alignment,
        );
        (event.confidence >= FLICK_MIN_CONFIDENCE).then_some(event)
    }
}

fn flick_event_from_parts(
    player: &PlayerSample,
    touch_event: &TouchEvent,
    dodge_start: &RecentDodgeStart,
    ball_impulse: glam::Vec3,
    ball_speed_change: f32,
    impulse_away_alignment: f32,
) -> FlickEvent {
    let time_since_dodge = touch_event.time - dodge_start.time;
    let vertical_impulse = ball_impulse.z.max(0.0);
    let setup = &dodge_start.setup;
    FlickEvent {
        time: touch_event.time,
        frame: touch_event.frame,
        sample_time: touch_event.time,
        sample_frame: touch_event.frame,
        player: player.player_id.clone(),
        is_team_0: player.is_team_0,
        dodge_time: dodge_start.time,
        dodge_frame: dodge_start.frame,
        time_since_dodge,
        setup_start_time: setup.start_time,
        setup_start_frame: setup.start_frame,
        setup_duration: setup.duration,
        setup_touch_count: setup.touch_count,
        average_horizontal_gap: setup.average_horizontal_gap,
        average_vertical_gap: setup.average_vertical_gap,
        ball_speed_change,
        ball_impulse: ball_impulse.to_array(),
        impulse_away_alignment,
        vertical_impulse,
        confidence: flick_event_confidence(
            setup,
            time_since_dodge,
            ball_speed_change,
            impulse_away_alignment,
            vertical_impulse,
        ),
    }
}
