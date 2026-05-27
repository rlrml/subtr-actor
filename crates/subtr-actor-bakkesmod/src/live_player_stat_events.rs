use super::*;

pub(crate) fn explicit_player_stat_events(
    frame: &FrameInfo,
    events: &[SaPlayerStatEvent],
) -> Vec<PlayerStatEvent> {
    events
        .iter()
        .map(|event| {
            let (frame_number, time) = event_frame_and_time(frame, event.timing);
            PlayerStatEvent {
                time,
                frame: frame_number,
                player: player_id(event.player_index),
                is_team_0: event.is_team_0 != 0,
                kind: match event.kind {
                    SaPlayerStatEventKind::Shot => PlayerStatEventKind::Shot,
                    SaPlayerStatEventKind::Save => PlayerStatEventKind::Save,
                    SaPlayerStatEventKind::Assist => PlayerStatEventKind::Assist,
                },
                shot: shot_event_metadata(event),
            }
        })
        .collect()
}

pub(crate) fn shot_event_metadata(event: &SaPlayerStatEvent) -> Option<ShotEventMetadata> {
    if event.kind != SaPlayerStatEventKind::Shot || event.has_shot_ball == 0 {
        return None;
    }

    let ball_body = rigid_body(event.shot_ball);
    let player_body = (event.has_shot_player != 0).then(|| rigid_body(event.shot_player));
    Some(ShotEventMetadata::from_rigid_bodies(
        event.is_team_0 != 0,
        &ball_body,
        player_body.as_ref(),
    ))
}
