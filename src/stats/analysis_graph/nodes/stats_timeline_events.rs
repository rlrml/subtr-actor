use super::*;
use crate::stats::calculators::*;
use crate::*;

#[derive(Debug, Clone, Default)]
pub struct StatsTimelineEventsState {
    pub events: ReplayStatsTimelineEvents,
}

pub struct StatsTimelineEventsNode {
    state: StatsTimelineEventsState,
}

impl StatsTimelineEventsNode {
    pub fn new() -> Self {
        Self {
            state: StatsTimelineEventsState::default(),
        }
    }
}

impl_analysis_node! {
    node = StatsTimelineEventsNode,
    state = StatsTimelineEventsState,
    name = "stats_timeline_events",
    dependencies = [
        match_stats_dependency(),
        demo_dependency(),
        backboard_dependency(),
        ball_carry_dependency(),
        ceiling_shot_dependency(),
        dodge_reset_dependency(),
        double_tap_dependency(),
        one_timer_dependency(),
        pass_dependency(),
        fifty_fifty_dependency(),
        flick_dependency(),
        musty_flick_dependency(),
        aerial_goal_dependency(),
        high_aerial_goal_dependency(),
        long_distance_goal_dependency(),
        own_half_goal_dependency(),
        empty_net_goal_dependency(),
        flick_goal_dependency(),
        one_timer_goal_dependency(),
        air_dribble_goal_dependency(),
        flip_reset_goal_dependency(),
        rush_dependency(),
        speed_flip_dependency(),
        half_flip_dependency(),
        wavedash_dependency(),
        whiff_dependency(),
        boost_dependency(),
        bump_dependency(),
    ],
    inputs = {
        match_stats: MatchStatsCalculator,
        demo: DemoCalculator,
        backboard: BackboardCalculator,
        ball_carry: BallCarryCalculator,
        ceiling_shot: CeilingShotCalculator,
        dodge_reset: DodgeResetCalculator,
        double_tap: DoubleTapCalculator,
        one_timer: OneTimerCalculator,
        pass: PassCalculator,
        fifty_fifty: FiftyFiftyCalculator,
        flick: FlickCalculator,
        musty_flick: MustyFlickCalculator,
        aerial_goal: AerialGoalCalculator,
        high_aerial_goal: HighAerialGoalCalculator,
        long_distance_goal: LongDistanceGoalCalculator,
        own_half_goal: OwnHalfGoalCalculator,
        empty_net_goal: EmptyNetGoalCalculator,
        flick_goal: FlickGoalCalculator,
        one_timer_goal: OneTimerGoalCalculator,
        air_dribble_goal: AirDribbleGoalCalculator,
        flip_reset_goal: FlipResetGoalCalculator,
        rush: RushCalculator,
        speed_flip: SpeedFlipCalculator,
        half_flip: HalfFlipCalculator,
        wavedash: WavedashCalculator,
        whiff: WhiffCalculator,
        boost: BoostCalculator,
        bump: BumpCalculator,
    },
    evaluate = |node| {
        let mut timeline = match_stats.timeline().to_vec();
        timeline.extend(demo.timeline().to_vec());
        timeline.sort_by(|left, right| left.time.total_cmp(&right.time));
        let goal_tags = combined_goal_tag_events(&[
            aerial_goal.events(),
            high_aerial_goal.events(),
            long_distance_goal.events(),
            own_half_goal.events(),
            empty_net_goal.events(),
            flick_goal.events(),
            one_timer_goal.events(),
            air_dribble_goal.events(),
            flip_reset_goal.events(),
        ]);

        node.state.events = ReplayStatsTimelineEvents {
            timeline,
            mechanics: build_mechanic_events(
                ball_carry,
                ceiling_shot,
                dodge_reset,
                double_tap,
                flick,
                musty_flick,
                one_timer,
                pass,
                speed_flip,
                half_flip,
                wavedash,
            ),
            goal_context: match_stats.goal_context_events().to_vec(),
            backboard: backboard.events().to_vec(),
            ceiling_shot: ceiling_shot.events().to_vec(),
            double_tap: double_tap.events().to_vec(),
            one_timer: one_timer.events().to_vec(),
            pass: pass.events().to_vec(),
            fifty_fifty: fifty_fifty.events().to_vec(),
            goal_tags,
            rush: rush.events().to_vec(),
            speed_flip: speed_flip.events().to_vec(),
            half_flip: half_flip.events().to_vec(),
            wavedash: wavedash.events().to_vec(),
            whiff: whiff.events().to_vec(),
            boost_pickups: boost.pickup_comparison_events().to_vec(),
            bump: bump.events().to_vec(),
        };
        Ok(())
    },
    state_ref = |node| &node.state,
}

fn moment_mechanic_event(
    kind: &str,
    index: usize,
    frame: usize,
    time: f32,
    player_id: PlayerId,
    is_team_0: bool,
) -> MechanicEvent {
    MechanicEvent {
        id: format!("{kind}:{frame}:{index}"),
        kind: kind.to_owned(),
        player_id,
        is_team_0,
        timing: MechanicTiming::Moment { frame, time },
    }
}

#[allow(clippy::too_many_arguments)]
fn span_mechanic_event(
    kind: &str,
    index: usize,
    start_frame: usize,
    end_frame: usize,
    start_time: f32,
    end_time: f32,
    player_id: PlayerId,
    is_team_0: bool,
) -> MechanicEvent {
    MechanicEvent {
        id: format!("{kind}:{start_frame}:{end_frame}:{index}"),
        kind: kind.to_owned(),
        player_id,
        is_team_0,
        timing: MechanicTiming::Span {
            start_frame,
            end_frame,
            start_time,
            end_time,
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn build_mechanic_events(
    ball_carry: &BallCarryCalculator,
    ceiling_shot: &CeilingShotCalculator,
    dodge_reset: &DodgeResetCalculator,
    double_tap: &DoubleTapCalculator,
    flick: &FlickCalculator,
    musty_flick: &MustyFlickCalculator,
    one_timer: &OneTimerCalculator,
    pass: &PassCalculator,
    speed_flip: &SpeedFlipCalculator,
    half_flip: &HalfFlipCalculator,
    wavedash: &WavedashCalculator,
) -> Vec<MechanicEvent> {
    let mut events = Vec::new();

    for (index, event) in ball_carry.carry_events().iter().enumerate() {
        let kind = match event.kind {
            BallCarryKind::Carry => "ball_carry",
            BallCarryKind::AirDribble => "air_dribble",
        };
        events.push(span_mechanic_event(
            kind,
            index,
            event.start_frame,
            event.end_frame,
            event.start_time,
            event.end_time,
            event.player_id.clone(),
            event.is_team_0,
        ));
    }

    for (index, event) in ceiling_shot.events().iter().enumerate() {
        events.push(span_mechanic_event(
            "ceiling_shot",
            index,
            event.ceiling_contact_frame,
            event.frame,
            event.ceiling_contact_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }

    for (index, event) in dodge_reset.on_ball_events().iter().enumerate() {
        events.push(moment_mechanic_event(
            "flip_reset",
            index,
            event.frame,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }

    for (index, event) in double_tap.events().iter().enumerate() {
        events.push(span_mechanic_event(
            "double_tap",
            index,
            event.backboard_frame,
            event.frame,
            event.backboard_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }

    for (index, event) in flick.events().iter().enumerate() {
        events.push(span_mechanic_event(
            "flick",
            index,
            event.setup_start_frame,
            event.frame,
            event.setup_start_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }

    for (index, event) in musty_flick.events().iter().enumerate() {
        events.push(span_mechanic_event(
            "musty_flick",
            index,
            event.dodge_frame,
            event.frame,
            event.dodge_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }

    for (index, event) in one_timer.events().iter().enumerate() {
        events.push(span_mechanic_event(
            "one_timer",
            index,
            event.pass_start_frame,
            event.frame,
            event.pass_start_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }

    for (index, event) in pass.events().iter().enumerate() {
        events.push(span_mechanic_event(
            "pass",
            index,
            event.start_frame,
            event.frame,
            event.start_time,
            event.time,
            event.passer.clone(),
            event.is_team_0,
        ));
    }

    for (index, event) in speed_flip.events().iter().enumerate() {
        events.push(moment_mechanic_event(
            "speed_flip",
            index,
            event.frame,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }

    for (index, event) in half_flip.events().iter().enumerate() {
        events.push(moment_mechanic_event(
            "half_flip",
            index,
            event.frame,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }

    for (index, event) in wavedash.events().iter().enumerate() {
        events.push(span_mechanic_event(
            "wavedash",
            index,
            event.dodge_frame,
            event.frame,
            event.dodge_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }

    events.sort_by(|left, right| {
        let left_time = mechanic_event_start_time(left);
        let right_time = mechanic_event_start_time(right);
        left_time
            .total_cmp(&right_time)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });
    events
}

fn mechanic_event_start_time(event: &MechanicEvent) -> f32 {
    match event.timing {
        MechanicTiming::Moment { time, .. } => time,
        MechanicTiming::Span { start_time, .. } => start_time,
    }
}
