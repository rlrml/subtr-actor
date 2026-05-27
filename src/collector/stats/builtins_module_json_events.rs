use super::*;

pub(super) fn events_module_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    macro_rules! event_json {
        ($calculator:ty) => {{
            let calculator = graph_state::<$calculator>(graph, module_name)?;
            serialize_to_json_value(&EventsExport {
                events: calculator.events(),
            })
            .map(Some)
        }};
    }

    match module_name {
        "aerial_goal" => event_json!(AerialGoalCalculator),
        "high_aerial_goal" => event_json!(HighAerialGoalCalculator),
        "long_distance_goal" => event_json!(LongDistanceGoalCalculator),
        "own_half_goal" => event_json!(OwnHalfGoalCalculator),
        "empty_net_goal" => event_json!(EmptyNetGoalCalculator),
        "counter_attack_goal" => event_json!(CounterAttackGoalCalculator),
        "flick_goal" => event_json!(FlickGoalCalculator),
        "double_tap_goal" => event_json!(DoubleTapGoalCalculator),
        "one_timer_goal" => event_json!(OneTimerGoalCalculator),
        "passing_goal" => event_json!(PassingGoalCalculator),
        "air_dribble_goal" => event_json!(AirDribbleGoalCalculator),
        "flip_reset_goal" => event_json!(FlipResetGoalCalculator),
        "half_volley_goal" => event_json!(HalfVolleyGoalCalculator),
        _ => Ok(None),
    }
}
