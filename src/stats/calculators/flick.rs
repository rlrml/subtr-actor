use super::*;

/// Ball gravity (Unreal units / s²) used to remove the gravitational component
/// when estimating the velocity change a touch imparted to the ball.
const BALL_GRAVITY_Z: f32 = -650.0;

/// How long after a flick-candidate touch the detector keeps measuring the
/// ball's velocity change. A flick's power is not delivered in the single frame
/// the touch is first detected: when a car carries/drags the ball through the
/// dodge (e.g. a 180 flick) the ball keeps accelerating for a few frames after
/// contact. Measuring the *peak* gravity-compensated impulse over this window —
/// instead of one frame — is what lets those flicks clear the impulse gate.
const FLICK_IMPULSE_WINDOW_SECONDS: f32 = 0.15;

const FLICK_MAX_DODGE_TO_TOUCH_SECONDS: f32 = 0.32;
/// How far *before* the recorded dodge transition the flick contact may register.
/// The ball can start accelerating well before the dodge-active byte flips — the
/// launch touch has been observed up to ~0.23s ahead of the byte on downsampled
/// replays — so the touch that the pending flick anchors to sometimes precedes
/// the recorded dodge start. Allow a negative `time_since_dodge` down to the
/// shared dodge-byte lag tolerance rather than rejecting these as "touch before
/// dodge". See [`DODGE_ACTIVE_BYTE_LAG_TOLERANCE_SECONDS`].
const FLICK_DODGE_LEAD_TOLERANCE_SECONDS: f32 = DODGE_ACTIVE_BYTE_LAG_TOLERANCE_SECONDS;
/// How long a pending flick is kept alive waiting to be confirmed. Impulse is
/// only *measured* over [`FLICK_IMPULSE_WINDOW_SECONDS`], but the entry must
/// outlive that window so a dodge byte that replicates late (see
/// [`FLICK_DODGE_LEAD_TOLERANCE_SECONDS`]) can still attach to the launch touch
/// and emit the flick. Must be at least the impulse window.
const FLICK_PENDING_RETENTION_SECONDS: f32 = FLICK_DODGE_LEAD_TOLERANCE_SECONDS;
const _: () = assert!(FLICK_PENDING_RETENTION_SECONDS >= FLICK_IMPULSE_WINDOW_SECONDS);
const FLICK_MAX_CONTROL_TO_DODGE_SECONDS: f32 = 0.08;
const FLICK_MAX_SETUP_STALE_SECONDS: f32 = 0.35;
/// How long a control setup survives without a fresh control observation before
/// it is finished. A real carry/dribble lets the ball wobble in and out of the
/// tight control volume (the ball briefly exceeds the gap thresholds), so
/// finishing the setup on the first dropped frame fragments one ~0.5s carry into
/// sub-`FLICK_MIN_SETUP_SECONDS` pieces that never qualify. Bridging brief gaps
/// keeps the setup continuous while still ending it when the carry truly stops.
const FLICK_SETUP_GAP_GRACE_SECONDS: f32 = 0.12;
const FLICK_MIN_PENDING_DODGE_SETUP_SECONDS: f32 = 0.10;
const FLICK_MIN_SETUP_SECONDS: f32 = 0.20;
const FLICK_MIN_BALL_SPEED_CHANGE: f32 = 325.0;
const FLICK_MIN_CONFIDENCE: f32 = 0.55;
const FLICK_MAX_CONTROL_BALL_Z: f32 = 700.0;
const FLICK_MAX_CONTROL_HORIZONTAL_GAP: f32 = BALL_RADIUS_Z * 1.7;
const FLICK_MIN_CONTROL_VERTICAL_GAP: f32 = 35.0;
const FLICK_MAX_CONTROL_VERTICAL_GAP: f32 = 280.0;
/// Carry evidence threshold: the *minimum* horizontal speed difference between
/// the ball and the car observed across a flick setup must fall at or below this
/// for the setup to count as a genuine carry/dribble. A real flick rides the
/// ball on the car, so at some point during the setup their horizontal
/// velocities track within tens of uu/s (observed minima ~24–46). A loose ball
/// the car is merely driving at keeps its own velocity, so the difference never
/// drops — its setup-minimum stays ~600+. Taking the minimum over the whole
/// setup (rather than gating per frame) keeps the high relative velocity at the
/// instant of the dodge from causing a false negative.
const FLICK_MAX_CARRY_REL_HORIZONTAL_SPEED: f32 = 300.0;
const FLICK_MIN_LOCAL_Z: f32 = 20.0;
const FLICK_MAX_LOCAL_X_BEHIND: f32 = 95.0;
const FLICK_MAX_LOCAL_X_FRONT: f32 = 210.0;
const FLICK_MAX_LOCAL_Y: f32 = 170.0;
const FLICK_MIN_IMPULSE_AWAY_ALIGNMENT: f32 = 0.15;

/// A flick's kind is read purely from the dodge's rotation — the `dodge_torque`
/// axis, which is **car-relative** (the flip's rotation axis in the car's body
/// frame, decoded from controlled flips): its `y` component is
/// `dodge_forward_back` (>0 forward dodge, <0 backflip) and its `x` component is
/// `dodge_side` (signed left/right). Because the axis is a unit vector in that
/// plane, `dodge_forward_back² + dodge_side² ≈ 1`. No linear ball impulse, and
/// no velocity/heading, enter the classification.
///
/// The components are read **directly** off the car-relative torque. (An earlier
/// version decomposed it in a travel frame — dotting this car-relative vector
/// against the world velocity heading — which was a frame error that made the
/// classification depend on the car's world facing.)
///
/// Reverse flick: the dodge is sufficiently *backward* (a backflip), i.e.
/// `dodge_forward_back <= -REVERSE_FLICK_MIN_BACKWARD`. Calibrated from a pro
/// corpus (Mawkzy's 98 flicks across 42 rocket-sense replays): his
/// `dodge_forward_back` is bimodal — a backflip cluster from ≈-1.0 up to ≈-0.25,
/// then a sparse valley (only ~6 of 98 in -0.25..+0.1), then a forward cluster
/// above +0.1. `0.25` sits at the top edge of the backflip cluster / start of
/// the valley, so it captures pure backflips (back ≈ 1.0) and back-diagonal
/// "reverse 45s" (back ≈ 0.5-0.7, heavily side) without reaching into the
/// forward population. (Was 0.35, which sliced into the backflip cluster.)
const REVERSE_FLICK_MIN_BACKWARD: f32 = 0.25;
/// A reverse flick must also actually *rotate the car onto its side/back* — a
/// plain backflip (end-over-end, no roll) is a different mechanic. Gated on
/// `|underside_rotation|`. Calibrated from the `reverse-flick-vs-backflip`
/// controlled replay (reviewed ground truth: no pre-goal dodge is a reverse
/// flick, every post-goal dodge is): the pre-goal backflips sit at
/// `|rotation| ≈ 0.00-0.13`, the post-goal reverse flicks at `≥ 0.27`. `0.2`
/// separates them with margin.
const REVERSE_FLICK_MIN_UNDERSIDE_ROTATION: f32 = 0.2;
/// A reverse flick drives the ball *forward*, not backward — the dodge is
/// backward but the carried ball is thrown out ahead of the car. Gated on
/// `launch_forward_alignment` (horizontal launch direction vs travel heading).
/// A mild `0.4` keeps every reviewed reverse flick (all launched `≥ 0.82`) while
/// rejecting backward pops; the precise discriminator is the verticality gate
/// below.
const REVERSE_FLICK_MIN_LAUNCH_FORWARD: f32 = 0.4;
/// The defining tell of a reverse flick vs a plain backflip "flick": a reverse
/// flick sends the ball *forward and flat*, while a backflip pops it nearly
/// straight up. Gated on the launch impulse's vertical fraction
/// (`impulse.z / |impulse|` = sin of the launch elevation). In the
/// `reverse-flick-vs-backflip` controlled replay (reviewed ground truth) this
/// separates the two cleanly with a wide margin: the reviewed reverse flicks
/// launched at vertical fraction `0.30-0.43` (elevation 17-26°), every reviewed
/// non-reverse pop at `0.82-0.99` (elevation 55-82°). `0.6` (elevation ~37°)
/// sits in the gap. This is what excludes the ~57s pre-goal vertical pop that
/// the forward/rotation gates alone let through.
const REVERSE_FLICK_MAX_LAUNCH_VERTICAL_FRACTION: f32 = 0.6;
/// A side flick's dodge is dominated by its sideways (roll) component.
const SIDE_FLICK_MIN_SIDE: f32 = 0.6;
/// A forward flick's dodge is sufficiently forward (front flip).
const FORWARD_FLICK_MIN_FORWARD: f32 = 0.35;
/// Minimum |dodge_side| (the dodge's sideways/roll component) to tag a flick's
/// handedness left/right; below it the flick is `center` (e.g. a pure 90).
const FLICK_DIRECTION_MIN_SIDE: f32 = 0.25;
/// Maps the signed `dodge_side` (already `= -torque.x`, so + means right) to the
/// right/left labels. Handedness was calibrated from the controlled-flip replay
/// (left runs read `torque.x > 0`, right runs `< 0`); flip this sign if visual
/// review shows it inverted — that replay's run order is the only ground truth so
/// far, not yet visually pinned.
const FLICK_DODGE_SIDE_RIGHT_SIGN: f32 = 1.0;

/// The kind of flick detected, from the dodge direction (see
/// [`REVERSE_FLICK_MIN_BACKWARD`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlickKind {
    /// No clear dodge direction (e.g. missing dodge torque), or a dodge that is
    /// neither clearly forward, backward, nor sideways.
    Other,
    /// A front-flip flick: the dodge is forward.
    Forward,
    /// A reverse flick: the dodge is a backflip.
    Reverse,
    /// A side flick: the dodge is dominated by its sideways component.
    Side,
}

/// The handedness of a flick, from the dodge's own sideways component
/// (`dodge_side`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlickDirection {
    Center,
    Left,
    Right,
}

pub(crate) const FLICK_KIND_LABELS: [StatLabel; 4] = [
    StatLabel::new("kind", "other"),
    StatLabel::new("kind", "forward"),
    StatLabel::new("kind", "reverse"),
    StatLabel::new("kind", "side"),
];

pub(crate) const FLICK_DIRECTION_LABELS: [StatLabel; 3] = [
    StatLabel::new("direction", "center"),
    StatLabel::new("direction", "left"),
    StatLabel::new("direction", "right"),
];

impl FlickKind {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Other => "other",
            Self::Forward => "forward",
            Self::Reverse => "reverse",
            Self::Side => "side",
        }
    }

    pub fn as_label(self) -> StatLabel {
        flick_kind_label(self.as_label_value())
    }
}

impl FlickDirection {
    pub fn as_label_value(self) -> &'static str {
        match self {
            Self::Center => "center",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

pub(crate) fn flick_kind_label(value: &str) -> StatLabel {
    match value {
        "forward" => StatLabel::new("kind", "forward"),
        "reverse" => StatLabel::new("kind", "reverse"),
        "side" => StatLabel::new("kind", "side"),
        _ => StatLabel::new("kind", "other"),
    }
}

pub(crate) fn flick_direction_label(value: &str) -> StatLabel {
    match value {
        "left" => StatLabel::new("direction", "left"),
        "right" => StatLabel::new("direction", "right"),
        _ => StatLabel::new("direction", "center"),
    }
}

/// A dodge-powered touch following a short controlled carry setup.
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct FlickEvent {
    pub time: f32,
    pub frame: usize,
    pub sample_time: f32,
    pub sample_frame: usize,
    #[ts(as = "crate::interop::ts_bindings::RemoteIdTs")]
    pub player: PlayerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_position: Option<[f32; 3]>,
    pub is_team_0: bool,
    pub dodge_time: f32,
    pub dodge_frame: usize,
    pub time_since_dodge: f32,
    pub setup_start_time: f32,
    pub setup_start_frame: usize,
    pub setup_duration: f32,
    pub setup_touch_count: u32,
    pub average_horizontal_gap: f32,
    pub average_vertical_gap: f32,
    pub ball_speed_change: f32,
    pub ball_impulse: [f32; 3],
    pub impulse_away_alignment: f32,
    pub vertical_impulse: f32,
    pub kind: String,
    pub direction: String,
    pub local_ball_position: [f32; 3],
    pub local_ball_impulse: [f32; 3],
    /// Dodge direction (car-relative): >0 forward dodge, <0 backflip.
    pub dodge_forward_back: f32,
    /// Dodge direction (car-relative): signed sideways component (+right, -left;
    /// the handedness source).
    pub dodge_side: f32,
    /// Raw car-relative dodge torque (the flip's rotation axis) `[x, y, z]`:
    /// `y` = forward/back, `x` = left/right, `z` ≈ 0 (nonzero only when the car
    /// is tilted). `dodge_forward_back`/`dodge_side` are its normalized 2D
    /// projection; this is the full raw signal. `None` on inputs that don't
    /// replicate dodge torque (e.g. the BakkesMod live path).
    pub dodge_torque: Option<[f32; 3]>,
    /// Signed horizontal angle (radians) from the car's facing to its velocity
    /// heading at the launch touch (projected to the x/y plane) — how far the
    /// car was turned off its line of travel. With the car-relative dodge
    /// direction this recovers the dodge direction relative to the run's motion.
    /// 0 when the car's speed or horizontal facing is too small to be meaningful.
    pub travel_offset_radians: f32,
    /// How forward the ball was launched: the ball's gravity-compensated launch
    /// impulse (horizontal) dotted with the run's travel heading at the touch,
    /// in the range -1 to 1. A high value means the ball was sent forward along
    /// the run (a real reverse or forward flick); a low or negative value means
    /// it was sent backward or sideways (e.g. a plain backflip, not a reverse
    /// flick). 0 when the speed or launch is degenerate.
    pub launch_forward_alignment: f32,
    /// How vertical the launch was: the fraction of the gravity-compensated launch
    /// impulse that points up (`impulse.z / |impulse|` = sin of the launch
    /// elevation). A reverse flick sends the ball forward and flat, so this is low
    /// (around 0.3-0.45); a plain backflip "flick" pops the ball nearly straight
    /// up, so this is high (above 0.8). This is the primary signal separating the
    /// two; gates the reverse classification.
    pub launch_vertical_fraction: f32,
    /// How far the car has rotated onto its side/back at the touch (its underside
    /// turned away from straight down): signed, positive when rolled to the car's
    /// right and negative to its left, magnitude about sin(roll angle) so values
    /// near 1 mean fully on its side. A reverse flick rolls the car (unlike a
    /// plain backflip); gates the reverse classification together with
    /// `launch_forward_alignment`.
    pub underside_rotation: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct FlickControlObservation {
    horizontal_gap: f32,
    vertical_gap: f32,
    /// Horizontal speed difference between ball and car this frame, or `None`
    /// when velocity data is unavailable. See [`FLICK_MAX_CARRY_REL_HORIZONTAL_SPEED`].
    relative_horizontal_speed: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
struct ActiveFlickSetup {
    is_team_0: bool,
    start_time: f32,
    start_frame: usize,
    last_time: f32,
    last_frame: usize,
    duration: f32,
    horizontal_gap_integral: f32,
    vertical_gap_integral: f32,
    touch_count: u32,
    /// Smallest ball-vs-car horizontal speed difference seen during a *non-dodge*
    /// frame of the setup, or `f32::INFINITY` if none. See
    /// [`FLICK_MAX_CARRY_REL_HORIZONTAL_SPEED`].
    min_relative_horizontal_speed: f32,
    /// Whether any frame of the setup carried ball+car velocity data. Lets the
    /// carry check stay lenient on replays without velocities while still
    /// rejecting a setup that *has* velocity data but no non-dodge carry.
    observed_velocity: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct FlickSetupSummary {
    is_team_0: bool,
    start_time: f32,
    start_frame: usize,
    last_time: f32,
    last_frame: usize,
    duration: f32,
    average_horizontal_gap: f32,
    average_vertical_gap: f32,
    touch_count: u32,
    min_relative_horizontal_speed: f32,
    observed_velocity: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct RecentDodgeStart {
    time: f32,
    frame: usize,
    setup: FlickSetupSummary,
    rotation_at_dodge: Option<glam::Quat>,
    /// Car-relative dodge torque (the flip's rotation axis) captured at the
    /// dodge: `y` = forward/back, `x` = left/right. `None` when the input did not
    /// replicate it. This is read directly — no travel/world frame needed.
    dodge_torque: Option<glam::Vec3>,
}

/// A touch that looks like it could be a flick, kept alive for a short window so
/// the detector can watch the ball's full velocity change (see
/// [`FLICK_IMPULSE_WINDOW_SECONDS`]). `peak_impulse` is the largest
/// gravity-compensated change observed since just before the touch; `ball` and
/// `player` are snapshotted at the touch so the flick geometry is measured at
/// contact while its power is measured across the window.
#[derive(Debug, Clone)]
struct PendingFlick {
    touch_event: TouchEvent,
    ball: BallFrameState,
    player: PlayerSample,
    /// Dodge start recorded by a genuine dodge-active transition, when present.
    real_dodge_start: Option<RecentDodgeStart>,
    /// Whether the touch was classified as a dodge contact downstream.
    classified_dodge: bool,
    /// Start of the impulse measurement: the time of the first touch in this
    /// contact episode. Equal to the touch time unless this pending superseded
    /// an earlier same-episode touch, in which case it inherits that touch's
    /// anchor so the measured impulse spans the whole episode (a dodge drags
    /// the ball across several frames; see [`FLICK_IMPULSE_WINDOW_SECONDS`]).
    measure_start_time: f32,
    /// Ball velocity in the frame just before the touch at `measure_start_time`.
    pre_velocity: glam::Vec3,
    peak_impulse: glam::Vec3,
    peak_magnitude: f32,
}

impl PartialEq for PendingFlick {
    fn eq(&self, other: &Self) -> bool {
        self.touch_event.touch_id == other.touch_event.touch_id
            && self.touch_event.player == other.touch_event.player
            && self.touch_event.frame == other.touch_event.frame
    }
}

/// Detects flicks from ball/player state and touches.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlickCalculator {
    events: EventStream<FlickEvent>,
    active_setups: HashMap<PlayerId, ActiveFlickSetup>,
    recent_setups: HashMap<PlayerId, FlickSetupSummary>,
    recent_dodge_starts: HashMap<PlayerId, RecentDodgeStart>,
    pending_flicks: Vec<PendingFlick>,
    previous_dodge_active: HashMap<PlayerId, bool>,
    previous_ball_velocity: Option<glam::Vec3>,
    /// Frame of the dodge start behind the most recent flick emitted for each
    /// player, used to enforce one flick per dodge. Frame numbers are monotonic,
    /// so a stored frame never collides with a later dodge.
    last_emitted_dodge_frame: HashMap<PlayerId, usize>,
}

impl FlickCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[FlickEvent] {
        self.events.all()
    }

    pub fn new_events(&self) -> &[FlickEvent] {
        self.events.new_events()
    }

    fn normalize_score(value: f32, min_value: f32, max_value: f32) -> f32 {
        if max_value <= min_value {
            return 0.0;
        }

        ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0)
    }

    /// Velocity change imparted to the ball between `reference_velocity` and
    /// `current_velocity`, with gravity over `elapsed` removed. With
    /// `elapsed == dt` and the previous frame's velocity this is the
    /// single-frame impulse; with a longer `elapsed` it measures the change
    /// accumulated across a flick's contact window.
    fn gravity_compensated_impulse(
        current_velocity: glam::Vec3,
        reference_velocity: glam::Vec3,
        elapsed: f32,
    ) -> glam::Vec3 {
        let expected_linear_delta = glam::Vec3::new(0.0, 0.0, BALL_GRAVITY_Z * elapsed.max(0.0));
        current_velocity - reference_velocity - expected_linear_delta
    }

    fn control_observation(
        ball: &BallSample,
        player: &PlayerSample,
        controlling_player: Option<&PlayerId>,
    ) -> Option<FlickControlObservation> {
        if controlling_player != Some(&player.player_id) {
            return None;
        }

        let player_rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        let ball_position = ball.position();
        if !(BALL_CARRY_MIN_BALL_Z..=FLICK_MAX_CONTROL_BALL_Z).contains(&ball_position.z) {
            return None;
        }

        let horizontal_gap = player_position
            .truncate()
            .distance(ball_position.truncate());
        if horizontal_gap > FLICK_MAX_CONTROL_HORIZONTAL_GAP {
            return None;
        }

        let vertical_gap = ball_position.z - player_position.z;
        if !(FLICK_MIN_CONTROL_VERTICAL_GAP..=FLICK_MAX_CONTROL_VERTICAL_GAP)
            .contains(&vertical_gap)
        {
            return None;
        }

        // How closely the ball tracks the car horizontally this frame. A real
        // flick is set up by a carry/dribble where the ball rides the car, so
        // this stays small; a loose ball the car is merely driving at keeps its
        // own velocity. `None` when velocity data is unavailable so the carry
        // check downstream stays lenient on such replays.
        let relative_horizontal_speed = match (
            ball.rigid_body.linear_velocity.as_ref().map(vec_to_glam),
            player.velocity(),
        ) {
            (Some(ball_velocity), Some(player_velocity)) => {
                Some((ball_velocity.truncate() - player_velocity.truncate()).length())
            }
            _ => None,
        };

        let local_ball_position =
            quat_to_glam(&player_rigid_body.rotation).inverse() * (ball_position - player_position);
        if local_ball_position.x < -FLICK_MAX_LOCAL_X_BEHIND
            || local_ball_position.x > FLICK_MAX_LOCAL_X_FRONT
            || local_ball_position.y.abs() > FLICK_MAX_LOCAL_Y
            || local_ball_position.z < FLICK_MIN_LOCAL_Z
        {
            return None;
        }

        Some(FlickControlObservation {
            horizontal_gap,
            vertical_gap,
            relative_horizontal_speed,
        })
    }

    fn setup_summary(setup: &ActiveFlickSetup) -> FlickSetupSummary {
        FlickSetupSummary {
            is_team_0: setup.is_team_0,
            start_time: setup.start_time,
            start_frame: setup.start_frame,
            last_time: setup.last_time,
            last_frame: setup.last_frame,
            duration: setup.duration,
            average_horizontal_gap: setup.horizontal_gap_integral
                / setup.duration.max(f32::EPSILON),
            average_vertical_gap: setup.vertical_gap_integral / setup.duration.max(f32::EPSILON),
            touch_count: setup.touch_count,
            min_relative_horizontal_speed: setup.min_relative_horizontal_speed,
            observed_velocity: setup.observed_velocity,
        }
    }

    /// Whether a setup shows genuine carry/dribble evidence: at some non-dodge
    /// frame the ball tracked the car closely. Lenient when no velocity data was
    /// available at all (replays without velocities keep prior behavior), but a
    /// setup that *has* velocity data yet never shows a non-dodge carry — a car
    /// that drove into a loose ball while already dodging — is rejected. This is
    /// what separates a flick off a dribble from a dodge into a loose ball that
    /// merely passed through the control volume.
    fn setup_shows_carry(setup: &FlickSetupSummary) -> bool {
        !setup.observed_velocity
            || setup.min_relative_horizontal_speed <= FLICK_MAX_CARRY_REL_HORIZONTAL_SPEED
    }

    fn setup_qualifies(setup: &FlickSetupSummary) -> bool {
        setup.duration >= FLICK_MIN_SETUP_SECONDS
    }

    /// Ball-relative geometry of the touch, in the dodge reference frame. Returns
    /// `(local_ball_position, local_ball_impulse)`.
    fn local_ball_geometry(
        player_rotation: glam::Quat,
        rotation_at_dodge: Option<glam::Quat>,
        relative_ball_position: glam::Vec3,
        ball_impulse: glam::Vec3,
    ) -> (glam::Vec3, glam::Vec3) {
        let local_ball_position = player_rotation.inverse() * relative_ball_position;
        let impulse_reference_rotation = rotation_at_dodge.unwrap_or(player_rotation);
        let local_ball_impulse = impulse_reference_rotation.inverse() * ball_impulse;
        (local_ball_position, local_ball_impulse)
    }

    /// Classify the flick from the dodge's rotation axis — no ball impulse.
    ///
    /// `dodge_torque` is the flip's rotation axis in the **car's body frame**
    /// (decoded from controlled cancel-free flips, align 0.998 against the
    /// observed spin axis). Its two horizontal components map straight to the
    /// dodge the player input — no world/travel frame, because the torque is
    /// already car-relative:
    /// - `y` = `dodge_forward_back` (>0 forward dodge, <0 backflip),
    /// - `-x` = `dodge_side` (signed; +right, -left, see
    ///   [`FLICK_DODGE_SIDE_RIGHT_SIGN`]).
    ///
    /// (The earlier travel-frame decomposition — dotting this car-relative vector
    /// against the world velocity heading — was a frame error that made the
    /// result depend on which way the car faced in the world.)
    ///
    /// Because the axis is a unit vector in that plane,
    /// `dodge_forward_back² + dodge_side² ≈ 1`. A reverse flick is a sufficiently
    /// backward dodge that *also* rolled the car onto its side and launched the
    /// ball forward (so a plain backflip — which does neither — is not a reverse
    /// flick, see [`REVERSE_FLICK_MIN_LAUNCH_FORWARD`],
    /// [`REVERSE_FLICK_MAX_LAUNCH_VERTICAL_FRACTION`], and
    /// [`REVERSE_FLICK_MIN_UNDERSIDE_ROTATION`]); a side flick is
    /// sideways-dominant; a forward flick is forward. Returns
    /// `(kind, direction, dodge_forward_back, dodge_side)`.
    fn classify_dodge(
        dodge_torque: Option<glam::Vec3>,
        launch_forward: f32,
        launch_vertical_fraction: f32,
        underside_rotation: f32,
    ) -> (FlickKind, FlickDirection, f32, f32) {
        let Some(torque) = dodge_torque else {
            return (FlickKind::Other, FlickDirection::Center, 0.0, 0.0);
        };
        let torque_horizontal = torque.truncate();
        if torque_horizontal.length_squared() <= f32::EPSILON {
            return (FlickKind::Other, FlickDirection::Center, 0.0, 0.0);
        }

        let t = torque_horizontal.normalize();
        // Car-relative axis: `y` is forward/back (+forward), `x` is the side the
        // car dodged toward (+left, -right, from the controlled-flip replay).
        // `dodge_side` negates `x` so that, like the rest of the codebase, a
        // positive value means *right*.
        let dodge_forward_back = t.y;
        let dodge_side = -t.x;
        let handed_side = dodge_side * FLICK_DODGE_SIDE_RIGHT_SIGN;

        let direction = if handed_side >= FLICK_DIRECTION_MIN_SIDE {
            FlickDirection::Right
        } else if handed_side <= -FLICK_DIRECTION_MIN_SIDE {
            FlickDirection::Left
        } else {
            FlickDirection::Center
        };

        let kind = if dodge_forward_back <= -REVERSE_FLICK_MIN_BACKWARD
            && launch_forward >= REVERSE_FLICK_MIN_LAUNCH_FORWARD
            && launch_vertical_fraction <= REVERSE_FLICK_MAX_LAUNCH_VERTICAL_FRACTION
            && underside_rotation.abs() >= REVERSE_FLICK_MIN_UNDERSIDE_ROTATION
        {
            FlickKind::Reverse
        } else if dodge_side.abs() >= SIDE_FLICK_MIN_SIDE {
            FlickKind::Side
        } else if dodge_forward_back >= FORWARD_FLICK_MIN_FORWARD {
            FlickKind::Forward
        } else {
            FlickKind::Other
        };

        (kind, direction, dodge_forward_back, dodge_side)
    }

    fn store_recent_setup(&mut self, player_id: PlayerId, setup: FlickSetupSummary) {
        if Self::setup_qualifies(&setup) {
            self.recent_setups.insert(player_id, setup);
        }
    }

    fn finish_setup(&mut self, player_id: &PlayerId) {
        let Some(setup) = self.active_setups.remove(player_id) else {
            return;
        };
        self.store_recent_setup(player_id.clone(), Self::setup_summary(&setup));
    }

    fn recent_setup_for_player(
        &self,
        player_id: &PlayerId,
        current_time: f32,
    ) -> Option<FlickSetupSummary> {
        if let Some(active) = self.active_setups.get(player_id) {
            return Some(Self::setup_summary(active));
        }

        self.recent_setups
            .get(player_id)
            .filter(|setup| current_time - setup.last_time <= FLICK_MAX_SETUP_STALE_SECONDS)
            .cloned()
    }

    fn update_control_setups(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
        controlling_player: Option<&PlayerId>,
    ) {
        let Some(ball) = ball.sample() else {
            let player_ids: Vec<_> = self.active_setups.keys().cloned().collect();
            for player_id in player_ids {
                self.finish_setup(&player_id);
            }
            return;
        };

        let mut observed_players = HashSet::new();
        for player in &players.players {
            let Some(observation) = Self::control_observation(ball, player, controlling_player)
            else {
                continue;
            };
            observed_players.insert(player.player_id.clone());
            let setup = self
                .active_setups
                .entry(player.player_id.clone())
                .or_insert_with(|| ActiveFlickSetup {
                    is_team_0: player.is_team_0,
                    start_time: (frame.time - frame.dt).max(0.0),
                    start_frame: frame.frame_number.saturating_sub(1),
                    last_time: frame.time,
                    last_frame: frame.frame_number,
                    duration: frame.dt.max(0.0),
                    horizontal_gap_integral: observation.horizontal_gap * frame.dt.max(0.0),
                    vertical_gap_integral: observation.vertical_gap * frame.dt.max(0.0),
                    touch_count: 0,
                    min_relative_horizontal_speed: f32::INFINITY,
                    observed_velocity: false,
                });

            // Carry evidence is the dribble *before* the flick. Once the player
            // is dodging, the ball is being struck, and its post-contact velocity
            // can transiently align with the car — so only frames where the
            // player is not dodging count toward the carry minimum.
            if let Some(relative_horizontal_speed) = observation.relative_horizontal_speed {
                setup.observed_velocity = true;
                // Carry evidence is the dribble *before* the flick. Once the
                // player is dodging the ball is being struck, and its
                // post-contact velocity can transiently align with the car — so
                // only frames where the player is not dodging count toward the
                // carry minimum. A setup whose control frames are *all* during a
                // dodge (a car that drove into a loose ball while already
                // flicking) therefore shows no carry and is rejected below.
                if !player.dodge_active {
                    setup.min_relative_horizontal_speed = setup
                        .min_relative_horizontal_speed
                        .min(relative_horizontal_speed);
                }
            }

            if setup.last_frame != frame.frame_number {
                setup.last_time = frame.time;
                setup.last_frame = frame.frame_number;
                setup.duration += frame.dt.max(0.0);
                setup.horizontal_gap_integral += observation.horizontal_gap * frame.dt.max(0.0);
                setup.vertical_gap_integral += observation.vertical_gap * frame.dt.max(0.0);
            }
        }

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            if let Some(setup) = self.active_setups.get_mut(player_id) {
                setup.touch_count += 1;
            }
        }

        let active_ids: Vec<_> = self.active_setups.keys().cloned().collect();
        for player_id in active_ids {
            if observed_players.contains(&player_id) {
                continue;
            }
            // Keep the setup alive across brief observation gaps; only finish it
            // once the ball has been out of the control volume long enough that
            // the carry is genuinely over.
            let gap_elapsed = self
                .active_setups
                .get(&player_id)
                .map(|setup| frame.time - setup.last_time > FLICK_SETUP_GAP_GRACE_SECONDS)
                .unwrap_or(true);
            if gap_elapsed {
                self.finish_setup(&player_id);
            }
        }
    }

    fn track_dodge_starts(&mut self, frame: &FrameInfo, players: &PlayerFrameState) {
        for player in &players.players {
            let was_dodge_active = self
                .previous_dodge_active
                .insert(player.player_id.clone(), player.dodge_active)
                .unwrap_or(false);
            if !player.dodge_active || was_dodge_active {
                continue;
            }

            let Some(setup) = self.recent_setup_for_player(&player.player_id, frame.time) else {
                continue;
            };
            if !Self::setup_qualifies(&setup) {
                continue;
            }
            if !Self::setup_shows_carry(&setup) {
                continue;
            }
            if frame.time - setup.last_time > FLICK_MAX_CONTROL_TO_DODGE_SECONDS {
                continue;
            }

            self.recent_dodge_starts.insert(
                player.player_id.clone(),
                Self::dodge_start(frame.time, frame.frame_number, setup, player),
            );
        }
    }

    /// Build a [`RecentDodgeStart`] from the player's state at the dodge,
    /// snapshotting the dodge reference rotation, the world-frame dodge torque,
    /// and the horizontal travel direction the dodge is measured against.
    fn dodge_start(
        time: f32,
        frame: usize,
        setup: FlickSetupSummary,
        player: &PlayerSample,
    ) -> RecentDodgeStart {
        RecentDodgeStart {
            time,
            frame,
            setup,
            rotation_at_dodge: player
                .rigid_body
                .as_ref()
                .map(|rigid_body| quat_to_glam(&rigid_body.rotation)),
            dodge_torque: player.dodge_torque,
        }
    }

    fn prune_recent_state(&mut self, current_time: f32) {
        self.recent_setups
            .retain(|_, setup| current_time - setup.last_time <= FLICK_MAX_SETUP_STALE_SECONDS);
        self.recent_dodge_starts
            .retain(|_, dodge| current_time - dodge.time <= FLICK_MAX_DODGE_TO_TOUCH_SECONDS);
    }

    fn candidate_event(
        &self,
        ball: &BallFrameState,
        player: &PlayerSample,
        touch_event: &TouchEvent,
        dodge_start: &RecentDodgeStart,
        ball_impulse: glam::Vec3,
    ) -> Option<FlickEvent> {
        let ball = ball.sample()?;
        let player_rigid_body = player.rigid_body.as_ref()?;
        let player_position = player.position()?;
        let time_since_dodge = touch_event.time - dodge_start.time;
        if !(-FLICK_DODGE_LEAD_TOLERANCE_SECONDS..=FLICK_MAX_DODGE_TO_TOUCH_SECONDS)
            .contains(&time_since_dodge)
        {
            return None;
        }

        let ball_speed_change = ball_impulse.length();
        if ball_speed_change < FLICK_MIN_BALL_SPEED_CHANGE {
            return None;
        }

        let to_ball = (ball.position() - player_position).normalize_or_zero();
        let impulse_direction = ball_impulse.normalize_or_zero();
        if to_ball.length_squared() <= f32::EPSILON
            || impulse_direction.length_squared() <= f32::EPSILON
        {
            return None;
        }

        let impulse_away_alignment = impulse_direction.dot(to_ball);
        if impulse_away_alignment < FLICK_MIN_IMPULSE_AWAY_ALIGNMENT {
            return None;
        }

        let vertical_impulse = ball_impulse.z.max(0.0);
        let player_rotation = quat_to_glam(&player_rigid_body.rotation);
        let (local_ball_position, local_ball_impulse) = Self::local_ball_geometry(
            player_rotation,
            dodge_start.rotation_at_dodge,
            ball.position() - player_position,
            ball_impulse,
        );
        // How far the car was turned off its line of travel at the launch touch
        // (x/y plane): signed angle from the car's facing to its velocity
        // heading. + = velocity is to the car's left of where it points.
        let travel_offset_radians = player
            .velocity()
            .map(|velocity| {
                let forward = (player_rotation * glam::Vec3::X).truncate();
                let heading = velocity.truncate();
                if forward.length() > 0.3 && heading.length() > 50.0 {
                    let forward = forward.normalize();
                    let heading = heading.normalize();
                    (forward.x * heading.y - forward.y * heading.x).atan2(forward.dot(heading))
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        // How forward the ball was launched, along the run's travel heading
        // (x/y). Distinguishes a reverse flick (ball thrown forward) from a plain
        // backflip (ball not thrown forward); gates the reverse classification.
        let launch_forward_alignment = player
            .velocity()
            .map(|velocity| {
                let launch = ball_impulse.truncate();
                let heading = velocity.truncate();
                if launch.length() > f32::EPSILON && heading.length() > 50.0 {
                    launch.normalize().dot(heading.normalize())
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        // How vertical the launch was: the fraction of the launch impulse that is
        // upward (sin of the launch elevation). A reverse flick sends the ball
        // forward and flat (low); a plain backflip pops it up (high).
        let launch_vertical_fraction = {
            let mag = ball_impulse.length();
            if mag > f32::EPSILON {
                ball_impulse.z / mag
            } else {
                0.0
            }
        };
        // How far the car has rotated onto its side/back at the touch: + = rolled
        // to its right (right axis dips below horizontal), magnitude ~sin(roll).
        // Distinguishes a reverse flick (car rolled) from a plain backflip.
        let underside_rotation = -(player_rotation * glam::Vec3::Y).z;
        let (kind, direction, dodge_forward_back, dodge_side) = Self::classify_dodge(
            dodge_start.dodge_torque,
            launch_forward_alignment,
            launch_vertical_fraction,
            underside_rotation,
        );
        let setup = &dodge_start.setup;
        let timing_score =
            1.0 - (time_since_dodge / FLICK_MAX_DODGE_TO_TOUCH_SECONDS).clamp(0.0, 1.0);
        let setup_duration_score =
            Self::normalize_score(setup.duration, FLICK_MIN_SETUP_SECONDS, 0.75);
        let horizontal_control_score =
            1.0 - (setup.average_horizontal_gap / FLICK_MAX_CONTROL_HORIZONTAL_GAP).clamp(0.0, 1.0);
        let vertical_control_score = 1.0
            - ((setup.average_vertical_gap - 110.0).abs() / FLICK_MAX_CONTROL_VERTICAL_GAP)
                .clamp(0.0, 1.0);
        let impulse_score =
            Self::normalize_score(ball_speed_change, FLICK_MIN_BALL_SPEED_CHANGE, 1450.0);
        let away_score = Self::normalize_score(
            impulse_away_alignment,
            FLICK_MIN_IMPULSE_AWAY_ALIGNMENT,
            0.85,
        );
        let vertical_score = Self::normalize_score(vertical_impulse, 100.0, 750.0);

        let confidence = 0.16 * timing_score
            + 0.19 * setup_duration_score
            + 0.12 * horizontal_control_score
            + 0.10 * vertical_control_score
            + 0.22 * impulse_score
            + 0.15 * away_score
            + 0.06 * vertical_score;
        if confidence < FLICK_MIN_CONFIDENCE {
            return None;
        }

        Some(FlickEvent {
            time: touch_event.time,
            frame: touch_event.frame,
            sample_time: touch_event.time,
            sample_frame: touch_event.frame,
            player: player.player_id.clone(),
            player_position: Some(player_position.to_array()),
            is_team_0: player.is_team_0,
            dodge_time: dodge_start.time,
            dodge_frame: dodge_start.frame,
            time_since_dodge,
            setup_start_time: setup.start_time,
            setup_start_frame: setup.start_frame,
            setup_duration: setup.duration,
            setup_touch_count: setup.touch_count,
            average_horizontal_gap: setup.average_horizontal_gap,
            average_vertical_gap: setup.average_vertical_gap,
            ball_speed_change,
            ball_impulse: ball_impulse.to_array(),
            impulse_away_alignment,
            vertical_impulse,
            kind: kind.as_label_value().to_owned(),
            direction: direction.as_label_value().to_owned(),
            local_ball_position: local_ball_position.to_array(),
            local_ball_impulse: local_ball_impulse.to_array(),
            dodge_forward_back,
            dodge_side,
            dodge_torque: dodge_start.dodge_torque.map(|torque| torque.to_array()),
            travel_offset_radians,
            launch_forward_alignment,
            launch_vertical_fraction,
            underside_rotation,
            confidence,
        })
    }

    fn apply_event(&mut self, frame: &FrameInfo, mut event: FlickEvent) {
        event.sample_time = frame.time;
        event.sample_frame = frame.frame_number;
        self.events.push(event);
    }

    fn dodge_start_for_touch(&self, player: &PlayerSample) -> Option<RecentDodgeStart> {
        if let Some(dodge_start) = self.recent_dodge_starts.get(&player.player_id) {
            return Some(dodge_start.clone());
        }
        None
    }

    fn classified_as_dodge_touch(
        touch_event: &TouchEvent,
        touch_classification_events: &[TouchClassificationEvent],
    ) -> bool {
        let Some(touch_player) = touch_event.player.as_ref() else {
            return false;
        };
        touch_classification_events.iter().any(|event| {
            let same_touch = match (event.touch_id, touch_event.touch_id) {
                (Some(event_id), Some(touch_id)) => event_id == touch_id,
                _ => event.player == *touch_player && event.frame == touch_event.frame,
            };
            same_touch && event.has_tag("dodge_state", "dodge")
        })
    }

    fn pending_dodge_start_for_touch(
        &self,
        player: &PlayerSample,
        touch_event: &TouchEvent,
    ) -> Option<RecentDodgeStart> {
        let setup = self.recent_setup_for_player(&player.player_id, touch_event.time)?;
        if setup.duration < FLICK_MIN_PENDING_DODGE_SETUP_SECONDS {
            return None;
        }
        if !Self::setup_shows_carry(&setup) {
            return None;
        }
        Some(Self::dodge_start(
            touch_event.time,
            touch_event.frame,
            setup,
            player,
        ))
    }

    /// Open (or refresh) a pending flick for a touch by a player who has a
    /// recent control setup (i.e. was dribbling/carrying). The pending entry is
    /// what lets the detector watch the ball's velocity change across the
    /// [`FLICK_IMPULSE_WINDOW_SECONDS`] window rather than only at the touch
    /// frame.
    fn store_pending_flick(
        &mut self,
        ball: &BallFrameState,
        player: &PlayerSample,
        touch_event: &TouchEvent,
        pre_velocity: glam::Vec3,
    ) {
        let already_tracked = self.pending_flicks.iter().any(|pending| {
            pending.touch_event.touch_id == touch_event.touch_id
                && pending.touch_event.player == touch_event.player
                && pending.touch_event.frame == touch_event.frame
        });
        if already_tracked {
            // Same touch reappearing on a later frame: keep accumulating into
            // its existing window rather than resetting it.
            return;
        }
        // Require at least the loose pending-setup threshold; the stricter
        // dodge/confidence gates are enforced when the window resolves.
        let has_setup = self
            .recent_setup_for_player(&player.player_id, touch_event.time)
            .is_some_and(|setup| setup.duration >= FLICK_MIN_PENDING_DODGE_SETUP_SECONDS);
        if !has_setup {
            return;
        }
        // One flick per dodge: a newer touch by the same player supersedes its
        // earlier window so a single dribble cannot emit multiple flicks when
        // its control touches fall within one impulse window of each other.
        // A still-live pending being superseded is the same contact episode —
        // it can only exist within the short retention window, and the touch
        // rate limit only lets a same-player pair through that fast when a
        // dodge-powered launch follows a passive contact. The new pending
        // inherits the earlier measurement anchor instead of resetting it, so
        // the launch impulse already delivered before this touch stays in the
        // measured peak.
        let inherited_measurement = self
            .pending_flicks
            .iter()
            .find(|pending| {
                pending.player.player_id == player.player_id
                    && touch_event.time >= pending.touch_event.time
            })
            .map(|pending| {
                (
                    pending.measure_start_time,
                    pending.pre_velocity,
                    pending.peak_impulse,
                    pending.peak_magnitude,
                )
            });
        self.pending_flicks
            .retain(|pending| pending.player.player_id != player.player_id);
        let (measure_start_time, pre_velocity, peak_impulse, peak_magnitude) =
            inherited_measurement.unwrap_or((
                touch_event.time,
                pre_velocity,
                glam::Vec3::ZERO,
                0.0,
            ));
        self.pending_flicks.push(PendingFlick {
            touch_event: touch_event.clone(),
            ball: ball.clone(),
            player: player.clone(),
            real_dodge_start: self.dodge_start_for_touch(player),
            classified_dodge: false,
            measure_start_time,
            pre_velocity,
            peak_impulse,
            peak_magnitude,
        });
    }

    /// Per-frame step: grow each pending flick's peak impulse from the live ball
    /// velocity, refresh its dodge evidence, and emit as soon as the peak clears
    /// the gates. Entries that never qualify are dropped once the window closes.
    fn update_and_resolve_pending_flicks(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        touch_classification_events: &[TouchClassificationEvent],
    ) {
        let current_velocity = ball.velocity();
        let mut pending = std::mem::take(&mut self.pending_flicks);
        let mut emitted = Vec::new();
        pending.retain_mut(|flick| {
            let elapsed = (frame.time - flick.touch_event.time).max(0.0);
            if elapsed > FLICK_PENDING_RETENTION_SECONDS {
                return false;
            }

            // Measure the impulse only over the (shorter) impulse window,
            // anchored on the latest touch; the entry is kept alive past it
            // purely so a late-replicating dodge byte can still confirm the
            // launch touch. Gravity compensation spans from the episode's
            // measurement anchor, which precedes the touch when this pending
            // inherited an earlier same-episode window.
            if elapsed <= FLICK_IMPULSE_WINDOW_SECONDS {
                if let Some(velocity) = current_velocity {
                    let measure_elapsed = (frame.time - flick.measure_start_time).max(0.0);
                    let impulse = Self::gravity_compensated_impulse(
                        velocity,
                        flick.pre_velocity,
                        measure_elapsed,
                    );
                    let magnitude = impulse.length();
                    if magnitude > flick.peak_magnitude {
                        flick.peak_magnitude = magnitude;
                        flick.peak_impulse = impulse;
                    }
                }
            }

            if flick.real_dodge_start.is_none() {
                flick.real_dodge_start = self.dodge_start_for_touch(&flick.player);
            }
            if !flick.classified_dodge {
                flick.classified_dodge = Self::classified_as_dodge_touch(
                    &flick.touch_event,
                    touch_classification_events,
                );
            }

            // A genuine dodge transition stands on its own; otherwise the touch
            // must have been classified as a dodge contact (the old pending
            // path) and have a recent control setup to synthesize a start from.
            let dodge_start = flick.real_dodge_start.clone().or_else(|| {
                if flick.classified_dodge {
                    self.pending_dodge_start_for_touch(&flick.player, &flick.touch_event)
                } else {
                    None
                }
            });
            let Some(dodge_start) = dodge_start else {
                return true;
            };

            // One flick per dodge. Extending the pending window so a late dodge
            // byte can confirm an earlier launch touch means several touches that
            // bracket a single dodge (a pre-dodge carry contact and the launch)
            // can each resolve against the same dodge start. Drop any candidate
            // for a dodge that already produced a flick — whether emitted on an
            // earlier frame or earlier in this same frame's batch — so the first
            // qualifying touch wins and the dodge is not double-counted.
            let already_emitted = self.last_emitted_dodge_frame.get(&flick.player.player_id)
                == Some(&dodge_start.frame)
                || emitted.iter().any(|event: &FlickEvent| {
                    event.player == flick.player.player_id && event.dodge_frame == dodge_start.frame
                });
            if already_emitted {
                return false;
            }

            if let Some(event) = self.candidate_event(
                &flick.ball,
                &flick.player,
                &flick.touch_event,
                &dodge_start,
                flick.peak_impulse,
            ) {
                emitted.push(event);
                return false;
            }
            true
        });
        self.pending_flicks = pending;
        for event in emitted {
            self.last_emitted_dodge_frame
                .insert(event.player.clone(), event.dodge_frame);
            self.apply_event(frame, event);
        }
    }

    fn apply_touch_events(
        &mut self,
        _frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_events: &[TouchEvent],
    ) {
        let pre_velocity = self
            .previous_ball_velocity
            .or_else(|| ball.velocity())
            .unwrap_or(glam::Vec3::ZERO);

        for touch_event in touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            let Some(player) = players
                .players
                .iter()
                .find(|player| &player.player_id == player_id)
            else {
                continue;
            };
            // Open a measurement window for any touch by a dribbling player; the
            // impulse, dodge, and confidence gates resolve over the window in
            // `update_and_resolve_pending_flicks`.
            self.store_pending_flick(ball, player, touch_event, pre_velocity);
        }
    }

    fn reset_live_play_state(&mut self, ball: &BallFrameState) {
        self.active_setups.clear();
        self.recent_setups.clear();
        self.recent_dodge_starts.clear();
        self.pending_flicks.clear();
        self.previous_dodge_active.clear();
        self.last_emitted_dodge_frame.clear();
        self.previous_ball_velocity = ball.velocity();
    }

    fn update_with_touch_classification_events(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        touch_classification_events: &[TouchClassificationEvent],
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.events.begin_update();
        if !live_play_state.is_live_play {
            self.reset_live_play_state(ball);
            return Ok(());
        }
        self.prune_recent_state(frame.time);
        self.update_control_setups(
            frame,
            ball,
            players,
            &touch_state.touch_events,
            touch_state.last_touch_player.as_ref(),
        );
        self.track_dodge_starts(frame, players);
        self.apply_touch_events(frame, ball, players, &touch_state.touch_events);
        self.update_and_resolve_pending_flicks(frame, ball, touch_classification_events);
        self.previous_ball_velocity = ball.velocity();
        Ok(())
    }

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        touch: &TouchCalculator,
        live_play_state: &LivePlayState,
    ) -> SubtrActorResult<()> {
        self.update_with_touch_classification_events(
            frame,
            ball,
            players,
            touch_state,
            touch.events(),
            live_play_state,
        )
    }
}

#[cfg(test)]
#[path = "flick_tests.rs"]
mod tests;
