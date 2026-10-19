use super::whiff_features::{
    WHIFF_FEATURE_NAMES, WHIFF_RESOLUTION_DELAY_SECONDS, WhiffFeatures, WhiffHistory,
    whiff_attempt_start, whiff_candidate_features,
};
use super::whiff_model::{WhiffAttemptClass, WhiffClassification, classify_whiff_candidate};
use super::*;

/// Candidates of one player closer together than this are one interaction.
const WHIFF_MIN_CANDIDATE_SPACING_SECONDS: f64 = 0.25;

/// Outcome carried by whiff events.
///
/// Detection emits clean whiffs and disrupted attempts as separate kinds. The
/// beaten-to-ball variant remains readable so existing timelines and
/// accumulated stats stay compatible; beaten-to-ball detection has its own
/// event stream.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum WhiffEventKind {
    /// A committed attempt that missed without another player's touch
    /// disrupting it.
    #[default]
    Whiff,
    BeatenToBall,
    /// A committed attempt that never connected because another player's touch
    /// took the ball first.
    DisruptedAttempt,
}

/// Why a whiff candidate became a confirmed whiff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, rename_all = "snake_case")]
pub enum WhiffResolutionReason {
    /// The player moved back outside the candidate window without any touch.
    SeparatedFromBall,
    /// Compatibility value for serialized events that predate resolution
    /// reasons (including the legacy beaten-to-ball subtype).
    LegacyUnknown,
    /// The attempt classifier labeled the closest approach once its
    /// resolution delay elapsed.
    AttemptClassified,
}

/// A committed attempt near the ball that resolves as a miss.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct WhiffEvent {
    #[serde(default)]
    pub kind: WhiffEventKind,
    /// Start of the approach that led to the closest approach.
    pub start_time: f32,
    pub start_frame: usize,
    /// Closest-approach anchor for the event.
    pub time: f32,
    pub frame: usize,
    /// Frame where the miss became known.
    pub resolved_time: f32,
    pub resolved_frame: usize,
    pub resolution_reason: WhiffResolutionReason,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    /// Player position at closest approach.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    /// Ball-center to car-hitbox distance at closest approach.
    pub closest_approach_distance: f32,
    /// Car-forward alignment with the ball direction at closest approach.
    pub forward_alignment: f32,
    /// Player velocity projected toward the ball at closest approach.
    pub approach_speed: f32,
    /// Player-minus-ball velocity projected toward the ball at closest
    /// approach. Optional only for compatibility with older serialized events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closing_speed_at_closest: Option<f32>,
    /// Player velocity alignment with the ball direction at closest approach.
    /// Optional only for compatibility with older serialized events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub velocity_alignment_at_closest: Option<f32>,
    /// Ball position in car-local coordinates at closest approach. Optional
    /// only for compatibility with older serialized events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_ball_position_at_closest: Option<[f32; 3]>,
    /// Hitbox distance on the frame that resolved the attempt. Optional only
    /// for compatibility with older serialized events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_distance: Option<f32>,
    /// Whether a dodge was observed during the attempt.
    pub dodge_active: bool,
    /// Whether the player was airborne during the attempt.
    pub aerial: bool,
    /// Classifier probability of the emitted kind. Not a calibrated
    /// probability of human agreement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    /// For disrupted attempts, the player whose touch nearest the closest
    /// approach took the ball.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(as = "Option<crate::interop::ts_bindings::RemoteIdTs>")]
    pub disrupted_by: Option<PlayerId>,
}

pub(crate) const WHIFF_DODGE_STATE_LABELS: [StatLabel; 2] = [
    StatLabel::new("dodge_state", "no_dodge"),
    StatLabel::new("dodge_state", "dodge"),
];

impl WhiffEvent {
    pub(crate) fn labels(&self) -> [StatLabel; 2] {
        [
            vertical_state_label(self.aerial),
            whiff_dodge_state_label(self.dodge_active),
        ]
    }
}

pub(crate) fn whiff_dodge_state_label(dodge_active: bool) -> StatLabel {
    if dodge_active {
        StatLabel::new("dodge_state", "dodge")
    } else {
        StatLabel::new("dodge_state", "no_dodge")
    }
}

/// Classifies a candidate's feature vector. Replaceable for tests.
pub type WhiffClassifier = fn(&WhiffFeatures) -> WhiffClassification;

/// A resolved candidate with its features, recorded for training/audit when
/// enabled via [`WhiffCalculator::enable_candidate_recording`].
#[derive(Debug, Clone, PartialEq)]
pub struct WhiffCandidateRecord {
    pub player: PlayerId,
    pub is_team_0: bool,
    pub anchor_time: f32,
    pub anchor_frame: usize,
    pub resolved_frame: usize,
    pub features: WhiffFeatures,
    pub classification: WhiffClassification,
}

impl WhiffCandidateRecord {
    pub fn feature_names() -> &'static [&'static str] {
        &WHIFF_FEATURE_NAMES
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PendingCandidate {
    slot: usize,
    anchor_seq: usize,
    anchor_time: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct SlotScan {
    next_seq: usize,
    last_anchor_time: Option<f64>,
}

/// Detects clean whiffs and disrupted attempts.
///
/// Candidates are local minima of each player's ball contact gap during live
/// play. Each is classified from causal telemetry once
/// [`WHIFF_RESOLUTION_DELAY_SECONDS`] have elapsed, so a successful touch, a
/// renewed attempt after the player's own touch, or another player's touch
/// before arrival is decided per closest approach rather than by clearing
/// every pending candidate on any touch.
#[derive(Debug, Clone)]
pub struct WhiffCalculator {
    history: WhiffHistory,
    scans: Vec<SlotScan>,
    pending: Vec<PendingCandidate>,
    classifier: WhiffClassifier,
    record_candidates: bool,
    candidates: Vec<WhiffCandidateRecord>,
    events: EventStream<WhiffEvent>,
}

impl Default for WhiffCalculator {
    fn default() -> Self {
        Self::with_classifier(classify_whiff_candidate)
    }
}

impl WhiffCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_classifier(classifier: WhiffClassifier) -> Self {
        Self {
            history: WhiffHistory::default(),
            scans: Vec::new(),
            pending: Vec::new(),
            classifier,
            record_candidates: false,
            candidates: Vec::new(),
            events: EventStream::default(),
        }
    }

    /// Record every resolved candidate with its features and classification.
    pub fn enable_candidate_recording(&mut self) {
        self.record_candidates = true;
    }

    pub fn candidates(&self) -> &[WhiffCandidateRecord] {
        &self.candidates
    }

    pub fn events(&self) -> &[WhiffEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[WhiffEvent] {
        self.events.new_events()
    }

    fn scan_candidates(&mut self) {
        let Some(last_seq) = self.history.last_seq() else {
            return;
        };
        let slot_count = self.history.slot_count();
        while self.scans.len() < slot_count {
            self.scans.push(SlotScan {
                next_seq: last_seq,
                last_anchor_time: None,
            });
        }
        for slot in 0..slot_count {
            let scan = &mut self.scans[slot];
            let mut seq = scan.next_seq.max(self.history.first_seq());
            while seq <= last_seq {
                let Some(is_minimum) = self.history.is_gap_minimum(slot, seq) else {
                    break;
                };
                if is_minimum {
                    let time = self.history.frame(seq).expect("seq in history").time;
                    let spaced = scan
                        .last_anchor_time
                        .is_none_or(|last| time - last >= WHIFF_MIN_CANDIDATE_SPACING_SECONDS);
                    if spaced {
                        scan.last_anchor_time = Some(time);
                        self.pending.push(PendingCandidate {
                            slot,
                            anchor_seq: seq,
                            anchor_time: time,
                        });
                    }
                }
                seq += 1;
            }
            scan.next_seq = seq;
        }
    }

    fn resolve_pending(&mut self, force: bool) {
        let Some(last_seq) = self.history.last_seq() else {
            return;
        };
        let now = self.history.frame(last_seq).expect("last frame").time;
        let (mut due, waiting): (Vec<_>, Vec<_>) = self.pending.drain(..).partition(|candidate| {
            force || now >= candidate.anchor_time + WHIFF_RESOLUTION_DELAY_SECONDS
        });
        self.pending = waiting;
        // Emit simultaneous resolutions in timeline order (attempt start).
        due.sort_by(|a, b| {
            let start =
                |c: &PendingCandidate| whiff_attempt_start(&self.history, c.slot, c.anchor_seq).0;
            start(a)
                .total_cmp(&start(b))
                .then(a.anchor_seq.cmp(&b.anchor_seq))
                .then(a.slot.cmp(&b.slot))
        });
        for candidate in due {
            self.resolve(candidate, last_seq);
        }
    }

    fn resolve(&mut self, candidate: PendingCandidate, resolved_seq: usize) {
        let Some(features) =
            whiff_candidate_features(&self.history, candidate.slot, candidate.anchor_seq)
        else {
            return;
        };
        let classification = (self.classifier)(&features);
        let anchor = self
            .history
            .frame(candidate.anchor_seq)
            .expect("anchor in history");
        let resolved = self.history.frame(resolved_seq).expect("resolved frame");
        let (player, is_team_0) = self.history.slot_player(candidate.slot).clone();
        if self.record_candidates {
            self.candidates.push(WhiffCandidateRecord {
                player: player.clone(),
                is_team_0,
                anchor_time: anchor.time as f32,
                anchor_frame: anchor.frame,
                resolved_frame: resolved.frame,
                features,
                classification,
            });
        }
        let kind = match classification.class {
            WhiffAttemptClass::CleanWhiff => WhiffEventKind::Whiff,
            WhiffAttemptClass::DisruptedAttempt => WhiffEventKind::DisruptedAttempt,
            WhiffAttemptClass::SuccessfulContact | WhiffAttemptClass::NoObservableAttempt => {
                return;
            }
        };
        let Some(sample) = anchor.players.get(candidate.slot).and_then(Option::as_ref) else {
            return;
        };
        let to_ball = anchor.ball_position - sample.position;
        let direction = to_ball.normalize_or_zero();
        let velocity = if sample.velocity.is_finite() {
            sample.velocity
        } else {
            glam::DVec3::ZERO
        };
        let ball_velocity = if anchor.ball_velocity.is_finite() {
            anchor.ball_velocity
        } else {
            glam::DVec3::ZERO
        };
        let feature = |name: &str| {
            features[WHIFF_FEATURE_NAMES
                .iter()
                .position(|n| *n == name)
                .expect("known feature")]
        };
        let (start_time, start_frame) =
            whiff_attempt_start(&self.history, candidate.slot, candidate.anchor_seq);
        let disrupted_by = (kind == WhiffEventKind::DisruptedAttempt)
            .then(|| {
                self.history
                    .nearest_other_touch(candidate.slot, candidate.anchor_seq)
            })
            .flatten();
        let resolved_distance = resolved
            .players
            .get(candidate.slot)
            .and_then(Option::as_ref)
            .map(|p| p.hitbox_distance)
            .filter(|d| d.is_finite());
        self.events.push(WhiffEvent {
            kind,
            start_time: start_time as f32,
            start_frame,
            time: anchor.time as f32,
            frame: anchor.frame,
            resolved_time: resolved.time as f32,
            resolved_frame: resolved.frame,
            resolution_reason: WhiffResolutionReason::AttemptClassified,
            player,
            player_position: Some(sample.position.as_vec3().to_array()),
            is_team_0,
            closest_approach_distance: sample.hitbox_distance as f32,
            forward_alignment: feature("forward_alignment") as f32,
            approach_speed: velocity.dot(direction) as f32,
            closing_speed_at_closest: Some((velocity - ball_velocity).dot(direction) as f32),
            velocity_alignment_at_closest: Some(velocity.normalize_or_zero().dot(direction) as f32),
            local_ball_position_at_closest: Some(sample.local_ball.as_vec3().to_array()),
            resolved_distance: resolved_distance.map(|d| d as f32),
            dodge_active: feature("dodge_recent") > 0.5,
            aerial: sample.position.z > f64::from(POWERSLIDE_MAX_Z_THRESHOLD),
            confidence: Some(classification.probability() as f32),
            disrupted_by,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        controls: &PlayerControlState,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if self
            .history
            .last_seq()
            .and_then(|seq| self.history.frame(seq))
            .is_some_and(|last| f64::from(frame.time) < last.time)
        {
            // Time went backwards (e.g. a seek): resolve what is known and
            // restart the history.
            self.resolve_pending(true);
            self.history.clear();
        }
        self.history.push(
            frame,
            ball,
            players,
            controls,
            touch_state,
            live_play_state.is_live_play,
        );
        self.scan_candidates();
        self.resolve_pending(false);
        Ok(())
    }

    /// Resolve remaining candidates with the frames observed so far.
    pub fn finish(&mut self) {
        self.resolve_pending(true);
    }
}

#[cfg(test)]
#[path = "whiff_tests.rs"]
mod tests;
