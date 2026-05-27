use super::*;

pub(crate) fn touch_stats_event(
    frame: &FrameInfo,
    touch_event: &TouchEvent,
    player_id: &PlayerId,
    classification: TouchClassification,
    ball_speed_change: f32,
) -> TouchStatsEvent {
    TouchStatsEvent {
        time: touch_event.time,
        frame: touch_event.frame,
        sample_time: frame.time,
        sample_frame: frame.frame_number,
        player: player_id.clone(),
        is_team_0: touch_event.team_is_team_0,
        kind: classification.kind.as_label_value().to_owned(),
        height_band: classification.height_band.as_label().value.to_owned(),
        surface: classification.surface.as_label_value().to_owned(),
        dodge_state: classification.dodge_state.as_label_value().to_owned(),
        ball_speed_change,
    }
}
