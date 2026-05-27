use super::*;

pub(super) fn demo_event_sample_json(sample: &DemoEventSample) -> Value {
    json!({
        "attacker": sample.attacker,
        "victim": sample.victim,
    })
}

fn vertical_band_label(band: PlayerVerticalBand) -> &'static str {
    match band {
        PlayerVerticalBand::Ground => "ground",
        PlayerVerticalBand::LowAir => "low_air",
        PlayerVerticalBand::HighAir => "high_air",
    }
}

pub(super) fn player_vertical_state_json(state: &PlayerVerticalState) -> Value {
    let mut players = state
        .players
        .iter()
        .map(|(player_id, sample)| {
            json!({
                "player_id": player_id,
                "height": sample.height,
                "band": vertical_band_label(sample.band),
            })
        })
        .collect::<Vec<_>>();
    players.sort_by_key(|value| value["player_id"].to_string());
    json!({ "players": players })
}

pub(super) fn settings_json(calculator: &SettingsCalculator) -> Value {
    let mut player_settings = calculator
        .player_settings()
        .iter()
        .map(|(player_id, settings)| {
            json!({
                "player_id": player_id,
                "settings": {
                    "steering_sensitivity": settings.steering_sensitivity,
                    "camera_fov": settings.camera_fov,
                    "camera_height": settings.camera_height,
                    "camera_pitch": settings.camera_pitch,
                    "camera_distance": settings.camera_distance,
                    "camera_stiffness": settings.camera_stiffness,
                    "camera_swivel_speed": settings.camera_swivel_speed,
                    "camera_transition_speed": settings.camera_transition_speed,
                },
            })
        })
        .collect::<Vec<_>>();
    player_settings.sort_by_key(|value| value["player_id"].to_string());
    json!({ "player_settings": player_settings })
}
