use super::*;

const GOAL_LINE_Y: f32 = 5120.0;
const GOAL_MOUTH_HEIGHT_Z: f32 = 642.775;
const GOAL_MOUTH_TRAJECTORY_MARGIN: f32 = BALL_RADIUS_Z * 1.5;
/// A post-touch trajectory must cross the opponent goal mouth within this many
/// seconds for the touch to read as a shot.
const SHOT_MAX_TIME_TO_GOAL_SECONDS: f32 = 2.5;
const SHOT_MIN_BALL_SPEED: f32 = 1000.0;
/// The pre-touch trajectory must have been crossing the toucher's own goal
/// mouth within this many seconds for the touch to read as a save.
const SAVE_MAX_TIME_TO_GOAL_SECONDS: f32 = 2.0;
const SAVE_MIN_INBOUND_BALL_SPEED: f32 = 250.0;
/// Window for matching a replay-reported shot/save stat event to a touch. Stat
/// events are matched looking backward only; the touch sample itself can lag
/// the stat event by a few frames because of touch-candidate scoring.
const STAT_EVENT_MATCH_WINDOW_SECONDS: f32 = 0.75;
/// Clears must start inside the toucher's defensive third.
const CLEAR_MAX_ATTACKING_Y: f32 = -GOAL_LINE_Y / 3.0;
const CLEAR_MIN_BALL_SPEED: f32 = 1300.0;
const CLEAR_MIN_AWAY_FROM_OWN_GOAL_ALIGNMENT: f32 = 0.2;
const PASS_MIN_BALL_SPEED: f32 = 500.0;
const PASS_MIN_LEAD_SECONDS: f32 = 0.15;
const PASS_MAX_LEAD_SECONDS: f32 = 2.5;
const PASS_RECEIVER_MAX_DISTANCE: f32 = 800.0;
const PASS_MIN_TRAVEL_DISTANCE: f32 = 500.0;
/// A touch starts a new reception (a "first touch") when the previous touch by
/// anyone was either by a different player or this long ago.
const FIRST_TOUCH_RESET_SECONDS: f32 = 2.5;
/// How long after a touch the control-follow window watches whether the
/// toucher stays with the ball.
const CONTROL_FOLLOW_WINDOW_SECONDS: f32 = 1.25;
/// A follow frame counts as controlled only while the toucher is at most this
/// far from the ball.
const CONTROL_FOLLOW_MAX_DISTANCE: f32 = 600.0;
/// A follow frame counts as controlled only while the toucher's velocity
/// roughly matches the ball's.
const CONTROL_FOLLOW_MAX_RELATIVE_SPEED: f32 = 800.0;
/// Minimum follow time before a window can resolve as control on the
/// stay-close criterion, so a window cut short (goal, stoppage) keeps its
/// provisional intention instead of fake-confirming control.
const CONTROL_FOLLOW_MIN_TRACKED_SECONDS: f32 = 0.4;
/// Fraction of the tracked follow time that must be controlled for the touch
/// to resolve as control without a follow-up touch.
const CONTROL_FOLLOW_MIN_CONTROLLED_FRACTION: f32 = 0.7;

/// The primary inferred intention of a ball touch.
///
/// Intentions are mutually exclusive; overlaps are resolved by a precedence
/// ladder (see [`TouchIntentionClassifier::classify`]). Contested and
/// first-touch context is preserved separately on
/// [`TouchIntentionResolution`] so no information is lost to the precedence
/// choice.
///
/// `Control` is never assigned at touch time: it is an outcome-based upgrade
/// applied retroactively by [`ControlFollowTracker`] when the toucher stays
/// with the ball after the touch or earns the follow-up touch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchIntention {
    Control,
    Shot,
    Save,
    Challenge,
    Clear,
    Pass,
    Neutral,
}

impl TouchIntention {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Shot => "shot",
            Self::Save => "save",
            Self::Challenge => "challenge",
            Self::Clear => "clear",
            Self::Pass => "pass",
            Self::Neutral => "neutral",
        }
    }
}

/// The resolved touch intention with supporting context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchIntentionResolution {
    pub intention: TouchIntention,
    pub first_touch: bool,
    pub contested: bool,
}

/// Per-frame context a touch intention classification draws on.
///
/// Ball state is split into pre-touch (previous frame) and post-touch (current
/// frame) samples: saves read the inbound trajectory, while shots, clears, and
/// passes read the outbound one.
pub struct TouchIntentionFrameContext<'a> {
    pub ball_position: Option<glam::Vec3>,
    pub ball_velocity: Option<glam::Vec3>,
    pub previous_ball_position: Option<glam::Vec3>,
    pub previous_ball_velocity: Option<glam::Vec3>,
    /// Positions of the toucher's teammates (excluding the toucher).
    pub teammate_positions: &'a [glam::Vec3],
    pub contested: bool,
}

/// The reception currently in progress: who most recently took clean
/// possession of the ball, and when the ball was last touched during it.
#[derive(Debug, Clone, PartialEq)]
struct Reception {
    player: PlayerId,
    last_touch_time: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct RecentStatEvent {
    kind: PlayerStatEventKind,
    player: PlayerId,
    time: f32,
}

/// Stateful helper that classifies the intention of each touch.
///
/// Holds the small amount of cross-frame state classification needs: the
/// reception in progress (for first-touch detection) and a short backward
/// window of replay-reported shot/save stat events (for replay-confirmed
/// classification).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TouchIntentionClassifier {
    reception: Option<Reception>,
    recent_stat_events: VecDeque<RecentStatEvent>,
}

impl TouchIntentionClassifier {
    /// Clear cross-frame state at a live-play boundary so each kickoff starts
    /// a fresh reception sequence.
    pub fn reset(&mut self) {
        self.reception = None;
        self.recent_stat_events.clear();
    }

    /// Ingest this frame's replay-reported stat events and drop ones that have
    /// aged out of the matching window.
    pub fn begin_frame(&mut self, frame: &FrameInfo, player_stat_events: &[PlayerStatEvent]) {
        for event in player_stat_events {
            match event.kind {
                PlayerStatEventKind::Shot | PlayerStatEventKind::Save => {
                    self.recent_stat_events.push_back(RecentStatEvent {
                        kind: event.kind,
                        player: event.player.clone(),
                        time: event.time,
                    });
                }
                PlayerStatEventKind::Assist => {}
            }
        }
        while self
            .recent_stat_events
            .front()
            .is_some_and(|event| frame.time - event.time > STAT_EVENT_MATCH_WINDOW_SECONDS)
        {
            self.recent_stat_events.pop_front();
        }
    }

    /// Classify one touch and advance first-touch tracking.
    ///
    /// Touches must be supplied in chronological order. The precedence ladder:
    /// replay-confirmed saves and shots first (game-authoritative), then
    /// contested challenges, then trajectory-based saves, shots, clears, and
    /// passes, then neutral. The result is provisional for pass and neutral
    /// touches: a [`ControlFollowTracker`] window may retroactively upgrade
    /// them to control once the post-touch outcome is known. The higher rungs
    /// are never upgraded, so control excludes them by construction.
    pub fn classify(
        &mut self,
        touch: &TouchEvent,
        player_id: &PlayerId,
        ctx: &TouchIntentionFrameContext,
    ) -> TouchIntentionResolution {
        let first_touch = self.is_first_touch(player_id, touch.time);
        let is_team_0 = touch.team_is_team_0;

        let intention =
            if self.has_matching_stat_event(PlayerStatEventKind::Save, player_id, touch.time) {
                TouchIntention::Save
            } else if self.has_matching_stat_event(PlayerStatEventKind::Shot, player_id, touch.time)
            {
                TouchIntention::Shot
            } else if ctx.contested {
                TouchIntention::Challenge
            } else if Self::is_geometric_save(ctx, is_team_0) {
                TouchIntention::Save
            } else if Self::is_geometric_shot(ctx, is_team_0) {
                TouchIntention::Shot
            } else if Self::is_clear(ctx, is_team_0) {
                TouchIntention::Clear
            } else if Self::is_pass(ctx) {
                TouchIntention::Pass
            } else {
                TouchIntention::Neutral
            };

        self.note_touch(player_id, touch.time, ctx.contested);

        TouchIntentionResolution {
            intention,
            first_touch,
            contested: ctx.contested,
        }
    }

    fn is_first_touch(&self, player_id: &PlayerId, time: f32) -> bool {
        match self.reception.as_ref() {
            None => true,
            Some(reception) => {
                reception.player != *player_id
                    || (time - reception.last_touch_time) > FIRST_TOUCH_RESET_SECONDS
            }
        }
    }

    /// Advance reception tracking past this touch.
    ///
    /// Uncontested touches claim the reception. Contested touches refresh a
    /// still-fresh reception without transferring it, so a 50/50 interruption
    /// does not break the original toucher's continuation; the challenger only
    /// claims the reception once they get a clean touch of their own.
    fn note_touch(&mut self, player_id: &PlayerId, time: f32, contested: bool) {
        let fresh = self.reception.as_ref().is_some_and(|reception| {
            (time - reception.last_touch_time) <= FIRST_TOUCH_RESET_SECONDS
        });
        match self.reception.as_mut() {
            Some(reception) if contested && fresh => {
                reception.last_touch_time = time;
            }
            _ => {
                self.reception = Some(Reception {
                    player: player_id.clone(),
                    last_touch_time: time,
                });
            }
        }
    }

    fn has_matching_stat_event(
        &self,
        kind: PlayerStatEventKind,
        player_id: &PlayerId,
        touch_time: f32,
    ) -> bool {
        self.recent_stat_events.iter().any(|event| {
            event.kind == kind
                && event.player == *player_id
                && (touch_time - event.time).abs() <= STAT_EVENT_MATCH_WINDOW_SECONDS
        })
    }

    fn is_geometric_save(ctx: &TouchIntentionFrameContext, is_team_0: bool) -> bool {
        let (Some(position), Some(velocity)) =
            (ctx.previous_ball_position, ctx.previous_ball_velocity)
        else {
            return false;
        };
        velocity.length() >= SAVE_MIN_INBOUND_BALL_SPEED
            && trajectory_crosses_goal_mouth(
                position,
                velocity,
                own_goal_line_y(is_team_0),
                SAVE_MAX_TIME_TO_GOAL_SECONDS,
            )
    }

    fn is_geometric_shot(ctx: &TouchIntentionFrameContext, is_team_0: bool) -> bool {
        let (Some(position), Some(velocity)) = (ctx.ball_position, ctx.ball_velocity) else {
            return false;
        };
        velocity.length() >= SHOT_MIN_BALL_SPEED
            && trajectory_crosses_goal_mouth(
                position,
                velocity,
                opponent_goal_line_y(is_team_0),
                SHOT_MAX_TIME_TO_GOAL_SECONDS,
            )
    }

    fn is_clear(ctx: &TouchIntentionFrameContext, is_team_0: bool) -> bool {
        let (Some(position), Some(velocity)) = (ctx.ball_position, ctx.ball_velocity) else {
            return false;
        };
        let team_forward_sign = if is_team_0 { 1.0 } else { -1.0 };
        if position.y * team_forward_sign > CLEAR_MAX_ATTACKING_Y {
            return false;
        }
        if velocity.length() < CLEAR_MIN_BALL_SPEED {
            return false;
        }
        let own_goal_center = glam::Vec3::new(0.0, own_goal_line_y(is_team_0), 0.0);
        let away_from_own_goal = (position - own_goal_center).normalize_or_zero();
        velocity.normalize_or_zero().dot(away_from_own_goal)
            >= CLEAR_MIN_AWAY_FROM_OWN_GOAL_ALIGNMENT
    }

    fn is_pass(ctx: &TouchIntentionFrameContext) -> bool {
        let (Some(position), Some(velocity)) = (ctx.ball_position, ctx.ball_velocity) else {
            return false;
        };
        let speed_squared = velocity.length_squared();
        if speed_squared < PASS_MIN_BALL_SPEED * PASS_MIN_BALL_SPEED {
            return false;
        }
        ctx.teammate_positions.iter().any(|teammate| {
            let lead_seconds = (*teammate - position).dot(velocity) / speed_squared;
            if !(PASS_MIN_LEAD_SECONDS..=PASS_MAX_LEAD_SECONDS).contains(&lead_seconds) {
                return false;
            }
            let lead_travel = velocity * lead_seconds;
            lead_travel.length() >= PASS_MIN_TRAVEL_DISTANCE
                && (position + lead_travel - *teammate).length() <= PASS_RECEIVER_MAX_DISTANCE
        })
    }
}

/// Outcome of a closed control-follow window. `touch_index` addresses the
/// touch event whose intention should be upgraded when `control` is true.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlResolution {
    pub touch_index: usize,
    pub control: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ControlFollowWindow {
    touch_index: usize,
    player: PlayerId,
    touch_time: f32,
    tracked_seconds: f32,
    controlled_seconds: f32,
}

impl ControlFollowWindow {
    /// Resolve on the stay-close criterion: most of the tracked follow time
    /// had the toucher close to the ball and roughly matching its velocity.
    fn stay_close_resolution(&self) -> ControlResolution {
        ControlResolution {
            touch_index: self.touch_index,
            control: self.tracked_seconds >= CONTROL_FOLLOW_MIN_TRACKED_SECONDS
                && self.controlled_seconds
                    >= CONTROL_FOLLOW_MIN_CONTROLLED_FRACTION * self.tracked_seconds,
        }
    }
}

/// Watches the window after a touch to decide, by outcome, whether the touch
/// was a control touch: did the toucher stay with the ball (close and
/// speed-matched) or earn the follow-up touch? Resolutions are applied
/// retroactively to the already-emitted touch event, mirroring how
/// ball-movement credit is finalized.
///
/// At most one window is open at a time: any new touch closes the previous
/// window, so the tracker never accumulates state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ControlFollowTracker {
    window: Option<ControlFollowWindow>,
}

impl ControlFollowTracker {
    /// The player whose follow window is open, so the caller can supply that
    /// player's frame sample to [`Self::advance`].
    pub fn window_player(&self) -> Option<&PlayerId> {
        self.window.as_ref().map(|window| &window.player)
    }

    /// Open a follow window for a just-emitted touch event. Only call this for
    /// touches whose provisional intention is upgradeable to control.
    pub fn open(&mut self, touch_index: usize, player_id: &PlayerId, time: f32) {
        self.window = Some(ControlFollowWindow {
            touch_index,
            player: player_id.clone(),
            touch_time: time,
            tracked_seconds: 0.0,
            controlled_seconds: 0.0,
        });
    }

    /// Resolve any open window against a new touch. A follow-up touch by the
    /// same player within the window confirms control directly (the touch
    /// enabled a follow-up); a touch by anyone else, or a late follow-up,
    /// closes the window on the stay-close criterion.
    pub fn observe_touch(&mut self, player_id: &PlayerId, time: f32) -> Option<ControlResolution> {
        let window = self.window.take()?;
        if window.player == *player_id && time - window.touch_time <= CONTROL_FOLLOW_WINDOW_SECONDS
        {
            return Some(ControlResolution {
                touch_index: window.touch_index,
                control: true,
            });
        }
        Some(window.stay_close_resolution())
    }

    /// Accumulate one frame of follow data for the open window and resolve it
    /// once it ages out. Missing ball or player data counts as uncontrolled
    /// time (a demolished toucher is not in control of anything).
    pub fn advance(
        &mut self,
        frame: &FrameInfo,
        ball_position: Option<glam::Vec3>,
        ball_velocity: Option<glam::Vec3>,
        player_position: Option<glam::Vec3>,
        player_velocity: Option<glam::Vec3>,
    ) -> Option<ControlResolution> {
        if self
            .window
            .as_ref()
            .is_some_and(|window| frame.time - window.touch_time > CONTROL_FOLLOW_WINDOW_SECONDS)
        {
            return self.flush();
        }
        let window = self.window.as_mut()?;
        let dt = frame.dt.max(0.0);
        window.tracked_seconds += dt;
        let close = match (ball_position, player_position) {
            (Some(ball_position), Some(player_position)) => {
                (ball_position - player_position).length() <= CONTROL_FOLLOW_MAX_DISTANCE
            }
            _ => false,
        };
        let speed_matched = match (ball_velocity, player_velocity) {
            (Some(ball_velocity), Some(player_velocity)) => {
                (ball_velocity - player_velocity).length() <= CONTROL_FOLLOW_MAX_RELATIVE_SPEED
            }
            _ => false,
        };
        if close && speed_matched {
            window.controlled_seconds += dt;
        }
        None
    }

    /// Close any open window immediately (live-play boundary or end of
    /// replay) on the stay-close criterion.
    pub fn flush(&mut self) -> Option<ControlResolution> {
        self.window
            .take()
            .map(|window| window.stay_close_resolution())
    }
}

/// A free-flight sample must be at least this far past the touch before its
/// projection is trusted. The dodge/flick impulse is delivered over several
/// frames, so the touch frame itself is the *least* reliable sample of where
/// the ball is actually headed; we wait for the trajectory to settle.
const SHOT_PROJECTION_MIN_FREE_FLIGHT_SECONDS: f32 = 0.06;

/// When a new touch ends free flight, samples within this many seconds of that
/// touch are ignored. The incoming touch's impulse can start moving the ball a
/// frame or two before its touch marker is detected (the same replication lag
/// as the dodge byte), so the very last free-flight samples may already be
/// contaminated by the next touch. Resolving from a few frames earlier keeps
/// the read on the ball's own settled trajectory.
const SHOT_PROJECTION_NEXT_TOUCH_GUARD_SECONDS: f32 = 0.12;

/// Outcome of a closed shot-projection window. `touch_index` addresses the
/// touch event to upgrade to a shot when `is_shot` is true.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShotProjectionResolution {
    pub touch_index: usize,
    pub is_shot: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ShotProjectionSample {
    position: glam::Vec3,
    velocity: glam::Vec3,
    time: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct ShotProjectionWindow {
    touch_index: usize,
    is_team_0: bool,
    touch_time: f32,
    /// Recent free-flight samples, oldest first. Bounded to the look-back the
    /// next-touch guard needs, so it never grows with window length.
    samples: VecDeque<ShotProjectionSample>,
}

impl ShotProjectionWindow {
    fn record(&mut self, sample: ShotProjectionSample) {
        // Retain enough history to look back past the next-touch guard, plus a
        // small margin, and no more.
        let cutoff = sample.time - (SHOT_PROJECTION_NEXT_TOUCH_GUARD_SECONDS + 0.1);
        self.samples.push_back(sample);
        while self
            .samples
            .front()
            .is_some_and(|oldest| oldest.time < cutoff)
        {
            self.samples.pop_front();
        }
    }

    fn is_shot_from(&self, sample: &ShotProjectionSample) -> bool {
        sample.time - self.touch_time >= SHOT_PROJECTION_MIN_FREE_FLIGHT_SECONDS
            && sample.velocity.length() >= SHOT_MIN_BALL_SPEED
            && trajectory_crosses_goal_mouth(
                sample.position,
                sample.velocity,
                opponent_goal_line_y(self.is_team_0),
                SHOT_MAX_TIME_TO_GOAL_SECONDS,
            )
    }

    /// Resolve from the most recent sample. Used when free flight ends at a
    /// boundary (goal / stoppage / end of replay): nothing perturbs the ball, so
    /// the last sample is the cleanest read of where it was headed.
    fn resolution_latest(&self) -> ShotProjectionResolution {
        let is_shot = self
            .samples
            .back()
            .is_some_and(|sample| self.is_shot_from(sample));
        ShotProjectionResolution {
            touch_index: self.touch_index,
            is_shot,
        }
    }

    /// Resolve from the most recent sample at least the guard interval before
    /// `close_time`. Used when a new touch ends free flight, so the read sits on
    /// the ball's own trajectory rather than the next touch's incoming impulse.
    fn resolution_before(&self, close_time: f32) -> ShotProjectionResolution {
        let limit = close_time - SHOT_PROJECTION_NEXT_TOUCH_GUARD_SECONDS;
        let is_shot = self
            .samples
            .iter()
            .rev()
            .find(|sample| sample.time <= limit)
            .is_some_and(|sample| self.is_shot_from(sample));
        ShotProjectionResolution {
            touch_index: self.touch_index,
            is_shot,
        }
    }
}

/// Watches the ball's free flight after a touch and decides, from the settled
/// trajectory before the next touch, whether the touch sent the ball goalward —
/// recovering shots whose touch-frame velocity had not yet finished updating.
/// Resolutions are applied retroactively to the already-emitted touch event,
/// mirroring [`ControlFollowTracker`].
///
/// At most one window is open at a time: any new touch closes the previous
/// window.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ShotProjectionTracker {
    window: Option<ShotProjectionWindow>,
}

impl ShotProjectionTracker {
    /// Open a projection window for a just-emitted touch event.
    pub fn open(&mut self, touch_index: usize, is_team_0: bool, time: f32) {
        self.window = Some(ShotProjectionWindow {
            touch_index,
            is_team_0,
            touch_time: time,
            samples: VecDeque::new(),
        });
    }

    /// Close the open window because a new touch at `next_touch_time` ended free
    /// flight, resolving it from a sample taken a guard interval before that
    /// touch so the next touch's pre-detection impulse can't skew the read.
    pub fn observe_touch(&mut self, next_touch_time: f32) -> Option<ShotProjectionResolution> {
        self.window
            .take()
            .map(|window| window.resolution_before(next_touch_time))
    }

    /// Record this frame's free-flight ball sample, or resolve the window once
    /// it ages past the shot time-to-goal horizon.
    pub fn advance(
        &mut self,
        frame: &FrameInfo,
        ball_position: Option<glam::Vec3>,
        ball_velocity: Option<glam::Vec3>,
    ) -> Option<ShotProjectionResolution> {
        if self
            .window
            .as_ref()
            .is_some_and(|window| frame.time - window.touch_time > SHOT_MAX_TIME_TO_GOAL_SECONDS)
        {
            return self.window.take().map(|window| window.resolution_latest());
        }
        let window = self.window.as_mut()?;
        if let (Some(position), Some(velocity)) = (ball_position, ball_velocity) {
            window.record(ShotProjectionSample {
                position,
                velocity,
                time: frame.time,
            });
        }
        None
    }

    /// Close any open window immediately (live-play boundary or end of replay).
    pub fn flush(&mut self) -> Option<ShotProjectionResolution> {
        self.window.take().map(|window| window.resolution_latest())
    }
}

/// True when an active 50/50 lists this player as one of its contestants.
pub(crate) fn fifty_fifty_involves_player(
    active: &ActiveFiftyFifty,
    player_id: &PlayerId,
    is_team_0: bool,
) -> bool {
    let contestant = if is_team_0 {
        active.team_zero_player.as_ref()
    } else {
        active.team_one_player.as_ref()
    };
    contestant == Some(player_id)
}

fn own_goal_line_y(is_team_0: bool) -> f32 {
    if is_team_0 { -GOAL_LINE_Y } else { GOAL_LINE_Y }
}

fn opponent_goal_line_y(is_team_0: bool) -> f32 {
    -own_goal_line_y(is_team_0)
}

/// Returns true when a ballistic projection of the trajectory crosses the goal
/// line at `target_goal_y` inside the goal mouth (with a ball-radius margin)
/// within `max_seconds`.
///
/// The horizontal (x/y) path is a straight line — gravity is purely vertical,
/// so it does not change when the ball reaches the goal line nor its lateral
/// position there. The vertical (z) path applies constant gravity, so a shot
/// hit upward that arcs back down into the net is read correctly instead of
/// being projected straight over the crossbar. Wall bounces and later touches
/// are still deliberately ignored.
fn trajectory_crosses_goal_mouth(
    position: glam::Vec3,
    velocity: glam::Vec3,
    target_goal_y: f32,
    max_seconds: f32,
) -> bool {
    if velocity.length_squared() <= f32::EPSILON {
        return false;
    }
    let time_to_goal_line = (target_goal_y - position.y) / velocity.y;
    if !time_to_goal_line.is_finite() || !(0.0..=max_seconds).contains(&time_to_goal_line) {
        return false;
    }
    let projected_x = position.x + velocity.x * time_to_goal_line;
    let projected_z = position.z
        + velocity.z * time_to_goal_line
        + 0.5
            * crate::util::ballistics::STANDARD_BALL_GRAVITY_Z
            * time_to_goal_line
            * time_to_goal_line;
    projected_x.abs() <= BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X + GOAL_MOUTH_TRAJECTORY_MARGIN
        && (BALL_RADIUS_Z - GOAL_MOUTH_TRAJECTORY_MARGIN
            ..=GOAL_MOUTH_HEIGHT_Z + GOAL_MOUTH_TRAJECTORY_MARGIN)
            .contains(&projected_z)
}

#[cfg(test)]
#[path = "touch_intention_tests.rs"]
mod tests;
