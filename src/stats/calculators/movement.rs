use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MovementSpeedBand {
    Slow,
    Boost,
    Supersonic,
}

impl MovementSpeedBand {
    fn as_label_value(self) -> &'static str {
        match self {
            Self::Slow => "slow",
            Self::Boost => "boost",
            Self::Supersonic => "supersonic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MovementClassification {
    speed_band: MovementSpeedBand,
    height_band: PlayerVerticalBand,
}

#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct MovementEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub dt: f32,
    pub speed: f32,
    pub distance: f32,
    pub speed_band: String,
    pub height_band: String,
}

impl MovementEvent {
    fn absorb_sample(&mut self, sample: Self) {
        self.end_time = sample.time;
        self.end_frame = sample.frame;
        self.player_position = sample.player_position;
        let combined_dt = self.dt + sample.dt;
        if combined_dt > 0.0 {
            self.speed = (self.speed * self.dt + sample.speed * sample.dt) / combined_dt;
        }
        self.dt = combined_dt;
        self.distance += sample.distance;
    }
}

#[derive(Debug, Clone, Default)]
pub struct MovementCalculator {
    player_teams: HashMap<PlayerId, bool>,
    previous_positions: HashMap<PlayerId, glam::Vec3>,
    events: EventStream<MovementEvent>,
    pending_events: HashMap<PlayerId, PendingMovementEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MovementEventState {
    is_team_0: bool,
    classification: MovementClassification,
}

#[derive(Debug, Clone, PartialEq)]
struct PendingMovementEvent {
    state: MovementEventState,
    event: MovementEvent,
}

impl MovementCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[MovementEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[MovementEvent] {
        self.events.new_events()
    }

    pub fn projected_events(&self) -> Vec<MovementEvent> {
        let mut events = self.events.all().to_vec();
        let mut pending: Vec<_> = self
            .pending_events
            .values()
            .map(|pending| pending.event.clone())
            .collect();
        pending.sort_by(|left, right| {
            left.frame
                .cmp(&right.frame)
                .then_with(|| format!("{:?}", left.player).cmp(&format!("{:?}", right.player)))
        });
        events.extend(pending);
        events
    }

    pub fn flush_pending_events(&mut self) {
        let mut pending: Vec<_> = self
            .pending_events
            .drain()
            .map(|(_, pending)| pending)
            .collect();
        pending.sort_by(|left, right| {
            left.event.frame.cmp(&right.event.frame).then_with(|| {
                format!("{:?}", left.event.player).cmp(&format!("{:?}", right.event.player))
            })
        });
        self.events
            .extend(pending.into_iter().map(|pending| pending.event));
    }

    fn classify_movement(speed: f32, height_band: PlayerVerticalBand) -> MovementClassification {
        let speed_band = if speed >= SUPERSONIC_SPEED_THRESHOLD {
            MovementSpeedBand::Supersonic
        } else if speed >= BOOST_SPEED_THRESHOLD {
            MovementSpeedBand::Boost
        } else {
            MovementSpeedBand::Slow
        };

        MovementClassification {
            speed_band,
            height_band,
        }
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        vertical_state: &PlayerVerticalState,
        live_play: bool,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play {
            self.flush_pending_events();
            for player in &players.players {
                if let Some(position) = player.position() {
                    self.previous_positions
                        .insert(player.player_id.clone(), position);
                }
            }
            return Ok(());
        }

        let active_players: HashSet<_> = players
            .players
            .iter()
            .map(|player| player.player_id.clone())
            .collect();
        self.flush_pending_events_for_missing_players(&active_players);

        if frame.dt == 0.0 {
            for player in &players.players {
                if let Some(position) = player.position() {
                    self.previous_positions
                        .insert(player.player_id.clone(), position);
                }
            }
            return Ok(());
        }

        for player in &players.players {
            self.player_teams
                .insert(player.player_id.clone(), player.is_team_0);
            let Some(position) = player.position() else {
                self.flush_pending_event_for_player(&player.player_id);
                continue;
            };
            let speed = player.speed().unwrap_or(0.0);
            let distance =
                if let Some(previous_position) = self.previous_positions.get(&player.player_id) {
                    position.distance(*previous_position)
                } else {
                    0.0
                };

            let height_band = vertical_state
                .band_for_player(&player.player_id)
                .unwrap_or_else(|| PlayerVerticalBand::from_height(position.z));
            let classification = Self::classify_movement(speed, height_band);
            let event = MovementEvent {
                time: frame.time,
                frame: frame.frame_number,
                end_time: frame.time,
                end_frame: frame.frame_number,
                player: player.player_id.clone(),
                player_position: Some(position.to_array()),
                is_team_0: player.is_team_0,
                dt: frame.dt,
                speed,
                distance,
                speed_band: classification.speed_band.as_label_value().to_owned(),
                height_band: classification.height_band.as_label().value.to_owned(),
            };
            self.record_event(
                MovementEventState {
                    is_team_0: player.is_team_0,
                    classification,
                },
                event,
            );

            self.previous_positions
                .insert(player.player_id.clone(), position);
        }

        Ok(())
    }

    fn record_event(&mut self, state: MovementEventState, event: MovementEvent) {
        let player = event.player.clone();
        let Some(pending) = self.pending_events.get_mut(&player) else {
            self.pending_events
                .insert(player, PendingMovementEvent { state, event });
            return;
        };

        if pending.state == state {
            pending.event.absorb_sample(event);
        } else {
            let previous = self
                .pending_events
                .insert(player, PendingMovementEvent { state, event });
            let Some(previous) = previous else {
                return;
            };
            self.events.push(previous.event);
        }
    }

    fn flush_pending_event_for_player(&mut self, player_id: &PlayerId) {
        let Some(pending) = self.pending_events.remove(player_id) else {
            return;
        };
        self.events.push(pending.event);
    }

    fn flush_pending_events_for_missing_players(&mut self, active_players: &HashSet<PlayerId>) {
        let missing_players: Vec<_> = self
            .pending_events
            .keys()
            .filter(|player_id| !active_players.contains(*player_id))
            .cloned()
            .collect();
        for player_id in missing_players {
            self.flush_pending_event_for_player(&player_id);
        }
    }
}
