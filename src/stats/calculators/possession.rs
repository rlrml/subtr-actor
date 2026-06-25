use super::*;

const PENDING_TURNOVER_CONFIRMATION_WINDOW_SECONDS: f32 = 1.25;
const LOOSE_BALL_TIMEOUT_SECONDS: f32 = 3.0;

/// How long the resolver waits for a follow-up touch before deciding a
/// possession's fate. A possession's credited span is backdated to its owner's
/// last touch; the loose time after that touch stays provisional until either
/// the same owner re-touches (kept), the opponent confirms a turnover (the gap
/// goes neutral and the opponent is credited from their first touch), or this
/// window elapses with no follow-up (the gap goes neutral). It unifies the old
/// pending-turnover (1.25s) and loose-ball (3.0s) windows: with loss backdated
/// to the last touch, the distinction no longer changes anyone's credited time,
/// so a single resolution deadline suffices.
const POSSESSION_RESOLUTION_WINDOW_SECONDS: f32 = 1.5;

/// A team-or-neutral label for a resolved possession segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PossessionLabel {
    TeamZero,
    TeamOne,
    Neutral,
}

impl PossessionLabel {
    pub(crate) fn team(team_is_team_0: bool) -> Self {
        if team_is_team_0 {
            Self::TeamZero
        } else {
            Self::TeamOne
        }
    }

    pub(crate) fn as_label_value(self) -> &'static str {
        match self {
            Self::TeamZero => "team_zero",
            Self::TeamOne => "team_one",
            Self::Neutral => "neutral",
        }
    }
}

/// A finalized stretch of the possession timeline. The resolver emits these as
/// soon as a touch or timeout decides who (if anyone) owned the stretch, so the
/// loose time after a possession's last touch only becomes a team's credit once
/// that team demonstrably keeps the ball.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedPossession {
    pub start_time: f32,
    pub start_frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub label: PossessionLabel,
    pub player: Option<PlayerId>,
}

/// The still-open (unresolved) trailing segment of the possession timeline. Its
/// label is the current best guess; its true extent is only known once it is
/// resolved, but display consumers can render it as the live possession.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OpenPossession {
    pub start_time: f32,
    pub start_frame: usize,
    pub label: PossessionLabel,
    pub player: Option<PlayerId>,
}

/// The touch situation for a single frame, reduced to which team(s) contacted
/// the ball and the latest contacting player per team.
#[derive(Debug, Clone)]
enum TouchInput {
    None,
    /// Exactly one team touched.
    Single {
        team_is_team_0: bool,
        player: Option<PlayerId>,
    },
    /// Both teams touched the same frame (contested).
    Contested {
        team_zero_player: Option<PlayerId>,
        team_one_player: Option<PlayerId>,
    },
}

impl TouchInput {
    fn opponent_player(&self, owner_team_is_team_0: bool) -> Option<PlayerId> {
        match self {
            TouchInput::Contested {
                team_zero_player,
                team_one_player,
            } => {
                if owner_team_is_team_0 {
                    team_one_player.clone()
                } else {
                    team_zero_player.clone()
                }
            }
            _ => None,
        }
    }

    fn owner_player(&self, owner_team_is_team_0: bool) -> Option<PlayerId> {
        match self {
            TouchInput::Contested {
                team_zero_player,
                team_one_player,
            } => {
                if owner_team_is_team_0 {
                    team_zero_player.clone()
                } else {
                    team_one_player.clone()
                }
            }
            _ => None,
        }
    }
}

/// The resolver's phase: who (if anyone) currently holds the ball and what
/// follow-up we are waiting on.
#[derive(Debug, Clone, PartialEq, Default)]
enum ResolverPhase {
    /// No one is credited; the open segment is neutral.
    #[default]
    Neutral,
    /// One team touched a loose/neutral ball once but has not confirmed control.
    /// The open segment is still neutral — an unconfirmed touch grants nothing.
    Acquiring {
        team_is_team_0: bool,
        first_touch_time: f32,
        first_touch_frame: usize,
        player: Option<PlayerId>,
    },
    /// A team holds the ball; the open segment is that team's. `last_touch` is
    /// the trailing edge a loss would be backdated to.
    Held {
        team_is_team_0: bool,
        player: Option<PlayerId>,
        last_touch_time: f32,
        last_touch_frame: usize,
    },
    /// The holder still owns it, but an opponent has touched once. The open
    /// segment is still the holder's; the contest resolves on the next touch.
    HeldChallenged {
        team_is_team_0: bool,
        player: Option<PlayerId>,
        held_last_touch_time: f32,
        held_last_touch_frame: usize,
        challenger_first_time: f32,
        challenger_first_frame: usize,
        challenger_player: Option<PlayerId>,
    },
}

/// Resolves a touch timeline into possession segments with loss backdated to
/// the loser's last touch (see [`POSSESSION_RESOLUTION_WINDOW_SECONDS`]). This
/// is the source of truth for team `PossessionEvent`s and `PossessionStats`;
/// the legacy eager fields on [`PossessionTracker`] still drive per-player
/// possession.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct PossessionResolver {
    phase: ResolverPhase,
    open_start_time: f32,
    open_start_frame: usize,
    newly_resolved: Vec<ResolvedPossession>,
}

impl PossessionResolver {
    fn reset(&mut self) {
        self.phase = ResolverPhase::Neutral;
        self.open_start_time = 0.0;
        self.open_start_frame = 0;
        self.newly_resolved.clear();
    }

    /// Close the open segment at `(end_time, end_frame)` with `label`/`player`
    /// and start a new open segment there.
    fn finalize(
        &mut self,
        end_time: f32,
        end_frame: usize,
        label: PossessionLabel,
        player: Option<PlayerId>,
    ) {
        if end_time > self.open_start_time {
            self.newly_resolved.push(ResolvedPossession {
                start_time: self.open_start_time,
                start_frame: self.open_start_frame,
                end_time,
                end_frame,
                label,
                player,
            });
        }
        self.open_start_time = end_time;
        self.open_start_frame = end_frame;
    }

    fn within_window(now: f32, since: f32) -> bool {
        now - since <= POSSESSION_RESOLUTION_WINDOW_SECONDS
    }

    fn touch_input(
        touched_team_zero_player: &Option<PlayerId>,
        touched_team_one_player: &Option<PlayerId>,
        touched_team_zero: bool,
        touched_team_one: bool,
    ) -> TouchInput {
        match (touched_team_zero, touched_team_one) {
            (false, false) => TouchInput::None,
            (true, false) => TouchInput::Single {
                team_is_team_0: true,
                player: touched_team_zero_player.clone(),
            },
            (false, true) => TouchInput::Single {
                team_is_team_0: false,
                player: touched_team_one_player.clone(),
            },
            (true, true) => TouchInput::Contested {
                team_zero_player: touched_team_zero_player.clone(),
                team_one_player: touched_team_one_player.clone(),
            },
        }
    }

    /// Advance the resolver one frame. Pushes any segments finalized this frame
    /// onto `newly_resolved` (cleared at the start of every call).
    fn update(
        &mut self,
        frame: &FrameInfo,
        touched_team_zero_player: &Option<PlayerId>,
        touched_team_one_player: &Option<PlayerId>,
        touched_team_zero: bool,
        touched_team_one: bool,
    ) {
        self.newly_resolved.clear();
        let time = frame.time;
        let fnum = frame.frame_number;
        let input = Self::touch_input(
            touched_team_zero_player,
            touched_team_one_player,
            touched_team_zero,
            touched_team_one,
        );

        let phase = std::mem::take(&mut self.phase);
        self.phase = match phase {
            ResolverPhase::Neutral => self.step_neutral(time, fnum, input),
            ResolverPhase::Acquiring {
                team_is_team_0,
                first_touch_time,
                first_touch_frame,
                player,
            } => self.step_acquiring(
                time,
                fnum,
                input,
                team_is_team_0,
                first_touch_time,
                first_touch_frame,
                player,
            ),
            ResolverPhase::Held {
                team_is_team_0,
                player,
                last_touch_time,
                last_touch_frame,
            } => self.step_held(
                time,
                fnum,
                input,
                team_is_team_0,
                player,
                last_touch_time,
                last_touch_frame,
            ),
            ResolverPhase::HeldChallenged {
                team_is_team_0,
                player,
                held_last_touch_time,
                held_last_touch_frame,
                challenger_first_time,
                challenger_first_frame,
                challenger_player,
            } => self.step_held_challenged(
                time,
                fnum,
                input,
                team_is_team_0,
                player,
                held_last_touch_time,
                held_last_touch_frame,
                challenger_first_time,
                challenger_first_frame,
                challenger_player,
            ),
        };
    }

    fn step_neutral(&mut self, time: f32, fnum: usize, input: TouchInput) -> ResolverPhase {
        match input {
            // A single touch on a neutral ball is provisional: it grants nothing
            // until the same team confirms control with a follow-up.
            TouchInput::Single {
                team_is_team_0,
                player,
            } => ResolverPhase::Acquiring {
                team_is_team_0,
                first_touch_time: time,
                first_touch_frame: fnum,
                player,
            },
            // A contested neutral ball stays neutral until someone gets clear of it.
            TouchInput::None | TouchInput::Contested { .. } => ResolverPhase::Neutral,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn step_acquiring(
        &mut self,
        time: f32,
        fnum: usize,
        input: TouchInput,
        team_is_team_0: bool,
        first_touch_time: f32,
        first_touch_frame: usize,
        player: Option<PlayerId>,
    ) -> ResolverPhase {
        let still_acquiring = ResolverPhase::Acquiring {
            team_is_team_0,
            first_touch_time,
            first_touch_frame,
            player: player.clone(),
        };
        match input {
            TouchInput::None => {
                if Self::within_window(time, first_touch_time) {
                    still_acquiring
                } else {
                    // The lone touch was a deflection; the ball stays neutral.
                    ResolverPhase::Neutral
                }
            }
            TouchInput::Single {
                team_is_team_0: touch_team,
                player: touch_player,
            } => {
                if touch_team == team_is_team_0 {
                    // Confirmed: credit the team from their first touch. The open
                    // neutral segment ends there; a held segment opens.
                    self.finalize(
                        first_touch_time,
                        first_touch_frame,
                        PossessionLabel::Neutral,
                        None,
                    );
                    ResolverPhase::Held {
                        team_is_team_0,
                        player: touch_player.or(player),
                        last_touch_time: time,
                        last_touch_frame: fnum,
                    }
                } else {
                    // The other team got the latest single touch; track them now.
                    ResolverPhase::Acquiring {
                        team_is_team_0: touch_team,
                        first_touch_time: time,
                        first_touch_frame: fnum,
                        player: touch_player,
                    }
                }
            }
            // Contested while acquiring: still nobody in clear control.
            TouchInput::Contested { .. } => still_acquiring,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn step_held(
        &mut self,
        time: f32,
        fnum: usize,
        input: TouchInput,
        team_is_team_0: bool,
        player: Option<PlayerId>,
        last_touch_time: f32,
        last_touch_frame: usize,
    ) -> ResolverPhase {
        match input {
            TouchInput::None => {
                if Self::within_window(time, last_touch_time) {
                    ResolverPhase::Held {
                        team_is_team_0,
                        player,
                        last_touch_time,
                        last_touch_frame,
                    }
                } else {
                    // No follow-up came: possession ends at the last touch and the
                    // loose tail is neutral.
                    self.finalize(
                        last_touch_time,
                        last_touch_frame,
                        PossessionLabel::team(team_is_team_0),
                        player,
                    );
                    ResolverPhase::Neutral
                }
            }
            TouchInput::Single {
                team_is_team_0: touch_team,
                player: touch_player,
            } => {
                if touch_team == team_is_team_0 {
                    // Same team re-touches: the tail since the last touch is kept,
                    // the trailing edge advances.
                    ResolverPhase::Held {
                        team_is_team_0,
                        player: touch_player.or(player),
                        last_touch_time: time,
                        last_touch_frame: fnum,
                    }
                } else {
                    ResolverPhase::HeldChallenged {
                        team_is_team_0,
                        player,
                        held_last_touch_time: last_touch_time,
                        held_last_touch_frame: last_touch_frame,
                        challenger_first_time: time,
                        challenger_first_frame: fnum,
                        challenger_player: touch_player,
                    }
                }
            }
            TouchInput::Contested { .. } => {
                // Owner and opponent both touched: treat as a fresh challenge,
                // with the owner's trailing edge advanced to this frame.
                ResolverPhase::HeldChallenged {
                    team_is_team_0,
                    player: input.owner_player(team_is_team_0).or(player),
                    held_last_touch_time: time,
                    held_last_touch_frame: fnum,
                    challenger_first_time: time,
                    challenger_first_frame: fnum,
                    challenger_player: input.opponent_player(team_is_team_0),
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn step_held_challenged(
        &mut self,
        time: f32,
        fnum: usize,
        input: TouchInput,
        team_is_team_0: bool,
        player: Option<PlayerId>,
        held_last_touch_time: f32,
        held_last_touch_frame: usize,
        challenger_first_time: f32,
        challenger_first_frame: usize,
        challenger_player: Option<PlayerId>,
    ) -> ResolverPhase {
        let confirm_turnover = |resolver: &mut Self, new_player: Option<PlayerId>| {
            // The holder's credit ends at their last touch; the loose gap up to
            // the challenger's first touch is neutral; the challenger is credited
            // from that first touch.
            resolver.finalize(
                held_last_touch_time,
                held_last_touch_frame,
                PossessionLabel::team(team_is_team_0),
                player.clone(),
            );
            resolver.finalize(
                challenger_first_time,
                challenger_first_frame,
                PossessionLabel::Neutral,
                None,
            );
            ResolverPhase::Held {
                team_is_team_0: !team_is_team_0,
                player: new_player.or_else(|| challenger_player.clone()),
                last_touch_time: time,
                last_touch_frame: fnum,
            }
        };

        match input {
            TouchInput::None => {
                if Self::within_window(time, challenger_first_time) {
                    ResolverPhase::HeldChallenged {
                        team_is_team_0,
                        player,
                        held_last_touch_time,
                        held_last_touch_frame,
                        challenger_first_time,
                        challenger_first_frame,
                        challenger_player,
                    }
                } else {
                    // Neither side followed up: the contested ball was loose.
                    // Backdate the holder's loss to their last touch.
                    self.finalize(
                        held_last_touch_time,
                        held_last_touch_frame,
                        PossessionLabel::team(team_is_team_0),
                        player,
                    );
                    ResolverPhase::Neutral
                }
            }
            TouchInput::Single {
                team_is_team_0: touch_team,
                player: touch_player,
            } => {
                if touch_team == team_is_team_0 {
                    // Holder repelled the challenge: the challenger's lone touch
                    // never confirmed, so it grants nothing and does not break the
                    // holder's possession. The hold stays continuous through the
                    // poke (loss is only backdated when the holder is genuinely
                    // dispossessed, i.e. the opponent confirms or no follow-up
                    // comes).
                    ResolverPhase::Held {
                        team_is_team_0,
                        player: touch_player.or(player),
                        last_touch_time: time,
                        last_touch_frame: fnum,
                    }
                } else {
                    confirm_turnover(self, touch_player)
                }
            }
            TouchInput::Contested { .. } => {
                confirm_turnover(self, input.opponent_player(team_is_team_0))
            }
        }
    }

    /// The current open (unresolved) segment, or `None` when neutral with no
    /// accumulated time.
    fn open(&self) -> OpenPossession {
        let (label, player) = match &self.phase {
            ResolverPhase::Neutral | ResolverPhase::Acquiring { .. } => {
                (PossessionLabel::Neutral, None)
            }
            ResolverPhase::Held {
                team_is_team_0,
                player,
                ..
            }
            | ResolverPhase::HeldChallenged {
                team_is_team_0,
                player,
                ..
            } => (PossessionLabel::team(*team_is_team_0), player.clone()),
        };
        OpenPossession {
            start_time: self.open_start_time,
            start_frame: self.open_start_frame,
            label,
            player,
        }
    }

    /// Flush the open segment as resolved, backdating any held tail to the last
    /// touch (the follow-up never came). Used when live play ends.
    fn flush(&mut self, end_time: f32, end_frame: usize) {
        let phase = std::mem::take(&mut self.phase);
        match phase {
            ResolverPhase::Neutral | ResolverPhase::Acquiring { .. } => {
                self.finalize(end_time, end_frame, PossessionLabel::Neutral, None);
            }
            ResolverPhase::Held {
                team_is_team_0,
                player,
                last_touch_time,
                last_touch_frame,
            } => {
                self.finalize(
                    last_touch_time,
                    last_touch_frame,
                    PossessionLabel::team(team_is_team_0),
                    player,
                );
                self.finalize(end_time, end_frame, PossessionLabel::Neutral, None);
            }
            ResolverPhase::HeldChallenged {
                team_is_team_0,
                player,
                held_last_touch_time,
                held_last_touch_frame,
                ..
            } => {
                self.finalize(
                    held_last_touch_time,
                    held_last_touch_frame,
                    PossessionLabel::team(team_is_team_0),
                    player,
                );
                self.finalize(end_time, end_frame, PossessionLabel::Neutral, None);
            }
        }
        self.phase = ResolverPhase::Neutral;
    }
}

/// A team-possession span.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PossessionEvent {
    pub time: f32,
    pub frame: usize,
    pub end_time: f32,
    pub end_frame: usize,
    pub active: bool,
    pub duration: f32,
    pub possession_state: String,
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub player_id: Option<PlayerId>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct PossessionTracker {
    current_team_is_team_0: Option<bool>,
    current_player: Option<PlayerId>,
    last_possession_touch_time: Option<f32>,
    pending_turnover_team_is_team_0: Option<bool>,
    pending_turnover_touch_time: Option<f32>,
    /// Backdating resolver; source of truth for team possession segments. The
    /// eager fields above remain the source for per-player possession.
    resolver: PossessionResolver,
}

impl PossessionTracker {
    fn clear_pending_turnover(&mut self) {
        self.pending_turnover_team_is_team_0 = None;
        self.pending_turnover_touch_time = None;
    }

    pub(crate) fn reset(&mut self) {
        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
        self.clear_pending_turnover();
    }

    /// Begin a fresh resolver run at the start of a live-play stretch.
    pub(crate) fn begin_resolver(&mut self, frame: &FrameInfo) {
        self.resolver.reset();
        self.resolver.open_start_time = frame.time;
        self.resolver.open_start_frame = frame.frame_number;
    }

    /// Flush the resolver's open segment when live play ends, returning the
    /// segments finalized by the flush.
    pub(crate) fn flush_resolver(&mut self, frame: &FrameInfo) -> Vec<ResolvedPossession> {
        self.resolver.newly_resolved.clear();
        self.resolver.flush(frame.time, frame.frame_number);
        std::mem::take(&mut self.resolver.newly_resolved)
    }

    fn expire_pending_turnover(&mut self, time: f32) {
        let Some(pending_time) = self.pending_turnover_touch_time else {
            return;
        };
        if time - pending_time < PENDING_TURNOVER_CONFIRMATION_WINDOW_SECONDS {
            return;
        }

        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
        self.clear_pending_turnover();
    }

    fn expire_loose_ball(&mut self, time: f32) {
        if self.pending_turnover_team_is_team_0.is_some() {
            return;
        }
        let Some(last_touch_time) = self.last_possession_touch_time else {
            return;
        };
        if time - last_touch_time < LOOSE_BALL_TIMEOUT_SECONDS {
            return;
        }

        self.current_team_is_team_0 = None;
        self.current_player = None;
        self.last_possession_touch_time = None;
    }

    fn register_single_team_touch(&mut self, team_is_team_0: bool, time: f32) {
        if self.current_team_is_team_0 == Some(team_is_team_0) {
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        if self.current_team_is_team_0.is_none() {
            self.current_team_is_team_0 = Some(team_is_team_0);
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        if self.pending_turnover_team_is_team_0 == Some(team_is_team_0) {
            self.current_team_is_team_0 = Some(team_is_team_0);
            self.last_possession_touch_time = Some(time);
            self.clear_pending_turnover();
            return;
        }

        self.pending_turnover_team_is_team_0 = Some(team_is_team_0);
        self.pending_turnover_touch_time = Some(time);
    }

    fn register_contested_touch(&mut self, time: f32) {
        let Some(current_team_is_team_0) = self.current_team_is_team_0 else {
            self.clear_pending_turnover();
            return;
        };

        self.last_possession_touch_time = Some(time);
        self.pending_turnover_team_is_team_0 = Some(!current_team_is_team_0);
        self.pending_turnover_touch_time = Some(time);
    }

    fn update_player_control(
        &mut self,
        active_team_before_sample: Option<bool>,
        touched_team_zero_player: Option<&PlayerId>,
        touched_team_one_player: Option<&PlayerId>,
    ) {
        let Some(current_team_is_team_0) = self.current_team_is_team_0 else {
            self.current_player = None;
            return;
        };

        if self.pending_turnover_team_is_team_0.is_some() {
            self.current_player = None;
            return;
        }

        let controlling_touch_player = if current_team_is_team_0 {
            touched_team_zero_player
        } else {
            touched_team_one_player
        };
        if let Some(player) = controlling_touch_player {
            self.current_player = Some(player.clone());
            return;
        }

        if active_team_before_sample != self.current_team_is_team_0 {
            self.current_player = None;
        }
    }

    fn latest_touch_player_for_team(
        touch_events: &[TouchEvent],
        team_is_team_0: bool,
    ) -> Option<PlayerId> {
        touch_events
            .iter()
            .filter(|touch| touch.team_is_team_0 == team_is_team_0)
            .max_by(|left, right| TouchEvent::timestamp_ordering(left, right))
            .and_then(|touch| touch.player.clone())
    }

    pub(crate) fn update(
        &mut self,
        frame: &FrameInfo,
        touch_events: &[TouchEvent],
    ) -> PossessionState {
        let time = frame.time;
        self.expire_pending_turnover(time);
        self.expire_loose_ball(time);

        let active_team_before_sample = self.current_team_is_team_0;
        let active_player_before_sample = self.current_player.clone();
        let touched_team_zero = touch_events.iter().any(|touch| touch.team_is_team_0);
        let touched_team_one = touch_events.iter().any(|touch| !touch.team_is_team_0);
        let touched_team_zero_player = Self::latest_touch_player_for_team(touch_events, true);
        let touched_team_one_player = Self::latest_touch_player_for_team(touch_events, false);

        match (touched_team_zero, touched_team_one) {
            (true, true) => self.register_contested_touch(time),
            (true, false) => self.register_single_team_touch(true, time),
            (false, true) => self.register_single_team_touch(false, time),
            (false, false) => {}
        }
        self.update_player_control(
            active_team_before_sample,
            touched_team_zero_player.as_ref(),
            touched_team_one_player.as_ref(),
        );

        self.resolver.update(
            frame,
            &touched_team_zero_player,
            &touched_team_one_player,
            touched_team_zero,
            touched_team_one,
        );

        PossessionState {
            active_team_before_sample,
            current_team_is_team_0: self.current_team_is_team_0,
            active_player_before_sample,
            current_player: self.current_player.clone(),
            newly_resolved: std::mem::take(&mut self.resolver.newly_resolved),
            open_possession: Some(self.resolver.open()),
        }
    }
}

#[cfg(test)]
#[path = "possession_tests.rs"]
mod tests;

/// Builds the team-possession event stream from the backdating resolver's
/// finalized segments.
///
/// Finalized [`ResolvedPossession`] segments (active spans) are emitted as
/// `PossessionEvent`s as soon as the resolver decides them; non-live stretches
/// are emitted as coalesced inactive markers so the stream stays contiguous.
/// The in-progress open segment is exposed via [`Self::current_event`] /
/// [`Self::projected_events`] for display and goal tagging.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PossessionCalculator {
    events: EventStream<PossessionEvent>,
    /// Segments finalized this frame, surfaced for the stats projection's
    /// deferred per-frame accumulation.
    new_resolved: Vec<ResolvedPossession>,
    /// The current open (unresolved) segment rendered up to the latest frame.
    open_event: Option<PossessionEvent>,
    /// Coalesced inactive (non-live) marker awaiting flush.
    inactive_pending: Option<PossessionEvent>,
}

impl PossessionCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[PossessionEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[PossessionEvent] {
        self.events.new_events()
    }

    /// All committed events plus the in-progress open/inactive span. Used by
    /// goal tagging to walk the possession that led to a goal.
    pub fn projected_events(&self) -> Vec<PossessionEvent> {
        let mut events = self.events.all().to_vec();
        if let Some(open) = &self.open_event {
            events.push(open.clone());
        }
        if let Some(inactive) = &self.inactive_pending {
            events.push(inactive.clone());
        }
        events
    }

    /// Segments the resolver finalized on the most recent frame.
    pub(crate) fn new_resolved(&self) -> &[ResolvedPossession] {
        &self.new_resolved
    }

    pub fn flush_pending_event(&mut self) {
        if let Some(inactive) = self.inactive_pending.take() {
            self.events.push(inactive);
        }
        if let Some(open) = self.open_event.take() {
            self.events.push(open);
        }
    }

    /// The span covering the most recently processed frame (the open segment if
    /// live, else the last committed event).
    pub fn current_event(&self) -> Option<&PossessionEvent> {
        self.open_event
            .as_ref()
            .or(self.inactive_pending.as_ref())
            .or_else(|| self.events.all().last())
    }

    fn segment_event(segment: &ResolvedPossession) -> PossessionEvent {
        PossessionEvent {
            time: segment.start_time,
            frame: segment.start_frame,
            end_time: segment.end_time,
            end_frame: segment.end_frame,
            active: true,
            duration: (segment.end_time - segment.start_time).max(0.0),
            possession_state: segment.label.as_label_value().to_owned(),
            player_id: segment.player.clone(),
        }
    }

    fn flush_inactive(&mut self) {
        if let Some(inactive) = self.inactive_pending.take() {
            self.events.push(inactive);
        }
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        possession_state: &PossessionState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        self.new_resolved.clear();

        // Commit segments the resolver finalized this frame. On the live→non-live
        // edge these are the flushed tail of the just-ended stretch, so they must
        // precede any inactive marker for this frame.
        if !possession_state.newly_resolved.is_empty() {
            self.flush_inactive();
            for segment in &possession_state.newly_resolved {
                self.events.push(Self::segment_event(segment));
                self.new_resolved.push(segment.clone());
            }
        }

        if !live_play_state.is_live_play {
            self.open_event = None;
            match self.inactive_pending.as_mut() {
                Some(inactive) => {
                    inactive.end_time = frame.time;
                    inactive.end_frame = frame.frame_number;
                    inactive.duration = (inactive.end_time - inactive.time).max(0.0);
                }
                None => {
                    self.inactive_pending = Some(PossessionEvent {
                        time: frame.time,
                        frame: frame.frame_number,
                        end_time: frame.time,
                        end_frame: frame.frame_number,
                        active: false,
                        duration: 0.0,
                        possession_state: PossessionLabel::Neutral.as_label_value().to_owned(),
                        player_id: None,
                    });
                }
            }
            return Ok(());
        }

        self.flush_inactive();
        self.open_event = possession_state.open_possession.as_ref().map(|open| {
            let label = open.label.as_label_value().to_owned();
            PossessionEvent {
                time: open.start_time,
                frame: open.start_frame,
                end_time: frame.time,
                end_frame: frame.frame_number,
                active: true,
                duration: (frame.time - open.start_time).max(0.0),
                possession_state: label,
                player_id: open.player.clone(),
            }
        });
        Ok(())
    }
}
