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
/// A boom is a hard hit sent a long way downfield into space. It only gets
/// evaluated after shot/clear/pass fail, so the remaining gate is "fast and
/// pointed toward the opponent half" — distinguishing a deliberate big hit from
/// a soft loose-ball poke (neutral).
const BOOM_MIN_BALL_SPEED: f32 = 1500.0;
const BOOM_MIN_DOWNFIELD_ALIGNMENT: f32 = 0.3;
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
/// In the same-player follow-up path, how far the ball must get from the
/// toucher at some point in the window for the touch to read as an *advance*
/// (the ball was played into space and recovered) rather than *control* (the
/// ball was kept close). Comfortably above [`CONTROL_FOLLOW_MAX_DISTANCE`] so a
/// brief bobble out of the control radius during a tight dribble still resolves
/// as control, not advance.
const ADVANCE_MIN_PEAK_DISTANCE: f32 = 900.0;

/// What a touch was trying to *do* with the ball, read at contact time.
///
/// This is the touch's action axis: it sits alongside the orthogonal
/// `possession` axis ([`Possession`], assigned retroactively) and the
/// `contested` flag, rather than competing with them in one slot. A touch can
/// therefore be both `Boom` (action) and [`Possession::Advance`] (outcome) — a
/// dump-in the player chases down stays a boom.
///
/// Actions are mutually exclusive; overlaps are resolved by a precedence ladder
/// (see [`TouchIntentionClassifier::classify`]). There is no "nothing" action:
/// a touch with no recognized action simply has no action tag, rather than a
/// catch-all value. Contested and first-touch context is preserved separately
/// on [`TouchActionResolution`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchAction {
    Shot,
    Save,
    Clear,
    /// A hard hit sent a long way downfield into space (not toward a teammate,
    /// not a defensive-third clear, not on goal). Booms count as booms even
    /// when the hitter recovers them — recovery is recorded on the orthogonal
    /// [`Possession`] axis.
    Boom,
    Pass,
}

impl TouchAction {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Shot => "shot",
            Self::Save => "save",
            Self::Clear => "clear",
            Self::Boom => "boom",
            Self::Pass => "pass",
        }
    }

    /// Whether a follow window should watch this action for a possession
    /// outcome. Shots and saves are not possession plays, so a shooter who
    /// happens to recover their own shot is not credited with control/advance;
    /// the looser, recoverable actions (pass, clear, boom) are. An action-less
    /// touch (a loose poke or soft dribble touch) is watched too; callers gate
    /// on `action.is_none_or(TouchAction::watches_possession)`.
    pub(crate) fn watches_possession(self) -> bool {
        matches!(self, Self::Pass | Self::Clear | Self::Boom)
    }
}

/// What *happened to possession* after a touch, assigned retroactively by
/// [`ControlFollowTracker`] once the outcome is known. Orthogonal to
/// [`TouchAction`]: it answers "did the toucher keep the ball?", independent of
/// what the touch was trying to do.
///
/// - `Control` — the ball is kept close. Either the toucher stays with it
///   (close and speed-matched) or wins the follow-up touch without the ball
///   ever leaving the control radius.
/// - `Advance` — the ball is played into space (it leaves the control radius)
///   and the same toucher still wins the next touch. The follow-up touch is the
///   evidence they got to it before anyone else: a self-pass into space they
///   knew they would win.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Possession {
    Control,
    Advance,
}

impl Possession {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Advance => "advance",
        }
    }
}

/// The resolved touch intention with supporting context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchActionResolution {
    /// The recognized action, or `None` when the touch has no notable action
    /// (it is described by its other tags — kind, possession, etc. — instead).
    pub action: Option<TouchAction>,
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

    /// Classify one touch's action and advance first-touch tracking.
    ///
    /// Touches must be supplied in chronological order. The precedence ladder:
    /// replay-confirmed saves and shots first (game-authoritative), then
    /// trajectory-based saves, shots, clears out of the defensive third, booms
    /// downfield into space, and passes led toward a teammate. A touch matching
    /// none of these has no action (`None`) rather than a catch-all value.
    /// `contested` is not a rung — it is reported on the resolution as an
    /// independent flag, so a contested touch keeps its real action (a contested
    /// shot stays a shot). The `possession` axis is likewise independent: a
    /// [`ControlFollowTracker`] window may later record control/advance on a
    /// pass/clear/boom or action-less touch without changing its action.
    pub fn classify(
        &mut self,
        touch: &TouchEvent,
        player_id: &PlayerId,
        ctx: &TouchIntentionFrameContext,
    ) -> TouchActionResolution {
        let first_touch = self.is_first_touch(player_id, touch.time);
        let is_team_0 = touch.team_is_team_0;

        let action =
            if self.has_matching_stat_event(PlayerStatEventKind::Save, player_id, touch.time) {
                Some(TouchAction::Save)
            } else if self.has_matching_stat_event(PlayerStatEventKind::Shot, player_id, touch.time)
            {
                Some(TouchAction::Shot)
            } else if Self::is_geometric_save(ctx, is_team_0) {
                Some(TouchAction::Save)
            } else if Self::is_geometric_shot(ctx, is_team_0) {
                Some(TouchAction::Shot)
            } else if Self::is_clear(ctx, is_team_0) {
                Some(TouchAction::Clear)
            } else if Self::is_pass(ctx) {
                Some(TouchAction::Pass)
            } else if Self::is_boom(ctx, is_team_0) {
                Some(TouchAction::Boom)
            } else {
                None
            };

        self.note_touch(player_id, touch.time, ctx.contested);

        TouchActionResolution {
            action,
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

    /// A hard hit pointed downfield into space. Evaluated only after shot,
    /// clear, and pass have been ruled out, so this captures the deliberate
    /// "boom it forward / dump it in" hit that isn't on goal, isn't a defensive
    /// clear, and isn't aimed at a teammate.
    fn is_boom(ctx: &TouchIntentionFrameContext, is_team_0: bool) -> bool {
        let (Some(position), Some(velocity)) = (ctx.ball_position, ctx.ball_velocity) else {
            return false;
        };
        if velocity.length() < BOOM_MIN_BALL_SPEED {
            return false;
        }
        let opponent_goal_center = glam::Vec3::new(0.0, opponent_goal_line_y(is_team_0), 0.0);
        let toward_opponent_goal = (opponent_goal_center - position).normalize_or_zero();
        velocity.normalize_or_zero().dot(toward_opponent_goal) >= BOOM_MIN_DOWNFIELD_ALIGNMENT
    }
}

/// Outcome of a closed control-follow window. `touch_index` addresses the
/// touch event whose `possession` tag should be set; `possession` is the tag to
/// apply, or `None` when the window resolved without confirming possession.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PossessionResolution {
    pub touch_index: usize,
    pub possession: Option<Possession>,
}

#[derive(Debug, Clone, PartialEq)]
struct ControlFollowWindow {
    touch_index: usize,
    player: PlayerId,
    touch_time: f32,
    tracked_seconds: f32,
    controlled_seconds: f32,
    /// Greatest toucher-to-ball distance seen so far in the window (only frames
    /// with both positions known count). Distinguishes an advance — the ball
    /// played into space — from control, where it stays within reach.
    max_ball_distance: f32,
}

impl ControlFollowWindow {
    /// Resolve on the stay-close criterion: most of the tracked follow time
    /// had the toucher close to the ball and roughly matching its velocity.
    /// Stay-close resolutions are always `Control` — the ball never left the
    /// toucher, so this path never yields an advance.
    fn stay_close_resolution(&self) -> PossessionResolution {
        let confirmed = self.tracked_seconds >= CONTROL_FOLLOW_MIN_TRACKED_SECONDS
            && self.controlled_seconds
                >= CONTROL_FOLLOW_MIN_CONTROLLED_FRACTION * self.tracked_seconds;
        PossessionResolution {
            touch_index: self.touch_index,
            possession: confirmed.then_some(Possession::Control),
        }
    }

    /// Resolve on a same-player follow-up touch: the toucher won the next touch.
    /// Whether that reads as control or advance turns on how far the ball got
    /// from them in between — kept close is control, played into space is an
    /// advance.
    fn follow_up_resolution(&self) -> PossessionResolution {
        let possession = if self.max_ball_distance >= ADVANCE_MIN_PEAK_DISTANCE {
            Possession::Advance
        } else {
            Possession::Control
        };
        PossessionResolution {
            touch_index: self.touch_index,
            possession: Some(possession),
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
    /// touches whose action watches the possession axis (pass/neutral/clear/boom).
    pub fn open(&mut self, touch_index: usize, player_id: &PlayerId, time: f32) {
        self.window = Some(ControlFollowWindow {
            touch_index,
            player: player_id.clone(),
            touch_time: time,
            tracked_seconds: 0.0,
            controlled_seconds: 0.0,
            max_ball_distance: 0.0,
        });
    }

    /// Resolve any open window against a new touch. A follow-up touch by the
    /// same player within the window confirms possession directly (the touch
    /// enabled a follow-up) — control if the ball stayed close, advance if it
    /// was played into space first; a touch by anyone else, or a late
    /// follow-up, closes the window on the stay-close criterion.
    pub fn observe_touch(
        &mut self,
        player_id: &PlayerId,
        time: f32,
    ) -> Option<PossessionResolution> {
        let window = self.window.take()?;
        if window.player == *player_id && time - window.touch_time <= CONTROL_FOLLOW_WINDOW_SECONDS
        {
            return Some(window.follow_up_resolution());
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
    ) -> Option<PossessionResolution> {
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
        let distance = match (ball_position, player_position) {
            (Some(ball_position), Some(player_position)) => {
                Some((ball_position - player_position).length())
            }
            _ => None,
        };
        if let Some(distance) = distance {
            window.max_ball_distance = window.max_ball_distance.max(distance);
        }
        let close = distance.is_some_and(|distance| distance <= CONTROL_FOLLOW_MAX_DISTANCE);
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
    pub fn flush(&mut self) -> Option<PossessionResolution> {
        self.window
            .take()
            .map(|window| window.stay_close_resolution())
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

/// Returns true when a straight-line projection of the trajectory crosses the
/// goal line at `target_goal_y` inside the goal mouth (with a ball-radius
/// margin) within `max_seconds`. Gravity, bounces, and later touches are
/// deliberately ignored, matching the other goal-mouth projections in this
/// crate.
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
    let projected = position + velocity * time_to_goal_line;
    projected.x.abs() <= BACK_WALL_GOAL_MOUTH_HALF_WIDTH_X + GOAL_MOUTH_TRAJECTORY_MARGIN
        && projected.z >= BALL_RADIUS_Z - GOAL_MOUTH_TRAJECTORY_MARGIN
        && projected.z <= GOAL_MOUTH_HEIGHT_Z + GOAL_MOUTH_TRAJECTORY_MARGIN
}

#[cfg(test)]
#[path = "touch_intention_tests.rs"]
mod tests;
