use super::*;

fn rigid_body(position: glam::Vec3, velocity: Option<glam::Vec3>) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: velocity.map(|velocity| glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

fn shot_metadata(is_team_0: bool) -> ShotEventMetadata {
    let target_y = if is_team_0 { 1200.0 } else { -1200.0 };
    let ball = rigid_body(
        glam::Vec3::new(10.0, target_y, 120.0),
        Some(glam::Vec3::new(0.0, 1800.0, 0.0)),
    );
    ShotEventMetadata::from_rigid_bodies(is_team_0, &ball, None)
}

fn player_stat_event(
    kind: PlayerStatEventKind,
    player: u64,
    is_team_0: bool,
    frame: usize,
) -> PlayerStatEvent {
    PlayerStatEvent {
        time: frame as f32 * 0.1,
        frame,
        player: boxcars::RemoteId::Steam(player),
        player_position: Some(glam_to_vec(&glam::Vec3::new(player as f32, 0.0, 17.0))),
        is_team_0,
        kind,
        shot: (kind == PlayerStatEventKind::Shot).then(|| shot_metadata(is_team_0)),
    }
}

#[test]
fn missing_metadata_i32_defaults_to_zero() {
    let missing_seconds = SubtrActorError::new(SubtrActorErrorVariant::PropertyNotFoundInState {
        property: SECONDS_REMAINING_KEY,
    });

    assert_eq!(metadata_i32_or_default(Err(missing_seconds)), 0);
}

#[test]
fn present_metadata_i32_is_preserved() {
    assert_eq!(metadata_i32_or_default(Ok(42)), 42);
}

#[test]
fn player_stat_events_with_shot_saves_links_opponent_save_to_latest_pending_shot() {
    let first_shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    let second_shot = player_stat_event(PlayerStatEventKind::Shot, 2, true, 11);
    let save = player_stat_event(PlayerStatEventKind::Save, 3, false, 12);

    let events = player_stat_events_with_shot_saves(&[first_shot, second_shot, save.clone()]);

    assert!(events[0].shot.as_ref().unwrap().resulting_save.is_none());
    let resulting_save = events[1]
        .shot
        .as_ref()
        .unwrap()
        .resulting_save
        .as_ref()
        .unwrap();
    assert_eq!(resulting_save.player, save.player);
    assert_eq!(resulting_save.frame, save.frame);
    assert_eq!(resulting_save.is_team_0, save.is_team_0);
    assert!(events[2].shot.is_none());
}

#[test]
fn player_stat_events_with_shot_saves_ignores_same_team_save() {
    let shot = player_stat_event(PlayerStatEventKind::Shot, 1, true, 10);
    let same_team_save = player_stat_event(PlayerStatEventKind::Save, 2, true, 11);

    let events = player_stat_events_with_shot_saves(&[shot, same_team_save]);

    assert!(events[0].shot.as_ref().unwrap().resulting_save.is_none());
}
