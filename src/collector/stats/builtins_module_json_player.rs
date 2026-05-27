use super::*;

pub(super) fn player_module_json(
    module_name: &str,
    graph: &AnalysisGraph,
) -> SubtrActorResult<Option<Value>> {
    macro_rules! player_events_json {
        ($calculator:ty) => {{
            let calculator = graph_state::<$calculator>(graph, module_name)?;
            serialize_to_json_value(&PlayerStatsWithEventsExport {
                player_stats: player_stats_entries(calculator.player_stats()),
                events: calculator.events(),
            })
            .map(Some)
        }};
    }

    match module_name {
        "ceiling_shot" => player_events_json!(CeilingShotCalculator),
        "wall_aerial" => player_events_json!(WallAerialCalculator),
        "wall_aerial_shot" => player_events_json!(WallAerialShotCalculator),
        "whiff" => player_events_json!(WhiffCalculator),
        "wavedash" => player_events_json!(WavedashCalculator),
        "speed_flip" => player_events_json!(SpeedFlipCalculator),
        "half_flip" => player_events_json!(HalfFlipCalculator),
        "flick" => player_events_json!(FlickCalculator),
        "musty_flick" => player_events_json!(MustyFlickCalculator),
        "positioning" => player_events_json!(PositioningCalculator),
        "dodge_reset" => {
            let calculator = graph_state::<DodgeResetCalculator>(graph, module_name)?;
            serialize_to_json_value(&serde_json::json!({
                "player_stats": player_stats_entries(calculator.player_stats()),
                "events": calculator.events(),
                "on_ball_events": calculator.on_ball_events(),
            }))
            .map(Some)
        }
        _ => Ok(None),
    }
}
