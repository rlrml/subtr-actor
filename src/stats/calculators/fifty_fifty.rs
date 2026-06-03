use super::*;

pub(crate) const FIFTY_FIFTY_CONTINUATION_TOUCH_WINDOW_SECONDS: f32 = 0.2;
pub(crate) const FIFTY_FIFTY_RESOLUTION_DELAY_SECONDS: f32 = 0.35;
pub(crate) const FIFTY_FIFTY_MAX_DURATION_SECONDS: f32 = 1.25;
pub(crate) const FIFTY_FIFTY_MIN_EXIT_DISTANCE: f32 = 180.0;
pub(crate) const FIFTY_FIFTY_MIN_EXIT_SPEED: f32 = 220.0;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FiftyFiftyState {
    pub active_event: Option<ActiveFiftyFifty>,
    pub resolved_events: Vec<FiftyFiftyEvent>,
    pub last_resolved_event: Option<FiftyFiftyEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveFiftyFifty {
    pub start_time: f32,
    pub start_frame: usize,
    pub last_touch_time: f32,
    pub last_touch_frame: usize,
    pub is_kickoff: bool,
    pub team_zero_player: Option<PlayerId>,
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
}

impl ActiveFiftyFifty {
    pub fn midpoint_vec(&self) -> glam::Vec3 {
        glam::Vec3::from_array(self.midpoint)
    }

    pub fn plane_normal_vec(&self) -> glam::Vec3 {
        glam::Vec3::from_array(self.plane_normal)
    }

    pub fn contains_team_touch(&self, touch_events: &[TouchEvent]) -> bool {
        touch_events.iter().any(|touch| {
            (touch.team_is_team_0 && self.team_zero_player.is_some())
                || (!touch.team_is_team_0 && self.team_one_player.is_some())
        })
    }
}

#[cfg(test)]
impl FiftyFiftyCalculator {
    pub fn stats(&self) -> &FiftyFiftyStats {
        let mut stats = FiftyFiftyStatsAccumulator::default();
        for event in self.events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.stats().clone())
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, FiftyFiftyPlayerStats> {
        let mut stats = FiftyFiftyStatsAccumulator::default();
        for event in self.events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}

#[cfg(test)]
#[path = "fifty_fifty_tests.rs"]
mod tests;

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

pub(crate) const FIFTY_FIFTY_PHASE_LABELS: [StatLabel; 2] = [
    StatLabel::new("phase", "open_play"),
    StatLabel::new("phase", "kickoff"),
];
pub(crate) const FIFTY_FIFTY_TEAM_OUTCOME_LABELS: [StatLabel; 3] = [
    StatLabel::new("winning_team", "team_zero"),
    StatLabel::new("winning_team", "team_one"),
    StatLabel::new("winning_team", "neutral"),
];
pub(crate) const FIFTY_FIFTY_POSSESSION_LABELS: [StatLabel; 3] = [
    StatLabel::new("possession_after", "team_zero"),
    StatLabel::new("possession_after", "team_one"),
    StatLabel::new("possession_after", "neutral"),
];
pub(crate) const FIFTY_FIFTY_PLAYER_OUTCOME_LABELS: [StatLabel; 3] = [
    StatLabel::new("outcome", "win"),
    StatLabel::new("outcome", "loss"),
    StatLabel::new("outcome", "neutral"),
];
pub(crate) const FIFTY_FIFTY_PLAYER_POSSESSION_LABELS: [StatLabel; 3] = [
    StatLabel::new("possession_after", "self"),
    StatLabel::new("possession_after", "opponent"),
    StatLabel::new("possession_after", "neutral"),
];
pub(crate) const FIFTY_FIFTY_TOUCH_DODGE_STATE_LABELS: [StatLabel; 2] = [
    StatLabel::new("dodge_state", "no_dodge"),
    StatLabel::new("dodge_state", "dodge"),
];
pub(crate) const FIFTY_FIFTY_TEAM_ZERO_DODGE_STATE_LABELS: [StatLabel; 2] = [
    StatLabel::new("team_zero_dodge_state", "no_dodge"),
    StatLabel::new("team_zero_dodge_state", "dodge"),
];
pub(crate) const FIFTY_FIFTY_TEAM_ONE_DODGE_STATE_LABELS: [StatLabel; 2] = [
    StatLabel::new("team_one_dodge_state", "no_dodge"),
    StatLabel::new("team_one_dodge_state", "dodge"),
];

pub(crate) fn fifty_fifty_phase_label(is_kickoff: bool) -> StatLabel {
    if is_kickoff {
        StatLabel::new("phase", "kickoff")
    } else {
        StatLabel::new("phase", "open_play")
    }
}

pub(crate) fn fifty_fifty_team_outcome_label(team_is_team_0: Option<bool>) -> StatLabel {
    match team_is_team_0 {
        Some(true) => StatLabel::new("winning_team", "team_zero"),
        Some(false) => StatLabel::new("winning_team", "team_one"),
        None => StatLabel::new("winning_team", "neutral"),
    }
}

pub(crate) fn fifty_fifty_possession_label(team_is_team_0: Option<bool>) -> StatLabel {
    match team_is_team_0 {
        Some(true) => StatLabel::new("possession_after", "team_zero"),
        Some(false) => StatLabel::new("possession_after", "team_one"),
        None => StatLabel::new("possession_after", "neutral"),
    }
}

pub(crate) fn fifty_fifty_player_outcome_label(
    player_team_is_team_0: bool,
    winning_team_is_team_0: Option<bool>,
) -> StatLabel {
    match winning_team_is_team_0 {
        Some(team_is_team_0) if team_is_team_0 == player_team_is_team_0 => {
            StatLabel::new("outcome", "win")
        }
        Some(_) => StatLabel::new("outcome", "loss"),
        None => StatLabel::new("outcome", "neutral"),
    }
}

pub(crate) fn fifty_fifty_player_possession_label(
    player_team_is_team_0: bool,
    possession_team_is_team_0: Option<bool>,
) -> StatLabel {
    match possession_team_is_team_0 {
        Some(team_is_team_0) if team_is_team_0 == player_team_is_team_0 => {
            StatLabel::new("possession_after", "self")
        }
        Some(_) => StatLabel::new("possession_after", "opponent"),
        None => StatLabel::new("possession_after", "neutral"),
    }
}

pub(crate) fn fifty_fifty_touch_dodge_state_label(dodge_contact: bool) -> StatLabel {
    if dodge_contact {
        StatLabel::new("dodge_state", "dodge")
    } else {
        StatLabel::new("dodge_state", "no_dodge")
    }
}

pub(crate) fn fifty_fifty_team_zero_dodge_state_label(dodge_contact: bool) -> StatLabel {
    if dodge_contact {
        StatLabel::new("team_zero_dodge_state", "dodge")
    } else {
        StatLabel::new("team_zero_dodge_state", "no_dodge")
    }
}

pub(crate) fn fifty_fifty_team_one_dodge_state_label(dodge_contact: bool) -> StatLabel {
    if dodge_contact {
        StatLabel::new("team_one_dodge_state", "dodge")
    } else {
        StatLabel::new("team_one_dodge_state", "no_dodge")
    }
}

impl FiftyFiftyEvent {
    pub(crate) fn labels(&self) -> Vec<StatLabel> {
        vec![
            fifty_fifty_phase_label(self.is_kickoff),
            fifty_fifty_team_outcome_label(self.winning_team_is_team_0),
            fifty_fifty_possession_label(self.possession_team_is_team_0),
            fifty_fifty_team_zero_dodge_state_label(self.team_zero_dodge_contact),
            fifty_fifty_team_one_dodge_state_label(self.team_one_dodge_contact),
        ]
    }

    pub(crate) fn player_labels(&self, player_team_is_team_0: bool) -> Vec<StatLabel> {
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FiftyFiftyCalculator {
    events: EventStream<FiftyFiftyEvent>,
}

impl FiftyFiftyCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[FiftyFiftyEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[FiftyFiftyEvent] {
        self.events.new_events()
    }

    fn apply_event(&mut self, event: &FiftyFiftyEvent) {
        self.events.push(event.clone());
    }

    pub(crate) fn kickoff_phase_active(gameplay: &GameplayState) -> bool {
        gameplay.kickoff_phase_active()
    }

    pub(crate) fn contested_touch(
        frame: &FrameInfo,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
        is_kickoff: bool,
    ) -> Option<ActiveFiftyFifty> {
        let team_zero_touch = touch_events.iter().find(|touch| touch.team_is_team_0)?;
        let team_one_touch = touch_events.iter().find(|touch| !touch.team_is_team_0)?;
        let team_zero_position = team_zero_touch.player.as_ref().and_then(|player_id| {
            players
                .players
                .iter()
                .find(|player| &player.player_id == player_id)
                .and_then(PlayerSample::position)
        })?;
        let team_one_position = team_one_touch.player.as_ref().and_then(|player_id| {
            players
                .players
                .iter()
                .find(|player| &player.player_id == player_id)
                .and_then(PlayerSample::position)
        })?;
        let midpoint = (team_zero_position + team_one_position) * 0.5;
        let mut plane_normal = team_one_position - team_zero_position;
        plane_normal.z = 0.0;
        if plane_normal.length_squared() <= f32::EPSILON {
            plane_normal = glam::Vec3::Y;
        } else {
            plane_normal = plane_normal.normalize();
        }

        Some(ActiveFiftyFifty {
            start_time: frame.time,
            start_frame: frame.frame_number,
            last_touch_time: frame.time,
            last_touch_frame: frame.frame_number,
            is_kickoff,
            team_zero_player: team_zero_touch.player.clone(),
            team_one_player: team_one_touch.player.clone(),
            team_zero_touch_time: Some(team_zero_touch.time),
            team_zero_touch_frame: Some(team_zero_touch.frame),
            team_zero_dodge_contact: team_zero_touch.dodge_contact,
            team_one_touch_time: Some(team_one_touch.time),
            team_one_touch_frame: Some(team_one_touch.frame),
            team_one_dodge_contact: team_one_touch.dodge_contact,
            team_zero_position: team_zero_position.to_array(),
            team_one_position: team_one_position.to_array(),
            midpoint: midpoint.to_array(),
            plane_normal: plane_normal.to_array(),
        })
    }

    pub(crate) fn winning_team_from_ball(
        active: &ActiveFiftyFifty,
        ball: &BallFrameState,
    ) -> Option<bool> {
        let ball = ball.sample()?;
        let midpoint = active.midpoint_vec();
        let plane_normal = active.plane_normal_vec();
        let displacement = ball.position() - midpoint;
        let signed_distance = displacement.dot(plane_normal);
        if signed_distance.abs() >= FIFTY_FIFTY_MIN_EXIT_DISTANCE {
            return Some(signed_distance > 0.0);
        }

        let signed_speed = ball.velocity().dot(plane_normal);
        if signed_speed.abs() >= FIFTY_FIFTY_MIN_EXIT_SPEED {
            return Some(signed_speed > 0.0);
        }

        None
    }

    pub fn update(&mut self, fifty_fifty_state: &FiftyFiftyState) -> SubtrActorResult<()> {
        self.events.begin_update();
        for event in &fifty_fifty_state.resolved_events {
            self.apply_event(event);
        }
        Ok(())
    }
}
