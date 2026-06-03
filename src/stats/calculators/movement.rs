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
    #[ts(as = "crate::ts_bindings::RemoteIdTs")]
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

#[derive(Debug, Clone, Default)]
pub struct MovementCalculator {
    player_teams: HashMap<PlayerId, bool>,
    previous_positions: HashMap<PlayerId, glam::Vec3>,
    events: EventStream<MovementEvent>,
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
                continue;
            };
            let speed = player.speed().unwrap_or(0.0);
            if live_play {
                let distance = if let Some(previous_position) =
                    self.previous_positions.get(&player.player_id)
                {
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
                    player: player.player_id.clone(),
                    player_position: Some(position.to_array()),
                    is_team_0: player.is_team_0,
                    dt: frame.dt,
                    speed,
                    distance,
                    speed_band: classification.speed_band.as_label_value().to_owned(),
                    height_band: classification.height_band.as_label().value.to_owned(),
                };
                self.events.push(event);
            }

            self.previous_positions
                .insert(player.player_id.clone(), position);
        }

        Ok(())
    }
}
