use super::match_stats_delta_helpers::optional_delta;
use super::*;

pub(super) fn record_player_context_core_delta(
    delta: &mut PlayerScoringContextStats,
    current: &PlayerScoringContextStats,
    previous: &PlayerScoringContextStats,
) {
    delta.goals_conceded_while_last_defender = current
        .goals_conceded_while_last_defender
        .saturating_sub(previous.goals_conceded_while_last_defender);
    delta.goals_for_while_most_back = current
        .goals_for_while_most_back
        .saturating_sub(previous.goals_for_while_most_back);
    delta.goals_against_while_most_back = current
        .goals_against_while_most_back
        .saturating_sub(previous.goals_against_while_most_back);
    delta.goal_against_boost_sample_count = current
        .goal_against_boost_sample_count
        .saturating_sub(previous.goal_against_boost_sample_count);
    delta.cumulative_boost_on_goals_against =
        current.cumulative_boost_on_goals_against - previous.cumulative_boost_on_goals_against;
    delta.last_boost_on_goal_against = optional_delta(
        current.last_boost_on_goal_against,
        previous.last_boost_on_goal_against,
    );
}

pub(super) fn record_player_context_boost_leadup_delta(
    delta: &mut PlayerScoringContextStats,
    current: &PlayerScoringContextStats,
    previous: &PlayerScoringContextStats,
) {
    delta.goal_against_boost_leadup_sample_count = current
        .goal_against_boost_leadup_sample_count
        .saturating_sub(previous.goal_against_boost_leadup_sample_count);
    delta.cumulative_average_boost_in_goal_against_leadup = current
        .cumulative_average_boost_in_goal_against_leadup
        - previous.cumulative_average_boost_in_goal_against_leadup;
    delta.cumulative_min_boost_in_goal_against_leadup = current
        .cumulative_min_boost_in_goal_against_leadup
        - previous.cumulative_min_boost_in_goal_against_leadup;
    delta.last_average_boost_in_goal_against_leadup = optional_delta(
        current.last_average_boost_in_goal_against_leadup,
        previous.last_average_boost_in_goal_against_leadup,
    );
    delta.last_min_boost_in_goal_against_leadup = optional_delta(
        current.last_min_boost_in_goal_against_leadup,
        previous.last_min_boost_in_goal_against_leadup,
    );
}

pub(super) fn record_player_context_position_delta(
    delta: &mut PlayerScoringContextStats,
    current: &PlayerScoringContextStats,
    previous: &PlayerScoringContextStats,
) {
    delta.goal_against_position_sample_count = current
        .goal_against_position_sample_count
        .saturating_sub(previous.goal_against_position_sample_count);
    delta.cumulative_goal_against_position_x =
        current.cumulative_goal_against_position_x - previous.cumulative_goal_against_position_x;
    delta.cumulative_goal_against_position_y =
        current.cumulative_goal_against_position_y - previous.cumulative_goal_against_position_y;
    delta.cumulative_goal_against_position_z =
        current.cumulative_goal_against_position_z - previous.cumulative_goal_against_position_z;
    delta.last_goal_against_position = optional_delta(
        current.last_goal_against_position,
        previous.last_goal_against_position,
    );
}

pub(super) fn record_player_context_scoring_touch_delta(
    delta: &mut PlayerScoringContextStats,
    current: &PlayerScoringContextStats,
    previous: &PlayerScoringContextStats,
) {
    delta.scoring_goal_last_touch_position_sample_count = current
        .scoring_goal_last_touch_position_sample_count
        .saturating_sub(previous.scoring_goal_last_touch_position_sample_count);
    delta.cumulative_scoring_goal_last_touch_position_x = current
        .cumulative_scoring_goal_last_touch_position_x
        - previous.cumulative_scoring_goal_last_touch_position_x;
    delta.cumulative_scoring_goal_last_touch_position_y = current
        .cumulative_scoring_goal_last_touch_position_y
        - previous.cumulative_scoring_goal_last_touch_position_y;
    delta.cumulative_scoring_goal_last_touch_position_z = current
        .cumulative_scoring_goal_last_touch_position_z
        - previous.cumulative_scoring_goal_last_touch_position_z;
    delta.last_scoring_goal_last_touch_position = optional_delta(
        current.last_scoring_goal_last_touch_position,
        previous.last_scoring_goal_last_touch_position,
    );
}
