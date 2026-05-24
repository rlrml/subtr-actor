# Statistic Confidence

`subtr-actor` exports a mix of replay-reported values, frame-derived
measurements, and higher-level gameplay heuristics. This document tracks the
current confidence level for the exported statistic identifiers.

The identifier format below is `<domain>.<name>`, matching the exported
`StatDescriptor` fields. Labeled variants use the same identifier plus labels.
This is not a formal accuracy benchmark; confidence is an implementation-status
assessment based on source semantics, tests, manual replay inspection, and how
much inference the stat requires.

## Confidence Levels

- **High**: direct replay data or simple arithmetic over reliable frame state.
  These stats are generally suitable for user-facing summaries.
- **Medium**: derived from reliable frame state, but dependent on thresholds,
  time windows, inferred ownership, or game-context classification. These stats
  are useful analytical signals, but should be validated against your replay set
  before use as hard labels.
- **Experimental**: specialized heuristic detectors. These are useful for
  discovery, review, and feature exploration, but should not be treated as a
  production-grade source of truth without additional validation.

## High Confidence

| Identifiers | Basis and caveats |
| --- | --- |
| `core.score`, `core.goals`, `core.assists`, `core.saves`, `core.shots` | Replay-reported scoreboard values. Reliability mainly depends on successful replay parsing and player/team mapping. |
| `core.shooting_percentage` | Simple derived value over `core.goals` and `core.shots`. |
| `demo.inflicted`, `demo.taken` | Backed by explicit replay demolition events. |
| `boost.avg_amount`, `boost.time_zero_boost`, `boost.time_full_boost`, `boost.time_boost_0_25`, `boost.time_boost_25_50`, `boost.time_boost_50_75`, `boost.time_boost_75_100`, `boost.percent_zero_boost`, `boost.percent_full_boost`, `boost.percent_boost_0_25`, `boost.percent_boost_25_50`, `boost.percent_boost_50_75`, `boost.percent_boost_75_100` | Computed from observed per-player boost values across processed frames. Accuracy still depends on frame reconstruction and replay sampling. |
| `movement.tracked_time`, `movement.total_distance`, `movement.avg_speed`, `movement.avg_speed_percentage`, `movement.time_ground`, `movement.time_low_air`, `movement.time_high_air`, `movement.time_slow_speed`, `movement.time_boost_speed`, `movement.time_supersonic_speed`, `movement.percent_ground`, `movement.percent_low_air`, `movement.percent_high_air`, `movement.percent_slow_speed`, `movement.percent_boost_speed`, `movement.percent_supersonic_speed` | Arithmetic over reconstructed player rigid-body state and simple speed/height thresholds. Non-standard modes or malformed actor state can still affect values. |

## Medium Confidence

| Identifiers | Basis and caveats |
| --- | --- |
| `core.goals_conceded_while_last_defender`, `core.goals_for_while_most_back`, `core.goals_against_while_most_back`, `core.goal_against_boost_sample_count`, `core.average_boost_on_goals_against`, `core.goal_against_boost_leadup_sample_count`, `core.average_boost_in_goal_against_leadup`, `core.average_min_boost_in_goal_against_leadup`, `core.goal_against_position_sample_count`, `core.average_goal_against_position_x`, `core.average_goal_against_position_y`, `core.average_goal_against_position_z`, `core.scoring_goal_last_touch_position_sample_count`, `core.average_scoring_goal_last_touch_position_x`, `core.average_scoring_goal_last_touch_position_y`, `core.average_scoring_goal_last_touch_position_z`, `core.average_goal_time_after_kickoff`, `core.median_goal_time_after_kickoff`, `core.goal_ball_air_time_sample_count`, `core.average_goal_ball_air_time`, `core.median_goal_ball_air_time`, `core.kickoff_goal_count`, `core.short_goal_count`, `core.medium_goal_count`, `core.long_goal_count`, `core.counter_attack_goal_count`, `core.sustained_pressure_goal_count`, `core.other_buildup_goal_count` | Goal-context metrics reconstructed from frame state around scoring events. They depend on last-touch, position, boost, kickoff, ball-ground contact, and buildup classifiers. |
| `boost.bpm`, `boost.amount_collected`, `boost.amount_collected_inactive`, `boost.count_collected_inactive_big`, `boost.count_collected_inactive_small`, `boost.amount_stolen`, `boost.amount_collected_big`, `boost.amount_stolen_big`, `boost.amount_collected_small`, `boost.amount_stolen_small`, `boost.amount_respawned`, `boost.count_collected_big`, `boost.count_stolen_big`, `boost.count_collected_small`, `boost.count_stolen_small`, `boost.amount_overfill`, `boost.amount_overfill_stolen`, `boost.amount_used`, `boost.amount_used_while_grounded`, `boost.amount_used_while_airborne`, `boost.amount_used_while_supersonic` | Uses boost deltas, resolved pad state, ownership, location, live-play state, vertical state, and supersonic classification. More inferential than boost amount over time. |
| `touch.touch_count`, `touch.control_touch_count`, `touch.medium_hit_count`, `touch.hard_hit_count`, `touch.aerial_touch_count`, `touch.high_aerial_touch_count`, `touch.is_last_touch`, `touch.last_touch_time`, `touch.last_touch_frame`, `touch.time_since_last_touch`, `touch.frames_since_last_touch`, `touch.last_ball_speed_change`, `touch.average_ball_speed_change`, `touch.max_ball_speed_change`, `touch.total_ball_travel_distance`, `touch.total_ball_advance_distance`, `touch.total_ball_retreat_distance` | Touches are grounded in replay events, but labels and ball-effect metrics depend on nearby frame state and thresholds. |
| `possession.time`, `possession.team_zero_time`, `possession.team_one_time`, `possession.neutral_time`, `possession.team_zero_pct`, `possession.team_one_pct`, `possession.neutral_pct` | Inferred from touch sequences and live-play filtering. Useful as an analytical possession signal, not an official game stat. |
| `pressure.time`, `pressure.team_zero_side_time`, `pressure.team_one_side_time`, `pressure.neutral_time`, `pressure.team_zero_side_pct`, `pressure.team_one_side_pct`, `pressure.neutral_pct` | Derived from possession, ball/field position, and accumulated time. Treat as a tactical heuristic. |
| `positioning.active_game_time`, `positioning.avg_distance_to_ball`, `positioning.avg_distance_to_ball_possession`, `positioning.avg_distance_to_ball_no_possession`, `positioning.avg_distance_to_mates`, `positioning.time_defensive_third`, `positioning.time_neutral_third`, `positioning.time_offensive_third`, `positioning.time_defensive_half`, `positioning.time_offensive_half`, `positioning.time_behind_ball`, `positioning.time_level_with_ball`, `positioning.time_in_front_of_ball`, `positioning.time_most_back`, `positioning.time_mid_role`, `positioning.time_most_forward`, `positioning.time_other_role`, `positioning.time_no_teammates`, `positioning.time_closest_to_ball`, `positioning.time_farthest_from_ball`, `positioning.time_demolished`, `positioning.percent_defensive_third`, `positioning.percent_neutral_third`, `positioning.percent_offensive_third`, `positioning.percent_defensive_half`, `positioning.percent_offensive_half`, `positioning.percent_behind_ball`, `positioning.percent_level_with_ball`, `positioning.percent_in_front_of_ball`, `positioning.percent_most_back`, `positioning.percent_mid_role`, `positioning.percent_most_forward`, `positioning.percent_other_role`, `positioning.percent_closest_to_ball`, `positioning.percent_farthest_from_ball`, `positioning.times_caught_ahead_of_play_on_conceded_goals` | Thresholded interpretation of player locations, team orientation, possession state, and role ordering. Sensitive to playlist shape and role-threshold choices. |
| `pass.completed_pass_count`, `pass.received_pass_count`, `pass.average_pass_distance`, `pass.average_pass_advance`, `pass.longest_pass_distance`, `one_timer.count`, `one_timer.average_pass_distance`, `one_timer.average_ball_speed`, `one_timer.fastest_ball_speed` | Depends on touch ordering, team ownership, ball movement, and timing windows. Spot-check before using as supervised labels. |
| `fifty_fifty.count`, `fifty_fifty.wins`, `fifty_fifty.losses`, `fifty_fifty.neutral_outcomes`, `fifty_fifty.possession_after_count`, `fifty_fifty.win_pct`, `fifty_fifty.team_zero_wins`, `fifty_fifty.team_one_wins`, `fifty_fifty.neutral_possession_after_count`, `fifty_fifty.team_zero_possession_after_count`, `fifty_fifty.team_one_possession_after_count`, `fifty_fifty.team_zero_win_pct`, `fifty_fifty.team_one_win_pct`, `fifty_fifty.kickoff_count`, `fifty_fifty.kickoff_wins`, `fifty_fifty.kickoff_losses`, `fifty_fifty.kickoff_neutral_outcomes`, `fifty_fifty.kickoff_possession_after_count`, `fifty_fifty.kickoff_win_pct`, `fifty_fifty.kickoff_team_zero_wins`, `fifty_fifty.kickoff_team_one_wins`, `fifty_fifty.kickoff_team_zero_win_pct`, `fifty_fifty.kickoff_team_one_win_pct` | Uses contested-touch and possession-transition context. Counts can be sensitive to timing windows and touch attribution. |
| `whiff.whiff_count`, `whiff.grounded_whiff_count`, `whiff.aerial_whiff_count`, `whiff.dodge_whiff_count`, `whiff.is_last_whiff`, `whiff.last_whiff_time`, `whiff.last_whiff_frame`, `whiff.time_since_last_whiff`, `whiff.frames_since_last_whiff`, `whiff.last_closest_approach_distance`, `whiff.best_closest_approach_distance`, `whiff.average_closest_approach_distance` | Inferred from approach, proximity, speed, dodge/aerial state, and lack of touch. False positives are possible in congested or fake-challenge plays. |
| `rush.team_zero_count`, `rush.team_one_count`, `rush.team_zero_two_v_one_count`, `rush.team_zero_two_v_two_count`, `rush.team_zero_two_v_three_count`, `rush.team_zero_three_v_one_count`, `rush.team_zero_three_v_two_count`, `rush.team_zero_three_v_three_count`, `rush.team_one_two_v_one_count`, `rush.team_one_two_v_two_count`, `rush.team_one_two_v_three_count`, `rush.team_one_three_v_one_count`, `rush.team_one_three_v_two_count`, `rush.team_one_three_v_three_count` | Rush classifications depend on possession changes, advancing play, timing, and available-player context. |
| `backboard.count`, `double_tap.count`, `ball_carry.count`, `ball_carry.total_time`, `ball_carry.avg_time`, `ball_carry.longest_time`, `ball_carry.total_path_distance`, `ball_carry.avg_path_distance`, `ball_carry.total_straight_line_distance`, `ball_carry.avg_straight_line_distance`, `ball_carry.furthest_straight_line_distance`, `ball_carry.avg_horizontal_gap`, `ball_carry.avg_vertical_gap`, `ball_carry.avg_speed`, `ball_carry.fastest_avg_speed`, `air_dribble.count`, `air_dribble.total_time`, `air_dribble.avg_time`, `air_dribble.longest_time`, `air_dribble.total_path_distance`, `air_dribble.avg_path_distance`, `air_dribble.total_straight_line_distance`, `air_dribble.avg_straight_line_distance`, `air_dribble.furthest_straight_line_distance`, `air_dribble.avg_horizontal_gap`, `air_dribble.avg_vertical_gap`, `air_dribble.avg_speed`, `air_dribble.fastest_avg_speed` | Higher-level event interpretations built from touch sequences and ball/player frame state. Plausible for review workflows, but representative replay clips should be checked. |
| `powerslide.count_powerslide`, `powerslide.time_powerslide`, `powerslide.avg_powerslide_duration` | Based on player input/state across frames. Thresholding and sampling can affect short slides. |

## Experimental

| Identifiers | Basis and caveats |
| --- | --- |
| `flick.count`, `flick.high_confidence_count`, `flick.is_last_flick`, `flick.last_flick_time`, `flick.last_flick_frame`, `flick.time_since_last_flick`, `flick.frames_since_last_flick`, `flick.last_confidence`, `flick.average_confidence`, `flick.best_confidence`, `flick.average_setup_duration`, `flick.average_ball_speed_change` | Specialized flick detector using setup, timing, ball-speed change, and car/ball state heuristics. |
| `musty_flick.count`, `musty_flick.aerial_count`, `musty_flick.high_confidence_count`, `musty_flick.is_last_musty`, `musty_flick.last_musty_time`, `musty_flick.last_musty_frame`, `musty_flick.time_since_last_musty`, `musty_flick.frames_since_last_musty`, `musty_flick.last_confidence`, `musty_flick.average_confidence`, `musty_flick.best_confidence` | Specialized musty-flick detector; should be treated as a review signal rather than ground truth. |
| `ceiling_shot.count`, `ceiling_shot.high_confidence_count`, `ceiling_shot.is_last_ceiling_shot`, `ceiling_shot.last_ceiling_shot_time`, `ceiling_shot.last_ceiling_shot_frame`, `ceiling_shot.time_since_last_ceiling_shot`, `ceiling_shot.frames_since_last_ceiling_shot`, `ceiling_shot.last_confidence`, `ceiling_shot.average_confidence`, `ceiling_shot.best_confidence` | Specialized detector for ceiling-shot-like sequences. Sensitive to car-surface state, aerial continuation, and touch timing. |
| `speed_flip.count`, `speed_flip.high_confidence_count`, `speed_flip.is_last_speed_flip`, `speed_flip.last_speed_flip_time`, `speed_flip.last_speed_flip_frame`, `speed_flip.time_since_last_speed_flip`, `speed_flip.frames_since_last_speed_flip`, `speed_flip.last_quality`, `speed_flip.average_quality`, `speed_flip.best_quality` | Specialized kickoff/movement mechanic detector using thresholded orientation and cancel signals. |
| `wavedash.count`, `wavedash.high_confidence_count`, `wavedash.is_last_wavedash`, `wavedash.last_wavedash_time`, `wavedash.last_wavedash_frame`, `wavedash.time_since_last_wavedash`, `wavedash.frames_since_last_wavedash`, `wavedash.last_quality`, `wavedash.average_quality`, `wavedash.best_quality` | Specialized landing/jump mechanic detector. Short timing windows make it sensitive to sample rate and state reconstruction. |
| `half_flip.count`, `half_flip.high_confidence_count`, `half_flip.is_last_half_flip`, `half_flip.last_half_flip_time`, `half_flip.last_half_flip_frame`, `half_flip.time_since_last_half_flip`, `half_flip.frames_since_last_half_flip`, `half_flip.last_quality`, `half_flip.average_quality`, `half_flip.best_quality` | Specialized recovery mechanic detector using orientation, jump, and movement signals. |
| `dodge_reset.count`, `dodge_reset.on_ball_count` | Infers dodge-reset mechanics from car/ball state. Needs more replay-backed validation before production use. |
| `bump.inflicted`, `bump.taken`, `bump.team_inflicted`, `bump.team_taken` | Infers non-demo player collisions from movement and proximity signals. Especially sensitive to sampling, crowding, and ambiguous contact. |

## Interpreting Detector Scores

Fields named `last_confidence`, `average_confidence`, `best_confidence`,
`last_quality`, `average_quality`, and `best_quality` are internal detector
scores. They are not calibrated probabilities. Read them as "how strongly the
observed frames matched this detector's heuristic."

For downstream models, prefer using these scores as soft features or review
signals. Avoid treating them as ground-truth labels without manual or benchmark
validation.

## Updating This Document

When adding or changing an exported stat, update this document with the
`<domain>.<name>` identifier and the most conservative confidence level that
fits the evidence.

Promote a stat only when there is evidence that its semantics hold across replay
versions, playlists, skill levels, and unusual game states. Useful evidence
includes focused unit tests, replay fixtures with known expected events, manual
review in the stats player, comparison with official Rocket League stats or a
trusted external source, and documented behavior for kickoffs, goals, overtime,
demos, disconnects, and non-standard modes.
