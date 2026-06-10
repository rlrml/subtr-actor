use super::*;
use crate::stats::calculators::*;
use crate::*;

#[derive(Debug, Clone, Default)]
pub struct StatsTimelineEventsState {
    pub events: ReplayStatsTimelineEvents,
}

const MECHANIC_AIR_DRIBBLE: &str = "air_dribble";
const MECHANIC_BALL_CARRY: &str = "ball_carry";
const MECHANIC_CEILING_SHOT: &str = "ceiling_shot";
const MECHANIC_CENTER: &str = "center";
const MECHANIC_DOUBLE_TAP: &str = "double_tap";
const MECHANIC_FLICK: &str = "flick";
const MECHANIC_FLIP_RESET: &str = "flip_reset";
const MECHANIC_HALF_FLIP: &str = "half_flip";
const MECHANIC_HALF_VOLLEY: &str = "half_volley";
const MECHANIC_MUSTY_FLICK: &str = "musty_flick";
const MECHANIC_ONE_TIMER: &str = "one_timer";
const MECHANIC_PASS: &str = "pass";
const MECHANIC_SPEED_FLIP: &str = "speed_flip";
const MECHANIC_WALL_AERIAL: &str = "wall_aerial";
const MECHANIC_WALL_AERIAL_SHOT: &str = "wall_aerial_shot";
const MECHANIC_WAVEDASH: &str = "wavedash";

pub const STATS_TIMELINE_MECHANIC_KINDS: &[&str] = &[
    MECHANIC_AIR_DRIBBLE,
    MECHANIC_BALL_CARRY,
    MECHANIC_CEILING_SHOT,
    MECHANIC_CENTER,
    MECHANIC_DOUBLE_TAP,
    MECHANIC_FLICK,
    MECHANIC_FLIP_RESET,
    MECHANIC_HALF_FLIP,
    MECHANIC_HALF_VOLLEY,
    MECHANIC_MUSTY_FLICK,
    MECHANIC_ONE_TIMER,
    MECHANIC_PASS,
    MECHANIC_SPEED_FLIP,
    MECHANIC_WALL_AERIAL,
    MECHANIC_WALL_AERIAL_SHOT,
    MECHANIC_WAVEDASH,
];

pub struct StatsTimelineEventsNode {
    state: StatsTimelineEventsState,
}

impl StatsTimelineEventsNode {
    pub fn new() -> Self {
        Self {
            state: StatsTimelineEventsState::default(),
        }
    }

    fn dependencies() -> NodeDependencies {
        vec![
            frame_info_dependency(),
            gameplay_state_dependency(),
            live_play_dependency(),
            match_stats_dependency(),
            // Keep compact event transfer independent from full partial-sum projection.
            backboard_dependency(),
            ceiling_shot_dependency(),
            wall_aerial_dependency(),
            wall_aerial_shot_dependency(),
            double_tap_dependency(),
            one_timer_dependency(),
            pass_dependency(),
            controlled_play_dependency(),
            fifty_fifty_dependency(),
            kickoff_dependency(),
            possession_dependency(),
            ball_half_dependency(),
            territorial_pressure_dependency(),
            rotation_dependency(),
            rush_dependency(),
            touch_dependency(),
            whiff_dependency(),
            wavedash_dependency(),
            flip_impulse_dependency(),
            speed_flip_dependency(),
            half_flip_dependency(),
            flick_dependency(),
            musty_flick_dependency(),
            dodge_reset_dependency(),
            ball_carry_dependency(),
            boost_dependency(),
            bump_dependency(),
            half_volley_dependency(),
            movement_dependency(),
            positioning_dependency(),
            powerslide_dependency(),
            demo_dependency(),
            center_dependency(),
            aerial_goal_dependency(),
            high_aerial_goal_dependency(),
            long_distance_goal_dependency(),
            own_half_goal_dependency(),
            empty_net_goal_dependency(),
            counter_attack_goal_dependency(),
            flick_goal_dependency(),
            ceiling_shot_goal_dependency(),
            double_tap_goal_dependency(),
            one_timer_goal_dependency(),
            passing_goal_dependency(),
            air_dribble_goal_dependency(),
            flip_reset_goal_dependency(),
            flip_into_ball_goal_dependency(),
            bump_goal_dependency(),
            demo_goal_dependency(),
            half_volley_goal_dependency(),
            kickoff_goal_dependency(),
        ]
    }

    fn capture_events(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let match_stats = ctx.get::<MatchStatsCalculator>()?;
        let possession = ctx.get::<PossessionCalculator>()?;
        let ball_half = ctx.get::<BallHalfCalculator>()?;
        let territorial_pressure = ctx.get::<TerritorialPressureCalculator>()?;
        let movement = ctx.get::<MovementCalculator>()?;
        let positioning = ctx.get::<PositioningCalculator>()?;
        let rotation = ctx.get::<RotationCalculator>()?;
        let demo = ctx.get::<DemoCalculator>()?;
        let backboard = ctx.get::<BackboardCalculator>()?;
        let ball_carry = ctx.get::<BallCarryCalculator>()?;
        let ceiling_shot = ctx.get::<CeilingShotCalculator>()?;
        let wall_aerial = ctx.get::<WallAerialCalculator>()?;
        let wall_aerial_shot = ctx.get::<WallAerialShotCalculator>()?;
        let center = ctx.get::<CenterCalculator>()?;
        let dodge_reset = ctx.get::<DodgeResetCalculator>()?;
        let double_tap = ctx.get::<DoubleTapCalculator>()?;
        let one_timer = ctx.get::<OneTimerCalculator>()?;
        let pass = ctx.get::<PassCalculator>()?;
        let controlled_play = ctx.get::<ControlledPlayCalculator>()?;
        let fifty_fifty = ctx.get::<FiftyFiftyCalculator>()?;
        let kickoff = ctx.get::<KickoffCalculator>()?;
        let flick = ctx.get::<FlickCalculator>()?;
        let musty_flick = ctx.get::<MustyFlickCalculator>()?;
        let aerial_goal = ctx.get::<AerialGoalCalculator>()?;
        let high_aerial_goal = ctx.get::<HighAerialGoalCalculator>()?;
        let long_distance_goal = ctx.get::<LongDistanceGoalCalculator>()?;
        let own_half_goal = ctx.get::<OwnHalfGoalCalculator>()?;
        let empty_net_goal = ctx.get::<EmptyNetGoalCalculator>()?;
        let counter_attack_goal = ctx.get::<CounterAttackGoalCalculator>()?;
        let flick_goal = ctx.get::<FlickGoalCalculator>()?;
        let ceiling_shot_goal = ctx.get::<CeilingShotGoalCalculator>()?;
        let double_tap_goal = ctx.get::<DoubleTapGoalCalculator>()?;
        let one_timer_goal = ctx.get::<OneTimerGoalCalculator>()?;
        let passing_goal = ctx.get::<PassingGoalCalculator>()?;
        let air_dribble_goal = ctx.get::<AirDribbleGoalCalculator>()?;
        let flip_reset_goal = ctx.get::<FlipResetGoalCalculator>()?;
        let flip_into_ball_goal = ctx.get::<FlipIntoBallGoalCalculator>()?;
        let bump_goal = ctx.get::<BumpGoalCalculator>()?;
        let demo_goal = ctx.get::<DemoGoalCalculator>()?;
        let half_volley_goal = ctx.get::<HalfVolleyGoalCalculator>()?;
        let kickoff_goal = ctx.get::<KickoffGoalCalculator>()?;
        let rush = ctx.get::<RushCalculator>()?;
        let flip_impulse = ctx.get::<FlipImpulseCalculator>()?;
        let speed_flip = ctx.get::<SpeedFlipCalculator>()?;
        let half_flip = ctx.get::<HalfFlipCalculator>()?;
        let half_volley = ctx.get::<HalfVolleyCalculator>()?;
        let wavedash = ctx.get::<WavedashCalculator>()?;
        let whiff = ctx.get::<WhiffCalculator>()?;
        let powerslide = ctx.get::<PowerslideCalculator>()?;
        let touch = ctx.get::<TouchCalculator>()?;
        let boost = ctx.get::<BoostCalculator>()?;
        let bump = ctx.get::<BumpCalculator>()?;

        let mut timeline = match_stats.timeline().to_vec();
        timeline.extend(demo.timeline().to_vec());
        timeline.sort_by(|left, right| left.time.total_cmp(&right.time));
        let goal_tag_assignments = combined_goal_tag_assignments(&[
            aerial_goal.events(),
            high_aerial_goal.events(),
            long_distance_goal.events(),
            own_half_goal.events(),
            empty_net_goal.events(),
            counter_attack_goal.events(),
            flick_goal.events(),
            ceiling_shot_goal.events(),
            double_tap_goal.events(),
            one_timer_goal.events(),
            passing_goal.events(),
            air_dribble_goal.events(),
            flip_reset_goal.events(),
            flip_into_ball_goal.events(),
            bump_goal.events(),
            demo_goal.events(),
            half_volley_goal.events(),
            kickoff_goal.events(),
        ]);
        let goal_context =
            goal_context_events_with_tags(match_stats.goal_context_events(), &goal_tag_assignments);

        self.state.events = ReplayStatsTimelineEvents {
            events: build_replay_events(
                &timeline,
                match_stats,
                possession,
                ball_half,
                territorial_pressure,
                movement,
                positioning,
                rotation,
                &goal_context,
                backboard,
                ball_carry,
                ceiling_shot,
                wall_aerial,
                wall_aerial_shot,
                center,
                dodge_reset,
                double_tap,
                one_timer,
                pass,
                controlled_play,
                fifty_fifty,
                kickoff,
                rush,
                flip_impulse,
                speed_flip,
                half_flip,
                half_volley,
                wavedash,
                whiff,
                powerslide,
                touch,
                boost,
                bump,
                flick,
                musty_flick,
            ),
        };
        Ok(())
    }
}

impl Default for StatsTimelineEventsNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNode for StatsTimelineEventsNode {
    type State = StatsTimelineEventsState;

    fn name(&self) -> &'static str {
        "stats_timeline_events"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        Self::dependencies()
    }

    fn evaluate(&mut self, _ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        Ok(())
    }

    fn finish(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        self.capture_events(ctx)
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

fn moment(frame: usize, time: f32) -> EventTiming {
    EventTiming::Moment { frame, time }
}

fn span(start_frame: usize, end_frame: usize, start_time: f32, end_time: f32) -> EventTiming {
    EventTiming::Span {
        start_frame,
        end_frame,
        start_time,
        end_time,
    }
}

#[allow(clippy::too_many_arguments)]
fn make_event(
    stream: &str,
    index: usize,
    timing: EventTiming,
    payload: EventPayload,
    primary_player: Option<PlayerId>,
    secondary_player: Option<PlayerId>,
    team_is_team_0: Option<bool>,
    player_position: Option<[f32; 3]>,
    ball_position: Option<[f32; 3]>,
    confidence: Option<f32>,
) -> Event {
    let frame_id = match timing {
        EventTiming::Moment { frame, .. } => frame.to_string(),
        EventTiming::Span {
            start_frame,
            end_frame,
            ..
        } => format!("{start_frame}:{end_frame}"),
    };
    Event {
        meta: EventMeta {
            id: format!("{stream}:{frame_id}:{index}"),
            stream: stream.to_owned(),
            label: stats_timeline_event_label(stream),
            timing,
            primary_player,
            secondary_player,
            player_position,
            ball_position,
            team_is_team_0,
            confidence,
            properties: Vec::new(),
        },
        payload,
    }
}

fn event_start_time(event: &Event) -> f32 {
    match event.meta.timing {
        EventTiming::Moment { time, .. } => time,
        EventTiming::Span { start_time, .. } => start_time,
    }
}

#[allow(clippy::too_many_arguments, clippy::cognitive_complexity)]
fn build_replay_events(
    timeline: &[TimelineEvent],
    match_stats: &MatchStatsCalculator,
    possession: &PossessionCalculator,
    ball_half: &BallHalfCalculator,
    territorial_pressure: &TerritorialPressureCalculator,
    movement: &MovementCalculator,
    positioning: &PositioningCalculator,
    rotation: &RotationCalculator,
    goal_context: &[GoalContextEvent],
    backboard: &BackboardCalculator,
    ball_carry: &BallCarryCalculator,
    ceiling_shot: &CeilingShotCalculator,
    wall_aerial: &WallAerialCalculator,
    wall_aerial_shot: &WallAerialShotCalculator,
    center: &CenterCalculator,
    dodge_reset: &DodgeResetCalculator,
    double_tap: &DoubleTapCalculator,
    one_timer: &OneTimerCalculator,
    pass: &PassCalculator,
    controlled_play: &ControlledPlayCalculator,
    fifty_fifty: &FiftyFiftyCalculator,
    kickoff: &KickoffCalculator,
    rush: &RushCalculator,
    flip_impulse: &FlipImpulseCalculator,
    speed_flip: &SpeedFlipCalculator,
    half_flip: &HalfFlipCalculator,
    half_volley: &HalfVolleyCalculator,
    wavedash: &WavedashCalculator,
    whiff: &WhiffCalculator,
    powerslide: &PowerslideCalculator,
    touch: &TouchCalculator,
    boost: &BoostCalculator,
    bump: &BumpCalculator,
    flick: &FlickCalculator,
    musty_flick: &MustyFlickCalculator,
) -> Vec<Event> {
    let mut events = Vec::new();

    for (index, event) in timeline.iter().enumerate() {
        events.push(make_event(
            "timeline",
            index,
            moment(event.frame.unwrap_or_default(), event.time),
            EventPayload::Timeline(event.clone()),
            event.player_id.clone(),
            None,
            event.is_team_0,
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in match_stats.core_player_events().iter().enumerate() {
        events.push(make_event(
            "core_player",
            index,
            moment(event.frame, event.time),
            EventPayload::CorePlayer(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in match_stats
        .core_player_goal_context_events()
        .iter()
        .enumerate()
    {
        events.push(make_event(
            "core_player_goal_context",
            index,
            moment(event.frame, event.time),
            EventPayload::CorePlayerGoalContext(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in possession.events().iter().enumerate() {
        events.push(make_event(
            "possession",
            index,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::Possession(event.clone()),
            event.player_id.clone(),
            None,
            None,
            None,
            None,
            None,
        ));
    }

    for (index, event) in ball_half.events().iter().enumerate() {
        events.push(make_event(
            "ball_half",
            index,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::BallHalf(event.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
        ));
    }

    for (index, event) in territorial_pressure.events().iter().enumerate() {
        events.push(make_event(
            "territorial_pressure",
            index,
            span(
                event.start_frame,
                event.end_frame,
                event.start_time,
                event.end_time,
            ),
            EventPayload::TerritorialPressure(event.clone()),
            None,
            None,
            Some(event.team_is_team_0),
            None,
            None,
            None,
        ));
    }

    for (index, event) in movement.events().iter().enumerate() {
        events.push(make_event(
            "movement",
            index,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::Movement(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in positioning.activity_events().iter().enumerate() {
        events.push(make_event(
            "positioning_activity",
            index,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::PositioningActivity(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in positioning.field_zone_events().iter().enumerate() {
        events.push(make_event(
            "positioning_field_zone",
            index,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::PositioningFieldZone(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in positioning.ball_relative_depth_events().iter().enumerate() {
        events.push(make_event(
            "positioning_ball_relative_depth",
            index,
            moment(event.frame, event.time),
            EventPayload::PositioningBallRelativeDepth(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in positioning.teammate_role_events().iter().enumerate() {
        events.push(make_event(
            "positioning_teammate_role",
            index,
            moment(event.frame, event.time),
            EventPayload::PositioningTeammateRole(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in positioning.ball_proximity_events().iter().enumerate() {
        events.push(make_event(
            "positioning_ball_proximity",
            index,
            moment(event.frame, event.time),
            EventPayload::PositioningBallProximity(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in rotation.player_events().iter().enumerate() {
        events.push(make_event(
            "rotation_player",
            index,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::RotationPlayer(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in rotation.role_span_events().iter().enumerate() {
        events.push(make_event(
            "rotation_role_span",
            index,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::RotationRoleSpan(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in rotation.depth_span_events().iter().enumerate() {
        events.push(make_event(
            "rotation_depth_span",
            index,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::RotationDepthSpan(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in rotation.first_man_stint_events().iter().enumerate() {
        events.push(make_event(
            "rotation_first_man_stint",
            index,
            span(event.frame, event.end_frame, event.time, event.end_time),
            EventPayload::RotationFirstManStint(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in rotation.team_events().iter().enumerate() {
        events.push(make_event(
            "rotation_team",
            index,
            moment(event.frame, event.time),
            EventPayload::RotationTeam(event.clone()),
            Some(event.next_first_man.clone()),
            Some(event.previous_first_man.clone()),
            Some(event.is_team_0),
            None,
            None,
            None,
        ));
    }

    for (index, event) in goal_context.iter().enumerate() {
        events.push(make_event(
            "goal_context",
            index,
            moment(event.frame, event.time),
            EventPayload::GoalContext(event.clone()),
            event.scorer.clone(),
            None,
            Some(event.scoring_team_is_team_0),
            None,
            event
                .ball_position
                .map(|position| [position.x, position.y, position.z]),
            None,
        ));
    }

    for (index, event) in backboard.events().iter().enumerate() {
        events.push(make_event(
            "backboard",
            index,
            moment(event.frame, event.time),
            EventPayload::Backboard(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in ball_carry.carry_events().iter().enumerate() {
        events.push(make_event(
            match event.kind {
                BallCarryKind::Carry => MECHANIC_BALL_CARRY,
                BallCarryKind::AirDribble => MECHANIC_AIR_DRIBBLE,
            },
            index,
            span(
                event.start_frame,
                event.end_frame,
                event.start_time,
                event.end_time,
            ),
            EventPayload::BallCarry(event.clone()),
            Some(event.player_id.clone()),
            None,
            Some(event.is_team_0),
            Some(event.end_position),
            Some(event.end_position),
            None,
        ));
    }

    for (index, event) in ceiling_shot.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_CEILING_SHOT,
            index,
            span(
                event.ceiling_contact_frame,
                event.frame,
                event.ceiling_contact_time,
                event.time,
            ),
            EventPayload::CeilingShot(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            Some(event.touch_position),
            Some(event.confidence),
        ));
    }

    for (index, event) in wall_aerial.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_WALL_AERIAL,
            index,
            span(
                event.wall_contact_frame,
                event.frame,
                event.wall_contact_time,
                event.time,
            ),
            EventPayload::WallAerial(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            Some(event.player_position),
            Some(event.ball_position),
            Some(event.confidence),
        ));
    }

    for (index, event) in wall_aerial_shot.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_WALL_AERIAL_SHOT,
            index,
            span(
                event.takeoff_frame,
                event.frame,
                event.takeoff_time,
                event.time,
            ),
            EventPayload::WallAerialShot(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            Some(event.player_position),
            Some(event.ball_position),
            Some(event.confidence),
        ));
    }

    for (index, event) in center.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_CENTER,
            index,
            span(event.start_frame, event.frame, event.start_time, event.time),
            EventPayload::Center(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            Some(event.end_ball_position),
            None,
        ));
    }

    for (index, event) in dodge_reset.events().iter().enumerate() {
        events.push(make_event(
            "dodge_reset",
            index,
            moment(event.frame, event.time),
            EventPayload::DodgeReset(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in double_tap.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_DOUBLE_TAP,
            index,
            span(
                event.backboard_frame,
                event.frame,
                event.backboard_time,
                event.time,
            ),
            EventPayload::DoubleTap(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in one_timer.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_ONE_TIMER,
            index,
            span(
                event.pass_start_frame,
                event.frame,
                event.pass_start_time,
                event.time,
            ),
            EventPayload::OneTimer(event.clone()),
            Some(event.player.clone()),
            Some(event.passer.clone()),
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in pass.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_PASS,
            index,
            span(event.start_frame, event.frame, event.start_time, event.time),
            EventPayload::Pass(event.clone()),
            Some(event.passer.clone()),
            Some(event.receiver.clone()),
            Some(event.is_team_0),
            event.passer_position,
            None,
            None,
        ));
    }

    for (index, event) in controlled_play.events().iter().enumerate() {
        events.push(make_event(
            "controlled_play",
            index,
            span(
                event.start_frame,
                event.end_frame,
                event.start_time,
                event.end_time,
            ),
            EventPayload::ControlledPlay(event.clone()),
            Some(event.player_id.clone()),
            None,
            Some(event.is_team_0),
            None,
            None,
            None,
        ));
    }

    for (index, event) in fifty_fifty.events().iter().enumerate() {
        events.push(make_event(
            "fifty_fifty",
            index,
            span(
                event.start_frame,
                event.resolve_frame,
                event.start_time,
                event.resolve_time,
            ),
            EventPayload::FiftyFifty(event.clone()),
            event
                .team_zero_player
                .clone()
                .or_else(|| event.team_one_player.clone()),
            None,
            event.winning_team_is_team_0,
            None,
            Some(event.midpoint),
            None,
        ));
    }

    for (index, event) in kickoff.events().iter().enumerate() {
        events.push(make_event(
            "kickoff",
            index,
            span(
                event.start_frame,
                event.end_frame,
                event.start_time,
                event.end_time,
            ),
            EventPayload::Kickoff(Box::new(event.clone())),
            event.first_touch_player.clone(),
            None,
            event.first_touch_team_is_team_0,
            None,
            None,
            None,
        ));
    }

    for (index, event) in rush.events().iter().enumerate() {
        events.push(make_event(
            "rush",
            index,
            span(
                event.start_frame,
                event.end_frame,
                event.start_time,
                event.end_time,
            ),
            EventPayload::Rush(event.clone()),
            None,
            None,
            Some(event.is_team_0),
            None,
            None,
            None,
        ));
    }

    for (index, event) in flip_impulse.events().iter().enumerate() {
        events.push(make_event(
            "dodge",
            index,
            span(
                event.frame,
                event.resolved_frame,
                event.time,
                event.resolved_time,
            ),
            EventPayload::Dodge(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event
                .dodge_impulse
                .as_ref()
                .map(|dodge_impulse| dodge_impulse.end_position),
            None,
            event
                .dodge_impulse
                .as_ref()
                .map(|dodge_impulse| dodge_impulse.confidence),
        ));
    }

    for (index, event) in speed_flip.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_SPEED_FLIP,
            index,
            span(
                event.frame,
                event.resolved_frame,
                event.time,
                event.resolved_time,
            ),
            EventPayload::SpeedFlip(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            Some(event.end_position),
            None,
            Some(event.confidence),
        ));
    }

    for (index, event) in half_flip.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_HALF_FLIP,
            index,
            moment(event.frame, event.time),
            EventPayload::HalfFlip(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            Some(event.end_position),
            None,
            Some(event.confidence),
        ));
    }

    for (index, event) in half_volley.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_HALF_VOLLEY,
            index,
            moment(event.frame, event.time),
            EventPayload::HalfVolley(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in wavedash.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_WAVEDASH,
            index,
            span(event.dodge_frame, event.frame, event.dodge_time, event.time),
            EventPayload::Wavedash(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            Some(event.landing_position),
            None,
            Some(event.confidence),
        ));
    }

    for (index, event) in whiff.events().iter().enumerate() {
        events.push(make_event(
            "whiff",
            index,
            span(
                event.frame,
                event.resolved_frame,
                event.time,
                event.resolved_time,
            ),
            EventPayload::Whiff(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in powerslide.events().iter().enumerate() {
        events.push(make_event(
            "powerslide",
            index,
            moment(event.frame, event.time),
            EventPayload::Powerslide(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in touch.events().iter().enumerate() {
        let timing =
            event
                .ball_movement
                .as_ref()
                .map_or(moment(event.frame, event.time), |movement| {
                    span(
                        movement.start_frame,
                        movement.end_frame,
                        movement.start_time,
                        movement.end_time,
                    )
                });
        events.push(make_event(
            "touch",
            index,
            timing,
            EventPayload::Touch(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in boost.pickup_events().iter().enumerate() {
        events.push(make_event(
            "boost_pickups",
            index,
            moment(event.frame, event.time),
            EventPayload::BoostPickup(event.clone()),
            Some(event.player_id.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in boost.respawn_events().iter().enumerate() {
        events.push(make_event(
            "boost_respawn",
            index,
            moment(event.frame, event.time),
            EventPayload::Respawn(event.clone()),
            Some(event.player_id.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            None,
        ));
    }

    for (index, event) in bump.events().iter().enumerate() {
        events.push(make_event(
            "bump",
            index,
            moment(event.frame, event.time),
            EventPayload::Bump(event.clone()),
            Some(event.initiator.clone()),
            Some(event.victim.clone()),
            Some(event.initiator_is_team_0),
            Some(event.initiator_position),
            None,
            Some(event.confidence),
        ));
    }

    for (index, event) in flick.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_FLICK,
            index,
            span(
                event.setup_start_frame,
                event.frame,
                event.setup_start_time,
                event.time,
            ),
            EventPayload::Flick(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            Some(event.confidence),
        ));
    }

    for (index, event) in musty_flick.events().iter().enumerate() {
        events.push(make_event(
            MECHANIC_MUSTY_FLICK,
            index,
            span(event.dodge_frame, event.frame, event.dodge_time, event.time),
            EventPayload::MustyFlick(event.clone()),
            Some(event.player.clone()),
            None,
            Some(event.is_team_0),
            event.player_position,
            None,
            Some(event.confidence),
        ));
    }

    events.sort_by(|left, right| {
        event_start_time(left)
            .total_cmp(&event_start_time(right))
            .then_with(|| left.meta.stream.cmp(&right.meta.stream))
            .then_with(|| left.meta.id.cmp(&right.meta.id))
    });
    events
}

pub(crate) fn boxed_default() -> Box<dyn AnalysisNodeDyn> {
    Box::new(StatsTimelineEventsNode::new())
}
