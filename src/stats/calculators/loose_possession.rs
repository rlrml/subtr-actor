use super::possession::{OpenPossession, PossessionLabel, ResolvedPossession};
use super::*;

/// How long the loose resolver waits for a follow-up touch before deciding a
/// contested ball's fate. Mirrors the strict resolver's window, but the loose
/// resolver keeps crediting the last team to touch through loose balls instead
/// of letting them lapse to neutral.
const LOOSE_RESOLUTION_WINDOW_SECONDS: f32 = 1.5;

/// A team-possession span under the *loose* definition: the team that last
/// touched the ball owns it until the opponent demonstrably takes it away.
///
/// Unlike the strict [`super::possession::PossessionEvent`] (which only credits
/// firmly-controlled time and sends loose tails / unconfirmed touches to
/// neutral), loose possession is sticky: it survives loose balls after a team's
/// last touch, survives teammate passes, and survives repelled 50-50s. On a
/// turnover the boundary is backdated to the opponent's takeover touch, so the
/// losing team keeps credit right up to the moment the opponent wins it and
/// there is no neutral gap. Consequently loose possession is almost always
/// `team_zero` or `team_one` (neutral only before the first touch of a live
/// stretch, or during a contested scramble off a neutral ball).
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct LoosePossessionEvent {
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

/// The touch situation for a single frame, reduced to which team(s) contacted
/// the ball and the latest contacting player per team.
#[derive(Debug, Clone)]
enum LooseTouchInput {
    None,
    Single {
        team_is_team_0: bool,
        player: Option<PlayerId>,
    },
    Contested {
        team_zero_player: Option<PlayerId>,
        team_one_player: Option<PlayerId>,
    },
}

impl LooseTouchInput {
    fn owner_player(&self, owner_team_is_team_0: bool) -> Option<PlayerId> {
        match self {
            LooseTouchInput::Contested {
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

    fn opponent_player(&self, owner_team_is_team_0: bool) -> Option<PlayerId> {
        match self {
            LooseTouchInput::Contested {
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
}

/// The loose resolver's phase: who (if anyone) is the last team to touch and
/// what follow-up we are waiting on.
#[derive(Debug, Clone, PartialEq, Default)]
enum LoosePhase {
    /// No one has touched yet (start of a live stretch) or the ball is loose off
    /// a contested neutral scramble. The open segment is neutral.
    #[default]
    Neutral,
    /// A team owns the ball (was the last to touch and has not been dispossessed).
    /// The open segment is that team's, and stays so through loose balls.
    Owned {
        team_is_team_0: bool,
        player: Option<PlayerId>,
    },
    /// The owner still owns it, but an opponent has touched once. The open
    /// segment is still the owner's; the contest resolves on the next touch
    /// (owner re-touch keeps it; opponent confirm or timeout turns it over,
    /// backdated to the opponent's first touch).
    OwnedChallenged {
        team_is_team_0: bool,
        player: Option<PlayerId>,
        challenger_first_time: f32,
        challenger_first_frame: usize,
        challenger_player: Option<PlayerId>,
    },
}

/// Resolves a touch timeline into *loose* possession segments: the last team to
/// touch owns the ball until the opponent takes it away, with the turnover
/// backdated to the opponent's takeover touch. See [`LoosePossessionEvent`].
#[derive(Debug, Clone, PartialEq, Default)]
struct LoosePossessionResolver {
    phase: LoosePhase,
    open_start_time: f32,
    open_start_frame: usize,
    newly_resolved: Vec<ResolvedPossession>,
}

impl LoosePossessionResolver {
    fn reset(&mut self) {
        self.phase = LoosePhase::Neutral;
        self.open_start_time = 0.0;
        self.open_start_frame = 0;
        self.newly_resolved.clear();
    }

    fn begin(&mut self, frame: &FrameInfo) {
        self.reset();
        self.open_start_time = frame.time;
        self.open_start_frame = frame.frame_number;
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
        now - since <= LOOSE_RESOLUTION_WINDOW_SECONDS
    }

    fn touch_input(
        touched_team_zero_player: &Option<PlayerId>,
        touched_team_one_player: &Option<PlayerId>,
        touched_team_zero: bool,
        touched_team_one: bool,
    ) -> LooseTouchInput {
        match (touched_team_zero, touched_team_one) {
            (false, false) => LooseTouchInput::None,
            (true, false) => LooseTouchInput::Single {
                team_is_team_0: true,
                player: touched_team_zero_player.clone(),
            },
            (false, true) => LooseTouchInput::Single {
                team_is_team_0: false,
                player: touched_team_one_player.clone(),
            },
            (true, true) => LooseTouchInput::Contested {
                team_zero_player: touched_team_zero_player.clone(),
                team_one_player: touched_team_one_player.clone(),
            },
        }
    }

    /// Advance the resolver one frame, pushing any finalized segments onto
    /// `newly_resolved` (cleared at the start of every call).
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
            LoosePhase::Neutral => self.step_neutral(time, fnum, input),
            LoosePhase::Owned {
                team_is_team_0,
                player,
            } => self.step_owned(time, fnum, input, team_is_team_0, player),
            LoosePhase::OwnedChallenged {
                team_is_team_0,
                player,
                challenger_first_time,
                challenger_first_frame,
                challenger_player,
            } => self.step_owned_challenged(
                time,
                fnum,
                input,
                team_is_team_0,
                player,
                challenger_first_time,
                challenger_first_frame,
                challenger_player,
            ),
        };
    }

    fn step_neutral(&mut self, time: f32, fnum: usize, input: LooseTouchInput) -> LoosePhase {
        match input {
            // A single touch grants the loose ball immediately: the team that
            // touched is now the last to touch, so it owns from this frame.
            LooseTouchInput::Single {
                team_is_team_0,
                player,
            } => {
                self.finalize(time, fnum, PossessionLabel::Neutral, None);
                LoosePhase::Owned {
                    team_is_team_0,
                    player,
                }
            }
            // No clear last-toucher: the ball stays neutral.
            LooseTouchInput::None | LooseTouchInput::Contested { .. } => LoosePhase::Neutral,
        }
    }

    fn step_owned(
        &mut self,
        time: f32,
        fnum: usize,
        input: LooseTouchInput,
        team_is_team_0: bool,
        player: Option<PlayerId>,
    ) -> LoosePhase {
        match input {
            // Sticky: a loose ball after the owner's last touch stays the owner's.
            LooseTouchInput::None => LoosePhase::Owned {
                team_is_team_0,
                player,
            },
            LooseTouchInput::Single {
                team_is_team_0: touch_team,
                player: touch_player,
            } => {
                if touch_team == team_is_team_0 {
                    LoosePhase::Owned {
                        team_is_team_0,
                        player: touch_player.or(player),
                    }
                } else {
                    LoosePhase::OwnedChallenged {
                        team_is_team_0,
                        player,
                        challenger_first_time: time,
                        challenger_first_frame: fnum,
                        challenger_player: touch_player,
                    }
                }
            }
            LooseTouchInput::Contested { .. } => LoosePhase::OwnedChallenged {
                team_is_team_0,
                player: input.owner_player(team_is_team_0).or(player),
                challenger_first_time: time,
                challenger_first_frame: fnum,
                challenger_player: input.opponent_player(team_is_team_0),
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn step_owned_challenged(
        &mut self,
        time: f32,
        _fnum: usize,
        input: LooseTouchInput,
        team_is_team_0: bool,
        player: Option<PlayerId>,
        challenger_first_time: f32,
        challenger_first_frame: usize,
        challenger_player: Option<PlayerId>,
    ) -> LoosePhase {
        // Turn the ball over to the challenger, backdated to their first touch:
        // the owner keeps credit up to that touch, then the challenger owns.
        let confirm_turnover = |resolver: &mut Self, new_player: Option<PlayerId>| {
            resolver.finalize(
                challenger_first_time,
                challenger_first_frame,
                PossessionLabel::team(team_is_team_0),
                player.clone(),
            );
            LoosePhase::Owned {
                team_is_team_0: !team_is_team_0,
                player: new_player.or_else(|| challenger_player.clone()),
            }
        };

        match input {
            LooseTouchInput::None => {
                if Self::within_window(time, challenger_first_time) {
                    LoosePhase::OwnedChallenged {
                        team_is_team_0,
                        player,
                        challenger_first_time,
                        challenger_first_frame,
                        challenger_player,
                    }
                } else {
                    // No follow-up: the challenger was the last to touch, so the
                    // ball is theirs from their touch.
                    confirm_turnover(self, None)
                }
            }
            LooseTouchInput::Single {
                team_is_team_0: touch_team,
                player: touch_player,
            } => {
                if touch_team == team_is_team_0 {
                    // Owner re-takes: the challenge was repelled, so possession
                    // stays continuous (no boundary) — survives the 50-50.
                    LoosePhase::Owned {
                        team_is_team_0,
                        player: touch_player.or(player),
                    }
                } else {
                    confirm_turnover(self, touch_player)
                }
            }
            // Still contested: defer until a clean touch or the window elapses.
            LooseTouchInput::Contested { .. } => LoosePhase::OwnedChallenged {
                team_is_team_0,
                player,
                challenger_first_time,
                challenger_first_frame,
                challenger_player,
            },
        }
    }

    /// The current open (unresolved) segment.
    fn open(&self) -> OpenPossession {
        let (label, player) = match &self.phase {
            LoosePhase::Neutral => (PossessionLabel::Neutral, None),
            LoosePhase::Owned {
                team_is_team_0,
                player,
            }
            | LoosePhase::OwnedChallenged {
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

    /// Flush the open segment when live play ends. The owner keeps its sticky
    /// tail all the way to the end (the opponent never took it away).
    fn flush(&mut self, end_time: f32, end_frame: usize) {
        let phase = std::mem::take(&mut self.phase);
        match phase {
            LoosePhase::Neutral => {
                self.finalize(end_time, end_frame, PossessionLabel::Neutral, None);
            }
            LoosePhase::Owned {
                team_is_team_0,
                player,
            }
            | LoosePhase::OwnedChallenged {
                team_is_team_0,
                player,
                ..
            } => {
                self.finalize(
                    end_time,
                    end_frame,
                    PossessionLabel::team(team_is_team_0),
                    player,
                );
            }
        }
        self.phase = LoosePhase::Neutral;
    }
}

/// Builds the loose team-possession event stream. Self-contained: it derives the
/// per-frame touch input from `TouchState` and drives its own
/// [`LoosePossessionResolver`], so it does not perturb the strict possession
/// path. Finalized segments are emitted as soon as the resolver decides them;
/// non-live stretches are coalesced into inactive markers so the stream stays
/// contiguous.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LoosePossessionCalculator {
    resolver: LoosePossessionResolver,
    was_live: bool,
    events: EventStream<LoosePossessionEvent>,
    open_event: Option<LoosePossessionEvent>,
    inactive_pending: Option<LoosePossessionEvent>,
}

impl LoosePossessionCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[LoosePossessionEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[LoosePossessionEvent] {
        self.events.new_events()
    }

    pub fn projected_events(&self) -> Vec<LoosePossessionEvent> {
        let mut events = self.events.all().to_vec();
        if let Some(open) = &self.open_event {
            events.push(open.clone());
        }
        if let Some(inactive) = &self.inactive_pending {
            events.push(inactive.clone());
        }
        events
    }

    pub fn current_event(&self) -> Option<&LoosePossessionEvent> {
        self.open_event
            .as_ref()
            .or(self.inactive_pending.as_ref())
            .or_else(|| self.events.all().last())
    }

    pub fn flush_pending_event(&mut self) {
        if let Some(inactive) = self.inactive_pending.take() {
            self.events.push(inactive);
        }
        if let Some(open) = self.open_event.take() {
            self.events.push(open);
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

    fn segment_event(segment: &ResolvedPossession) -> LoosePossessionEvent {
        LoosePossessionEvent {
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

    fn commit_resolved(&mut self) {
        let resolved = std::mem::take(&mut self.resolver.newly_resolved);
        if resolved.is_empty() {
            return;
        }
        self.flush_inactive();
        for segment in &resolved {
            self.events.push(Self::segment_event(segment));
        }
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();

        if !live_play_state.is_live_play {
            if self.was_live {
                self.resolver.flush(frame.time, frame.frame_number);
                self.commit_resolved();
                self.was_live = false;
            }
            self.open_event = None;
            match self.inactive_pending.as_mut() {
                Some(inactive) => {
                    inactive.end_time = frame.time;
                    inactive.end_frame = frame.frame_number;
                    inactive.duration = (inactive.end_time - inactive.time).max(0.0);
                }
                None => {
                    self.inactive_pending = Some(LoosePossessionEvent {
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

        if !self.was_live {
            self.resolver.begin(frame);
            self.was_live = true;
        }

        let touched_team_zero = touch_state
            .touch_events
            .iter()
            .any(|touch| touch.team_is_team_0);
        let touched_team_one = touch_state
            .touch_events
            .iter()
            .any(|touch| !touch.team_is_team_0);
        let touched_team_zero_player =
            Self::latest_touch_player_for_team(&touch_state.touch_events, true);
        let touched_team_one_player =
            Self::latest_touch_player_for_team(&touch_state.touch_events, false);

        self.resolver.update(
            frame,
            &touched_team_zero_player,
            &touched_team_one_player,
            touched_team_zero,
            touched_team_one,
        );
        self.commit_resolved();

        self.flush_inactive();
        let open = self.resolver.open();
        self.open_event = Some(LoosePossessionEvent {
            time: open.start_time,
            frame: open.start_frame,
            end_time: frame.time,
            end_frame: frame.frame_number,
            active: true,
            duration: (frame.time - open.start_time).max(0.0),
            possession_state: open.label.as_label_value().to_owned(),
            player_id: open.player.clone(),
        });
        Ok(())
    }

    pub fn finish(&mut self) {
        self.flush_pending_event();
    }
}

#[cfg(test)]
#[path = "loose_possession_tests.rs"]
mod tests;
