use super::*;

impl PlayerFrame {
    /// Builds a player frame from processor state at the requested replay time.
    pub(super) fn new_from_processor(
        processor: &dyn ProcessorView,
        player_id: &PlayerId,
        current_time: f32,
    ) -> SubtrActorResult<Self> {
        let rigid_body =
            processor.get_interpolated_player_rigid_body(player_id, current_time, 0.0)?;

        let boost_amount = processor.get_player_boost_level(player_id).unwrap_or(0.0);
        let boost_active = processor.get_boost_active(player_id).unwrap_or(0) % 2 == 1;
        let powerslide_active = processor.get_powerslide_active(player_id).unwrap_or(false);
        let jump_active = processor.get_jump_active(player_id).unwrap_or(0) % 2 == 1;
        let double_jump_active = processor.get_double_jump_active(player_id).unwrap_or(0) % 2 == 1;
        let dodge_active = processor.get_dodge_active(player_id).unwrap_or(0) % 2 == 1;

        // Extract player identity information
        let player_name = processor.get_player_name(player_id).ok();
        let team = processor
            .get_player_team_key(player_id)
            .ok()
            .and_then(|team_key| team_key.parse::<i32>().ok());
        let is_team_0 = processor.get_player_is_team_0(player_id).ok();

        Ok(Self::from_data(
            rigid_body,
            boost_amount,
            boost_active,
            powerslide_active,
            jump_active,
            double_jump_active,
            dodge_active,
            player_name,
            team,
            is_team_0,
        ))
    }

    /// Stores all player fields, including sleeping rigid bodies for kickoff/reset frames.
    #[allow(clippy::too_many_arguments)]
    fn from_data(
        rigid_body: boxcars::RigidBody,
        boost_amount: f32,
        boost_active: bool,
        powerslide_active: bool,
        jump_active: bool,
        double_jump_active: bool,
        dodge_active: bool,
        player_name: Option<String>,
        team: Option<i32>,
        is_team_0: Option<bool>,
    ) -> Self {
        Self::Data {
            rigid_body,
            boost_amount,
            boost_active,
            powerslide_active,
            jump_active,
            double_jump_active,
            dodge_active,
            player_name,
            team,
            is_team_0,
        }
    }
}
