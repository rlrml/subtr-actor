use super::*;

const AIR_DRIBBLE_MIN_BALL_Z: f32 = 300.0;
pub(crate) const AIR_DRIBBLE_MIN_PLAYER_Z: f32 = 100.0;
pub(crate) const AIR_DRIBBLE_MIN_DURATION: f32 = 0.65;
const AIR_DRIBBLE_MIN_TOUCHES: u32 = 3;
const AIR_DRIBBLE_TOUCH_MAX_GAP_SECONDS: f32 = 3.0;
const WALL_TAKEOFF_MIN_Z: f32 = 120.0;
const SIDE_WALL_START_ABS_X: f32 = 3200.0;
const BACK_WALL_START_ABS_Y: f32 = 4600.0;

/// Where an air dribble originated (ground vs wall).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum AirDribbleOrigin {
    GroundToAir,
    WallToAir,
}

impl AirDribbleOrigin {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::GroundToAir => "ground_to_air",
            Self::WallToAir => "wall_to_air",
        }
    }
}

pub(crate) struct AirDribblePolicy;

impl AirDribblePolicy {
    pub(crate) fn is_air_touch_position(player_position: glam::Vec3) -> bool {
        player_position.z > PLAYER_GROUND_Z_THRESHOLD && !player_is_on_wall(player_position)
    }

    pub(crate) fn origin(start_position: glam::Vec3) -> AirDribbleOrigin {
        if start_position.z >= WALL_TAKEOFF_MIN_Z
            && (start_position.x.abs() >= SIDE_WALL_START_ABS_X
                || start_position.y.abs() >= BACK_WALL_START_ABS_Y)
        {
            AirDribbleOrigin::WallToAir
        } else {
            AirDribbleOrigin::GroundToAir
        }
    }
}

#[derive(Debug, Clone)]
struct ActiveAirDribble {
    player_id: PlayerId,
    is_team_0: bool,
    start_frame: usize,
    end_frame: usize,
    start_time: f32,
    end_time: f32,
    start_position: glam::Vec3,
    end_position: glam::Vec3,
    path_distance: f32,
    horizontal_gap_sum: f32,
    vertical_gap_sum: f32,
    touch_count: u32,
}

impl ActiveAirDribble {
    fn from_touch(touch: &TouchClassificationEvent) -> Option<Self> {
        let player_position = touch.player_position.map(glam::Vec3::from_array)?;
        let ball_position = touch.ball_position.map(glam::Vec3::from_array)?;
        let (horizontal_gap, vertical_gap) = touch_gaps(player_position, ball_position);
        Some(Self {
            player_id: touch.player.clone(),
            is_team_0: touch.is_team_0,
            start_frame: touch.frame,
            end_frame: touch.frame,
            start_time: touch.time,
            end_time: touch.time,
            start_position: player_position,
            end_position: player_position,
            path_distance: 0.0,
            horizontal_gap_sum: horizontal_gap,
            vertical_gap_sum: vertical_gap,
            touch_count: 1,
        })
    }

    fn extend(&mut self, touch: &TouchClassificationEvent) -> Option<()> {
        let player_position = touch.player_position.map(glam::Vec3::from_array)?;
        let ball_position = touch.ball_position.map(glam::Vec3::from_array)?;
        let (horizontal_gap, vertical_gap) = touch_gaps(player_position, ball_position);
        self.path_distance += player_position.distance(self.end_position);
        self.end_position = player_position;
        self.end_time = touch.time;
        self.end_frame = touch.frame;
        self.horizontal_gap_sum += horizontal_gap;
        self.vertical_gap_sum += vertical_gap;
        self.touch_count += 1;
        Some(())
    }

    fn event(self) -> Option<BallCarryEvent> {
        if self.touch_count < AIR_DRIBBLE_MIN_TOUCHES {
            return None;
        }
        let duration = self.end_time - self.start_time;
        if duration < AIR_DRIBBLE_MIN_DURATION {
            return None;
        }
        let average_speed = if duration > 0.0 {
            self.path_distance / duration
        } else {
            0.0
        };
        Some(BallCarryEvent {
            player_id: self.player_id,
            is_team_0: self.is_team_0,
            kind: BallCarryKind::AirDribble,
            start_position: self.start_position.to_array(),
            end_position: self.end_position.to_array(),
            start_frame: self.start_frame,
            end_frame: self.end_frame,
            start_time: self.start_time,
            end_time: self.end_time,
            duration,
            straight_line_distance: self
                .start_position
                .truncate()
                .distance(self.end_position.truncate()),
            path_distance: self.path_distance,
            average_horizontal_gap: self.horizontal_gap_sum / self.touch_count as f32,
            average_vertical_gap: self.vertical_gap_sum / self.touch_count as f32,
            average_speed,
            touch_count: self.touch_count,
            air_touch_count: self.touch_count,
            air_dribble_origin: Some(AirDribblePolicy::origin(self.start_position)),
        })
    }
}

fn touch_gaps(player_position: glam::Vec3, ball_position: glam::Vec3) -> (f32, f32) {
    (
        player_position
            .truncate()
            .distance(ball_position.truncate()),
        ball_position.z - player_position.z,
    )
}

fn is_air_dribble_touch(touch: &TouchClassificationEvent) -> bool {
    matches!(touch.tag("surface"), Some("air" | "wall"))
        && touch
            .ball_position
            .is_some_and(|position| position[2] >= AIR_DRIBBLE_MIN_BALL_Z)
}

/// Detects air dribbles from continuous same-player non-ground touches.
#[derive(Debug, Clone, Default)]
pub struct AirDribbleCalculator {
    events: EventStream<BallCarryEvent>,
    active: Option<ActiveAirDribble>,
    processed_touch_count: usize,
}

impl AirDribbleCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[BallCarryEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[BallCarryEvent] {
        self.events.new_events()
    }

    fn finish_active(&mut self) {
        if let Some(event) = self.active.take().and_then(ActiveAirDribble::event) {
            self.events.push(event);
        }
    }

    fn observe_touch(&mut self, touch: &TouchClassificationEvent) {
        if !is_air_dribble_touch(touch) {
            self.finish_active();
            return;
        }

        let same_sequence = self.active.as_ref().is_some_and(|active| {
            active.player_id == touch.player
                && active.is_team_0 == touch.is_team_0
                && touch.time - active.end_time <= AIR_DRIBBLE_TOUCH_MAX_GAP_SECONDS
        });
        if same_sequence {
            if self
                .active
                .as_mut()
                .and_then(|active| active.extend(touch))
                .is_none()
            {
                self.finish_active();
            }
            return;
        }

        self.finish_active();
        self.active = ActiveAirDribble::from_touch(touch);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        live_play: &LivePlayState,
        touch: &TouchCalculator,
    ) -> SubtrActorResult<()> {
        self.update_with_touch_classification_events(
            frame,
            ball,
            players,
            live_play,
            touch.events(),
        )
    }

    pub(crate) fn update_with_touch_classification_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        _players: &PlayerFrameState,
        live_play: &LivePlayState,
        touch_events: &[TouchClassificationEvent],
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play.is_live_play {
            self.finish_active();
            self.processed_touch_count = touch_events.len();
            return Ok(());
        }
        if ball
            .position()
            .is_none_or(|position| position.z < AIR_DRIBBLE_MIN_BALL_Z)
        {
            self.finish_active();
        }
        for touch in &touch_events[self.processed_touch_count..] {
            if touch.frame > frame.frame_number {
                break;
            }
            self.observe_touch(touch);
            self.processed_touch_count += 1;
        }
        Ok(())
    }

    pub fn finish(&mut self) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.finish_active();
        Ok(())
    }
}

#[cfg(test)]
#[path = "air_dribble_tests.rs"]
mod tests;
