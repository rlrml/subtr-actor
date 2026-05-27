use super::*;

impl ArmedWallAerial {
    pub(super) fn new(contact: RecentWallContact, frame: &FrameInfo, position: glam::Vec3) -> Self {
        Self {
            player: contact.player,
            is_team_0: contact.is_team_0,
            wall: contact.wall,
            wall_contact_time: contact.time,
            wall_contact_frame: contact.frame,
            wall_contact_position: contact.position,
            takeoff_time: frame.time,
            takeoff_frame: frame.frame_number,
            takeoff_position: position,
            controlled_setup: contact
                .controlled_setup
                .expect("validated wall contact should have controlled setup"),
            recorded: false,
        }
    }
}
