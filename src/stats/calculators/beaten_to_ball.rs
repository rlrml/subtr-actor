use super::*;

// Touch-anchored "beaten to ball" detection.
//
// These thresholds are evaluation-phase values: like the whiff detector, this
// stream feeds a human confirm/reject labeling loop, so the gates are tuned
// moderately toward recall. Precision is recovered downstream from the labels.
//
// The anchor is the *winner's* touch: at every confirmed touch we look back
// over each non-touching opponent's recent motion history and ask whether they
// were committed to the same ball and lost the race narrowly.

/// How much per-player history is retained, in seconds. Slightly longer than
/// the lookback so the window is fully populated when a touch lands.
const BEATEN_TO_BALL_HISTORY_SECONDS: f32 = 1.25;
/// Lookback window over which convergence and commitment are evaluated.
const BEATEN_TO_BALL_LOOKBACK_SECONDS: f32 = 1.0;
/// Minimum time span the lookback history must cover for the convergence
/// signals to be meaningful (avoids firing off one or two samples right after
/// a kickoff reset or a player re-appearing).
const BEATEN_TO_BALL_MIN_HISTORY_SECONDS: f32 = 0.4;
/// Minimum number of samples required in the lookback window.
const BEATEN_TO_BALL_MIN_HISTORY_SAMPLES: usize = 4;
/// Overall hitbox-distance decrease (uu) required across the lookback window
/// for the approach to count as sustained convergence.
const BEATEN_TO_BALL_MIN_DISTANCE_DECREASE: f32 = 200.0;
/// Fraction of lookback samples that must have positive closing speed toward
/// the ball.
const BEATEN_TO_BALL_MIN_CLOSING_FRACTION: f32 = 0.7;
/// Window (seconds, ending at the touch) over which velocity alignment toward
/// the ball is averaged.
const BEATEN_TO_BALL_ALIGNMENT_WINDOW_SECONDS: f32 = 0.4;
/// Minimum mean velocity alignment toward the ball over the recent window.
const BEATEN_TO_BALL_MIN_VELOCITY_ALIGNMENT: f32 = 0.45;
/// Minimum sustained approach speed (uu/s, mean of velocity projected onto the
/// to-ball direction over the recent window) for the commitment gate. A dodge
/// while closing on the ball inside the lookback window also satisfies
/// commitment.
const BEATEN_TO_BALL_MIN_APPROACH_SPEED: f32 = 400.0;
/// Maximum estimated time-to-ball (seconds) for the loss to count as narrow.
const BEATEN_TO_BALL_MAX_MARGIN_SECONDS: f32 = 0.75;
/// Alternative narrow-loss gate: being physically close at the touch counts
/// even if approach speed (and thus the margin estimate) is noisy.
const BEATEN_TO_BALL_NEAR_DISTANCE: f32 = 400.0;
/// Hard cap on hitbox distance to the ball at the touch frame.
const BEATEN_TO_BALL_MAX_DISTANCE_AT_TOUCH: f32 = 1200.0;
/// Rate limit: at most one event per losing player per this many seconds.
const BEATEN_TO_BALL_EVENT_COOLDOWN_SECONDS: f32 = 1.0;

/// An actively challenging player who was beaten to the ball by an opponent's
/// touch without getting a touch of their own.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct BeatenToBallEvent {
    /// Time of the winning touch.
    pub time: f32,
    /// Frame of the winning touch.
    pub frame: usize,
    /// The losing player (the one who was beaten).
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    /// The player whose touch anchored the event.
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub winner: PlayerId,
    /// The losing player's team.
    pub is_team_0: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    /// Loser's hitbox distance to the ball at the touch frame.
    pub distance_at_touch: f32,
    /// Estimated time-to-ball for the loser at the touch frame.
    pub margin_seconds: f32,
    /// Mean approach speed toward the ball over the recent window.
    pub approach_speed: f32,
    /// Mean velocity alignment toward the ball over the recent window.
    pub velocity_alignment: f32,
    /// Whether the loser dodged toward the ball within the lookback window.
    pub dodge_active: bool,
    /// Whether the loser was airborne at the touch frame.
    pub aerial: bool,
}

/// Why a candidate loser was not turned into a [`BeatenToBallEvent`] at an
/// opponent's touch. Variants carry the measured values that failed the gate so
/// diagnostics can report how close the candidate came.
///
/// The variants mirror the gate chain in `evaluate_loser` (in order), plus the
/// pre-gate exclusions applied in `evaluate_touches` before the gates run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BeatenToBallRejection {
    /// Pre-gate: the loser also touched the ball on this same frame.
    SameFrameTouch,
    /// Pre-gate: the loser touched the ball within the lookback window.
    RecentTouch { seconds_since_touch: f32 },
    /// Pre-gate: the per-player event cooldown suppressed re-emission.
    EventCooldown { seconds_since_event: f32 },
    /// No motion history existed for the loser at the touch frame.
    NoHistory,
    /// Fewer than the minimum number of samples in the lookback window.
    InsufficientHistorySamples { samples: usize },
    /// The lookback window covered too little time.
    InsufficientHistorySpan { span_seconds: f32 },
    /// Overall hitbox-distance decrease across the window was below threshold.
    NoDistanceDecrease {
        first_distance: f32,
        last_distance: f32,
    },
    /// Too few samples had positive closing speed toward the ball.
    ClosingFraction { fraction: f32 },
    /// No samples fell inside the recent alignment window.
    NoRecentSamples,
    /// Mean recent velocity alignment toward the ball was below threshold.
    Alignment { alignment: f32 },
    /// Neither sustained approach speed nor a dodge toward the ball.
    Commitment { approach_speed: f32 },
    /// Hitbox distance to the ball at the touch frame exceeded the hard cap.
    TooFarAtTouch { distance: f32 },
    /// Estimated loss margin was too wide and the loser was not near the ball.
    WideMargin { margin_seconds: f32, distance: f32 },
}

/// A rejected (or excluded) candidate loser recorded when diagnostics are
/// enabled via [`BeatenToBallCalculator::enable_diagnostics`].
#[derive(Debug, Clone, PartialEq)]
pub struct BeatenToBallDiagnostic {
    /// Time of the anchoring winning touch.
    pub time: f32,
    /// Frame of the anchoring winning touch.
    pub frame: usize,
    /// The candidate loser that was rejected.
    pub player: PlayerId,
    /// The player whose touch anchored the evaluation.
    pub winner: PlayerId,
    /// Which exclusion or gate rejected the candidate.
    pub rejection: BeatenToBallRejection,
}

/// One frame of a player's motion relative to the ball.
#[derive(Debug, Clone, Copy, PartialEq)]
struct MotionSample {
    time: f32,
    position: [f32; 3],
    /// Hitbox distance from the player's car to the ball.
    hitbox_distance: f32,
    /// Player velocity projected onto the to-ball direction.
    approach_speed: f32,
    /// Relative (player minus ball) velocity projected onto the to-ball
    /// direction.
    closing_speed: f32,
    /// Normalized player velocity dotted with the to-ball direction.
    velocity_alignment: f32,
    /// Whether the player's dodge byte was active while closing on the ball.
    dodge_toward_ball: bool,
}

/// Detects players who were actively challenging for the ball when an opponent
/// beat them to it. Retrospective: evaluated only at confirmed touches, so no
/// speculative in-flight ledger is needed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BeatenToBallCalculator {
    histories: HashMap<PlayerId, VecDeque<MotionSample>>,
    last_touch_times: HashMap<PlayerId, f32>,
    last_event_times: HashMap<PlayerId, f32>,
    events: EventStream<BeatenToBallEvent>,
    /// When true, every evaluated-but-rejected loser (including pre-gate
    /// exclusions) is recorded in `diagnostics`. Off by default so the normal
    /// path stays zero-overhead.
    diagnostics_enabled: bool,
    diagnostics: Vec<BeatenToBallDiagnostic>,
}

impl BeatenToBallCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[BeatenToBallEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[BeatenToBallEvent] {
        self.events.new_events()
    }

    /// Turns on rejection recording. Intended for offline audit tooling.
    pub fn enable_diagnostics(&mut self) {
        self.diagnostics_enabled = true;
    }

    /// All rejections recorded since diagnostics were enabled.
    pub fn diagnostics(&self) -> &[BeatenToBallDiagnostic] {
        &self.diagnostics
    }

    fn record_rejection(
        &mut self,
        frame: &FrameInfo,
        loser: &PlayerId,
        winner: &PlayerId,
        rejection: BeatenToBallRejection,
    ) {
        if !self.diagnostics_enabled {
            return;
        }
        self.diagnostics.push(BeatenToBallDiagnostic {
            time: frame.time,
            frame: frame.frame_number,
            player: loser.clone(),
            winner: winner.clone(),
            rejection,
        });
    }

    fn motion_sample(
        frame: &FrameInfo,
        ball_position: glam::Vec3,
        ball_velocity: glam::Vec3,
        player: &PlayerSample,
    ) -> Option<MotionSample> {
        let rigid_body = player.rigid_body.as_ref()?;
        let hitbox_distance = car_hitbox_distance(ball_position, rigid_body, player.hitbox)?;
        let player_position = player.position()?;
        let to_ball = (ball_position - player_position).normalize_or_zero();
        if to_ball.length_squared() <= f32::EPSILON {
            return None;
        }
        let player_velocity = player.velocity().unwrap_or(glam::Vec3::ZERO);
        let approach_speed = player_velocity.dot(to_ball);
        let closing_speed = (player_velocity - ball_velocity).dot(to_ball);
        let velocity_alignment = if player_velocity.length() <= f32::EPSILON {
            0.0
        } else {
            player_velocity.normalize_or_zero().dot(to_ball)
        };
        Some(MotionSample {
            time: frame.time,
            position: player_position.to_array(),
            hitbox_distance,
            approach_speed,
            closing_speed,
            velocity_alignment,
            dodge_toward_ball: player.dodge_active && closing_speed > 0.0,
        })
    }

    fn update_histories(
        &mut self,
        frame: &FrameInfo,
        ball_position: glam::Vec3,
        ball_velocity: glam::Vec3,
        players: &PlayerFrameState,
    ) {
        for player in &players.players {
            let Some(sample) = Self::motion_sample(frame, ball_position, ball_velocity, player)
            else {
                continue;
            };
            let history = self.histories.entry(player.player_id.clone()).or_default();
            history.push_back(sample);
            while history
                .front()
                .is_some_and(|front| frame.time - front.time > BEATEN_TO_BALL_HISTORY_SECONDS)
            {
                history.pop_front();
            }
        }
    }

    fn record_touch_times(&mut self, touch_state: &TouchState) {
        for touch in &touch_state.touch_events {
            if let Some(player) = touch.player.as_ref() {
                self.last_touch_times.insert(player.clone(), touch.time);
            }
        }
    }

    /// Evaluates a non-touching opponent's history against the convergence,
    /// commitment, and narrow-loss gates. Returns the event to emit, or the
    /// gate that rejected the candidate along with the measured value that
    /// failed it.
    fn evaluate_loser(
        &self,
        frame: &FrameInfo,
        loser: &PlayerSample,
        winner: &PlayerId,
    ) -> Result<BeatenToBallEvent, BeatenToBallRejection> {
        let history = self
            .histories
            .get(&loser.player_id)
            .ok_or(BeatenToBallRejection::NoHistory)?;
        let lookback_start = frame.time - BEATEN_TO_BALL_LOOKBACK_SECONDS;
        let window: Vec<&MotionSample> = history
            .iter()
            .filter(|sample| sample.time >= lookback_start)
            .collect();
        if window.len() < BEATEN_TO_BALL_MIN_HISTORY_SAMPLES {
            return Err(BeatenToBallRejection::InsufficientHistorySamples {
                samples: window.len(),
            });
        }
        let first = window.first().ok_or(BeatenToBallRejection::NoHistory)?;
        let last = window.last().ok_or(BeatenToBallRejection::NoHistory)?;
        if last.time - first.time < BEATEN_TO_BALL_MIN_HISTORY_SECONDS {
            return Err(BeatenToBallRejection::InsufficientHistorySpan {
                span_seconds: last.time - first.time,
            });
        }

        // Sustained convergence: overall distance decrease, mostly-positive
        // closing speed, and recent velocity alignment toward the ball.
        if first.hitbox_distance - last.hitbox_distance < BEATEN_TO_BALL_MIN_DISTANCE_DECREASE {
            return Err(BeatenToBallRejection::NoDistanceDecrease {
                first_distance: first.hitbox_distance,
                last_distance: last.hitbox_distance,
            });
        }
        let closing_count = window
            .iter()
            .filter(|sample| sample.closing_speed > 0.0)
            .count();
        if (closing_count as f32) < BEATEN_TO_BALL_MIN_CLOSING_FRACTION * window.len() as f32 {
            return Err(BeatenToBallRejection::ClosingFraction {
                fraction: closing_count as f32 / window.len() as f32,
            });
        }
        let recent_start = frame.time - BEATEN_TO_BALL_ALIGNMENT_WINDOW_SECONDS;
        let recent: Vec<&&MotionSample> = window
            .iter()
            .filter(|sample| sample.time >= recent_start)
            .collect();
        if recent.is_empty() {
            return Err(BeatenToBallRejection::NoRecentSamples);
        }
        let mean = |values: &[f32]| values.iter().sum::<f32>() / values.len() as f32;
        let recent_alignment = mean(
            &recent
                .iter()
                .map(|sample| sample.velocity_alignment)
                .collect::<Vec<_>>(),
        );
        if recent_alignment < BEATEN_TO_BALL_MIN_VELOCITY_ALIGNMENT {
            return Err(BeatenToBallRejection::Alignment {
                alignment: recent_alignment,
            });
        }

        // Commitment: sustained approach speed, or a dodge toward the ball
        // anywhere in the lookback window.
        let recent_approach_speed = mean(
            &recent
                .iter()
                .map(|sample| sample.approach_speed)
                .collect::<Vec<_>>(),
        );
        let dodged_toward_ball = window.iter().any(|sample| sample.dodge_toward_ball);
        if recent_approach_speed < BEATEN_TO_BALL_MIN_APPROACH_SPEED && !dodged_toward_ball {
            return Err(BeatenToBallRejection::Commitment {
                approach_speed: recent_approach_speed,
            });
        }

        // Narrow loss margin at the touch frame.
        let distance_at_touch = last.hitbox_distance;
        if distance_at_touch > BEATEN_TO_BALL_MAX_DISTANCE_AT_TOUCH {
            return Err(BeatenToBallRejection::TooFarAtTouch {
                distance: distance_at_touch,
            });
        }
        let margin_seconds = distance_at_touch / recent_approach_speed.max(f32::EPSILON);
        if margin_seconds > BEATEN_TO_BALL_MAX_MARGIN_SECONDS
            && distance_at_touch > BEATEN_TO_BALL_NEAR_DISTANCE
        {
            return Err(BeatenToBallRejection::WideMargin {
                margin_seconds,
                distance: distance_at_touch,
            });
        }

        Ok(BeatenToBallEvent {
            time: frame.time,
            frame: frame.frame_number,
            player: loser.player_id.clone(),
            winner: winner.clone(),
            is_team_0: loser.is_team_0,
            player_position: Some(last.position),
            distance_at_touch,
            margin_seconds,
            approach_speed: recent_approach_speed,
            velocity_alignment: recent_alignment,
            dodge_active: dodged_toward_ball,
            aerial: last.position[2] > POWERSLIDE_MAX_Z_THRESHOLD,
        })
    }

    fn evaluate_touches(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
        touch_state: &TouchState,
    ) {
        let touching_players: HashSet<&PlayerId> = touch_state
            .touch_events
            .iter()
            .filter_map(|touch| touch.player.as_ref())
            .collect();

        let mut emitted = Vec::new();
        for touch in &touch_state.touch_events {
            let Some(winner) = touch.player.as_ref() else {
                continue;
            };
            for loser in &players.players {
                if loser.is_team_0 == touch.team_is_team_0 {
                    continue;
                }
                if touching_players.contains(&loser.player_id) {
                    self.record_rejection(
                        frame,
                        &loser.player_id,
                        winner,
                        BeatenToBallRejection::SameFrameTouch,
                    );
                    continue;
                }
                if let Some(seconds_since_touch) = self
                    .last_touch_times
                    .get(&loser.player_id)
                    .map(|last_touch_time| frame.time - last_touch_time)
                    .filter(|seconds| *seconds <= BEATEN_TO_BALL_LOOKBACK_SECONDS)
                {
                    self.record_rejection(
                        frame,
                        &loser.player_id,
                        winner,
                        BeatenToBallRejection::RecentTouch {
                            seconds_since_touch,
                        },
                    );
                    continue;
                }
                if let Some(seconds_since_event) = self
                    .last_event_times
                    .get(&loser.player_id)
                    .map(|last_event_time| frame.time - last_event_time)
                    .filter(|seconds| *seconds < BEATEN_TO_BALL_EVENT_COOLDOWN_SECONDS)
                {
                    self.record_rejection(
                        frame,
                        &loser.player_id,
                        winner,
                        BeatenToBallRejection::EventCooldown {
                            seconds_since_event,
                        },
                    );
                    continue;
                }
                match self.evaluate_loser(frame, loser, winner) {
                    Ok(event) => {
                        self.last_event_times
                            .insert(loser.player_id.clone(), frame.time);
                        emitted.push(event);
                    }
                    Err(rejection) => {
                        self.record_rejection(frame, &loser.player_id, winner, rejection);
                    }
                }
            }
        }
        self.events.extend(emitted);
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play_state.is_live_play {
            self.histories.clear();
            self.last_touch_times.clear();
            self.last_event_times.clear();
            return Ok(());
        }
        if let Some(ball_position) = ball.position() {
            self.update_histories(
                frame,
                ball_position,
                ball.velocity().unwrap_or(glam::Vec3::ZERO),
                players,
            );
        }
        if !touch_state.touch_events.is_empty() {
            // Exclude anyone who touched recently, including this frame's
            // touchers, then evaluate every non-touching opponent against the
            // winner's touch. Recording this frame's touch times first is safe
            // because this frame's touchers are excluded via the touching set.
            self.record_touch_times(touch_state);
            self.evaluate_touches(frame, players, touch_state);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "beaten_to_ball_tests.rs"]
mod tests;
