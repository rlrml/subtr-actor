use super::builtins_snapshot_frame_values::player_stats_snapshot_value;
use super::*;

pub(super) fn player_snapshot_frame_json(
    module_name: &str,
    graph: &AnalysisGraph,
    _replay_meta: &ReplayMeta,
) -> SubtrActorResult<Option<Value>> {
    macro_rules! player_module {
        ($name:literal, $calculator:ty) => {{
            let calculator = graph_state::<$calculator>(graph, module_name)?;
            player_stats_snapshot_value(calculator.player_stats()).map(Some)
        }};
    }

    match module_name {
        "ceiling_shot" => player_module!("ceiling_shot", CeilingShotCalculator),
        "wall_aerial" => player_module!("wall_aerial", WallAerialCalculator),
        "wall_aerial_shot" => player_module!("wall_aerial_shot", WallAerialShotCalculator),
        "whiff" => player_module!("whiff", WhiffCalculator),
        "wavedash" => player_module!("wavedash", WavedashCalculator),
        "speed_flip" => player_module!("speed_flip", SpeedFlipCalculator),
        "half_flip" => player_module!("half_flip", HalfFlipCalculator),
        "flick" => player_module!("flick", FlickCalculator),
        "musty_flick" => player_module!("musty_flick", MustyFlickCalculator),
        "dodge_reset" => player_module!("dodge_reset", DodgeResetCalculator),
        "positioning" => player_module!("positioning", PositioningCalculator),
        _ => Ok(None),
    }
}
