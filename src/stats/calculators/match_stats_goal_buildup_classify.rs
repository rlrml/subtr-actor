use super::match_stats_goal_buildup::GoalBuildupZoneTimes;
use super::*;

pub(super) fn goal_buildup_zone_times(
    samples: &[&GoalBuildupSample],
    scoring_team_is_team_0: bool,
) -> GoalBuildupZoneTimes {
    let mut times = GoalBuildupZoneTimes::default();
    for entry in samples {
        let normalized_ball_y = normalized_goal_buildup_ball_y(entry, scoring_team_is_team_0);
        if normalized_ball_y < 0.0 {
            times.defensive_half += entry.dt;
        } else {
            times.offensive_half += entry.dt;
        }
        if normalized_ball_y < -FIELD_ZONE_BOUNDARY_Y {
            times.defensive_third += entry.dt;
        }
        if normalized_ball_y > FIELD_ZONE_BOUNDARY_Y {
            times.offensive_third += entry.dt;
        }
    }
    times
}

pub(super) fn current_goal_buildup_attack_time(
    samples: &[&GoalBuildupSample],
    scoring_team_is_team_0: bool,
) -> f32 {
    let mut current_attack_time = 0.0;
    for entry in samples.iter().rev() {
        if normalized_goal_buildup_ball_y(entry, scoring_team_is_team_0) > 0.0 {
            current_attack_time += entry.dt;
        } else {
            break;
        }
    }
    current_attack_time
}

pub(super) fn classify_goal_buildup_from_times(
    zone_times: GoalBuildupZoneTimes,
    current_attack_time: f32,
    opponent_shot_in_lookback: bool,
) -> GoalBuildupKind {
    let has_defensive_pressure_signal = zone_times.defensive_half
        >= COUNTER_ATTACK_MIN_DEFENSIVE_HALF_SECONDS
        || zone_times.defensive_third >= COUNTER_ATTACK_MIN_DEFENSIVE_THIRD_SECONDS
        || opponent_shot_in_lookback;

    if current_attack_time <= COUNTER_ATTACK_MAX_ATTACK_SECONDS && has_defensive_pressure_signal {
        GoalBuildupKind::CounterAttack
    } else if current_attack_time >= SUSTAINED_PRESSURE_MIN_ATTACK_SECONDS
        && zone_times.offensive_half >= SUSTAINED_PRESSURE_MIN_OFFENSIVE_HALF_SECONDS
        && zone_times.offensive_third >= SUSTAINED_PRESSURE_MIN_OFFENSIVE_THIRD_SECONDS
    {
        GoalBuildupKind::SustainedPressure
    } else {
        GoalBuildupKind::Other
    }
}

fn normalized_goal_buildup_ball_y(sample: &GoalBuildupSample, scoring_team_is_team_0: bool) -> f32 {
    if scoring_team_is_team_0 {
        sample.ball_y
    } else {
        -sample.ball_y
    }
}
