use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FiftyFiftyEvent {
    pub start_time: f32,
    pub start_frame: usize,
    pub resolve_time: f32,
    pub resolve_frame: usize,
    pub is_kickoff: bool,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub team_zero_player: Option<PlayerId>,
    #[ts(as = "Option<crate::ts_bindings::RemoteIdTs>")]
    pub team_one_player: Option<PlayerId>,
    pub team_zero_touch_time: Option<f32>,
    pub team_zero_touch_frame: Option<usize>,
    pub team_zero_dodge_contact: bool,
    pub team_one_touch_time: Option<f32>,
    pub team_one_touch_frame: Option<usize>,
    pub team_one_dodge_contact: bool,
    pub team_zero_position: [f32; 3],
    pub team_one_position: [f32; 3],
    pub midpoint: [f32; 3],
    pub plane_normal: [f32; 3],
    pub winning_team_is_team_0: Option<bool>,
    pub possession_team_is_team_0: Option<bool>,
}

impl FiftyFiftyEvent {
    pub(super) fn labels(&self) -> Vec<StatLabel> {
        vec![
            fifty_fifty_phase_label(self.is_kickoff),
            fifty_fifty_team_outcome_label(self.winning_team_is_team_0),
            fifty_fifty_possession_label(self.possession_team_is_team_0),
            fifty_fifty_team_zero_dodge_state_label(self.team_zero_dodge_contact),
            fifty_fifty_team_one_dodge_state_label(self.team_one_dodge_contact),
        ]
    }

    pub(super) fn player_labels(&self, player_team_is_team_0: bool) -> Vec<StatLabel> {
        let dodge_contact = if player_team_is_team_0 {
            self.team_zero_dodge_contact
        } else {
            self.team_one_dodge_contact
        };
        vec![
            fifty_fifty_phase_label(self.is_kickoff),
            fifty_fifty_player_outcome_label(player_team_is_team_0, self.winning_team_is_team_0),
            fifty_fifty_player_possession_label(
                player_team_is_team_0,
                self.possession_team_is_team_0,
            ),
            fifty_fifty_touch_dodge_state_label(dodge_contact),
        ]
    }
}
