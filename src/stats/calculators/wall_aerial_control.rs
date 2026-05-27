use super::*;
use wall_aerial_wall::wall_aerial_setup_wall_for_position;

impl WallAerialCalculator {
    pub(super) fn control_observation(
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
    ) -> Option<(PlayerId, bool, WallControl)> {
        let player_id = touch_state.last_touch_player.as_ref()?;
        let ball_position = ball.position()?;
        let player = players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)?;
        let player_position = player.position()?;
        let wall = wall_aerial_setup_wall_for_position(player_position)?;
        if player_position.distance(ball_position) > WALL_AERIAL_MAX_CONTROL_BALL_DISTANCE {
            return None;
        }

        Some((
            player_id.clone(),
            player.is_team_0,
            WallControl {
                player_position,
                ball_position,
                wall,
            },
        ))
    }

    pub(super) fn update_active_wall_control(
        &mut self,
        frame: &FrameInfo,
        control: Option<(PlayerId, bool, WallControl)>,
    ) {
        let Some((player_id, is_team_0, control)) = control else {
            self.active_wall_controls.clear();
            return;
        };

        self.active_wall_controls
            .retain(|active_player, _| active_player == &player_id);
        if self.active_wall_control_continues(&player_id, control, frame) {
            return;
        }
        self.active_wall_controls.insert(
            player_id.clone(),
            ActiveWallControl::new(player_id, is_team_0, control, frame),
        );
    }

    fn active_wall_control_continues(
        &mut self,
        player_id: &PlayerId,
        control: WallControl,
        frame: &FrameInfo,
    ) -> bool {
        let Some(active) = self.active_wall_controls.get_mut(player_id) else {
            return false;
        };
        if active.wall != control.wall {
            return false;
        }
        active.last_time = frame.time;
        active.last_frame = frame.frame_number;
        active.last_position = control.player_position;
        active.last_ball_position = control.ball_position;
        true
    }
}

impl ActiveWallControl {
    fn new(player: PlayerId, is_team_0: bool, control: WallControl, frame: &FrameInfo) -> Self {
        Self {
            player,
            is_team_0,
            wall: control.wall,
            start_time: frame.time,
            start_frame: frame.frame_number,
            last_time: frame.time,
            last_frame: frame.frame_number,
            start_position: control.player_position,
            last_position: control.player_position,
            last_ball_position: control.ball_position,
        }
    }
}
