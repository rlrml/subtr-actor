use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RecentWallContact {
    pub(super) player: PlayerId,
    pub(super) is_team_0: bool,
    pub(super) wall: WallAerialWall,
    pub(super) time: f32,
    pub(super) frame: usize,
    pub(super) position: glam::Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ArmedWallAerialShot {
    pub(super) player: PlayerId,
    pub(super) is_team_0: bool,
    pub(super) wall: WallAerialWall,
    pub(super) wall_contact_time: f32,
    pub(super) wall_contact_frame: usize,
    pub(super) wall_contact_position: glam::Vec3,
    pub(super) takeoff_time: f32,
    pub(super) takeoff_frame: usize,
    pub(super) takeoff_position: glam::Vec3,
}

impl RecentWallContact {
    pub(super) fn armed(
        self,
        frame: &FrameInfo,
        takeoff_position: glam::Vec3,
    ) -> ArmedWallAerialShot {
        ArmedWallAerialShot {
            player: self.player,
            is_team_0: self.is_team_0,
            wall: self.wall,
            wall_contact_time: self.time,
            wall_contact_frame: self.frame,
            wall_contact_position: self.position,
            takeoff_time: frame.time,
            takeoff_frame: frame.frame_number,
            takeoff_position,
        }
    }
}
