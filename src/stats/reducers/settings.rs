use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayerSettings {
    pub steering_sensitivity: Option<f32>,
    pub camera_fov: Option<f32>,
    pub camera_height: Option<f32>,
    pub camera_pitch: Option<f32>,
    pub camera_distance: Option<f32>,
    pub camera_stiffness: Option<f32>,
    pub camera_swivel_speed: Option<f32>,
    pub camera_transition_speed: Option<f32>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SettingsReducer {
    player_settings: HashMap<PlayerId, PlayerSettings>,
}

impl SettingsReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_settings(&self) -> &HashMap<PlayerId, PlayerSettings> {
        &self.player_settings
    }
}

impl StatsReducer for SettingsReducer {
    fn on_replay_meta(&mut self, meta: &ReplayMeta) -> SubtrActorResult<()> {
        for player in meta.player_order() {
            let Some(stats) = &player.stats else {
                continue;
            };
            self.player_settings.insert(
                player.remote_id.clone(),
                PlayerSettings {
                    steering_sensitivity: get_header_f32(
                        stats,
                        &["SteeringSensitivity", "SteerSensitivity"],
                    ),
                    camera_fov: get_header_f32(stats, &["CameraFOV"]),
                    camera_height: get_header_f32(stats, &["CameraHeight"]),
                    camera_pitch: get_header_f32(stats, &["CameraPitch"]),
                    camera_distance: get_header_f32(stats, &["CameraDistance"]),
                    camera_stiffness: get_header_f32(stats, &["CameraStiffness"]),
                    camera_swivel_speed: get_header_f32(stats, &["CameraSwivelSpeed"]),
                    camera_transition_speed: get_header_f32(stats, &["CameraTransitionSpeed"]),
                },
            );
        }
        Ok(())
    }

    fn on_sample(&mut self, _sample: &StatsSample) -> SubtrActorResult<()> {
        Ok(())
    }
}
