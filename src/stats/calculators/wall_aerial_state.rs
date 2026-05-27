use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct WallControl {
    pub(super) player_position: glam::Vec3,
    pub(super) ball_position: glam::Vec3,
    pub(super) wall: WallAerialWall,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ActiveWallControl {
    pub(super) player: PlayerId,
    pub(super) is_team_0: bool,
    pub(super) wall: WallAerialWall,
    pub(super) start_time: f32,
    pub(super) start_frame: usize,
    pub(super) last_time: f32,
    pub(super) last_frame: usize,
    pub(super) start_position: glam::Vec3,
    pub(super) last_position: glam::Vec3,
    pub(super) last_ball_position: glam::Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct RecentWallContact {
    pub(super) player: PlayerId,
    pub(super) is_team_0: bool,
    pub(super) wall: WallAerialWall,
    pub(super) time: f32,
    pub(super) frame: usize,
    pub(super) position: glam::Vec3,
    pub(super) controlled_setup: Option<CompletedWallSetup>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CompletedWallSetup {
    pub(super) start_time: f32,
    pub(super) start_frame: usize,
    pub(super) duration: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ArmedWallAerial {
    pub(super) player: PlayerId,
    pub(super) is_team_0: bool,
    pub(super) wall: WallAerialWall,
    pub(super) wall_contact_time: f32,
    pub(super) wall_contact_frame: usize,
    pub(super) wall_contact_position: glam::Vec3,
    pub(super) takeoff_time: f32,
    pub(super) takeoff_frame: usize,
    pub(super) takeoff_position: glam::Vec3,
    pub(super) controlled_setup: CompletedWallSetup,
    pub(super) recorded: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WallAerialCalculator {
    pub(super) player_stats: HashMap<PlayerId, WallAerialStats>,
    pub(super) events: Vec<WallAerialEvent>,
    pub(super) active_wall_controls: HashMap<PlayerId, ActiveWallControl>,
    pub(super) recent_wall_contacts: HashMap<PlayerId, RecentWallContact>,
    pub(super) armed_aerials: HashMap<PlayerId, ArmedWallAerial>,
    pub(super) recent_event_times: HashMap<PlayerId, f32>,
    pub(super) previous_ball_velocity: Option<glam::Vec3>,
    pub(super) current_last_wall_aerial_player: Option<PlayerId>,
}
