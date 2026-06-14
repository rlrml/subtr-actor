use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Records a [`PlayerCameraStateChange`] for any player whose discrete
    /// camera toggles (ball cam, behind-view) or driving flag changed this
    /// frame.
    ///
    /// These flip only a handful of times per match, so rather than storing a
    /// value on every [`PlayerFrame`] we coalesce them into a per-player change
    /// stream. Each change carries the full discrete state so consumers resolve
    /// the value at an arbitrary frame with a last-change-before lookup. The
    /// underlying attributes are sticky in actor state (the camera-settings and
    /// vehicle actors retain their last value), so a player's state only
    /// transitions `None -> Some(..)` once and never flaps back to `None`.
    pub(crate) fn update_player_camera_events(
        &mut self,
        _frame: &boxcars::Frame,
        frame_index: usize,
    ) -> SubtrActorResult<()> {
        let player_ids: Vec<PlayerId> = self.iter_player_ids_in_order().cloned().collect();
        for player_id in player_ids {
            let current: PlayerCameraToggleState = (
                self.get_ball_cam_active(&player_id).ok(),
                self.get_behind_view_active(&player_id).ok(),
                self.get_driving(&player_id).ok(),
            );

            // Nothing replicated yet for this player; don't record empty changes.
            if current == (None, None, None) {
                continue;
            }

            if self.player_camera_state_last.get(&player_id) == Some(&current) {
                continue;
            }
            self.player_camera_state_last
                .insert(player_id.clone(), current);

            self.player_camera_events.push((
                player_id,
                PlayerCameraStateChange {
                    frame: frame_index,
                    ball_cam_active: current.0,
                    behind_view_active: current.1,
                    driving: current.2,
                },
            ));
        }

        Ok(())
    }
}
