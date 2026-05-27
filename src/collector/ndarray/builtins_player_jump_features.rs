use super::*;

build_player_feature_adder!(
    PlayerBoost,
    |_, player_id: &PlayerId, processor: &dyn ProcessorView, _frame, _index, _current_time: f32| {
        convert_all_floats!(processor.get_player_boost_level(player_id).unwrap_or(0.0))
    },
    "boost level (raw replay units)"
);

pub(crate) fn u8_get_f32(v: u8) -> SubtrActorResult<f32> {
    Ok(v.into())
}

build_player_feature_adder!(
    PlayerJump,
    |_,
     player_id: &PlayerId,
     processor: &dyn ProcessorView,
     _frame,
     _frame_number,
     _current_time: f32| {
        convert_all_floats!(
            processor
                .get_dodge_active(player_id)
                .and_then(u8_get_f32)
                .unwrap_or(0.0),
            processor
                .get_jump_active(player_id)
                .and_then(u8_get_f32)
                .unwrap_or(0.0),
            processor
                .get_double_jump_active(player_id)
                .and_then(u8_get_f32)
                .unwrap_or(0.0),
        )
    },
    "dodge active",
    "jump active",
    "double jump active"
);

build_player_feature_adder!(
    PlayerAnyJump,
    |_,
     player_id: &PlayerId,
     processor: &dyn ProcessorView,
     _frame,
     _frame_number,
     _current_time: f32| {
        let dodge_is_active = processor.get_dodge_active(player_id).unwrap_or(0) % 2;
        let jump_is_active = processor.get_jump_active(player_id).unwrap_or(0) % 2;
        let double_jump_is_active = processor.get_double_jump_active(player_id).unwrap_or(0) % 2;
        let value: f32 = [dodge_is_active, jump_is_active, double_jump_is_active]
            .into_iter()
            .enumerate()
            .map(|(index, is_active)| (1 << index) * is_active)
            .sum::<u8>() as f32;
        convert_all_floats!(value)
    },
    "any_jump_active"
);

build_player_feature_adder!(
    PlayerDodgeRefreshed,
    |_,
     player_id: &PlayerId,
     processor: &dyn ProcessorView,
     _frame,
     _frame_number,
     _current_time: f32| {
        let dodge_refresh_count = processor
            .current_frame_dodge_refreshed_events()
            .iter()
            .filter(|event| &event.player == player_id)
            .count() as f32;
        convert_all_floats!(dodge_refresh_count)
    },
    "dodge refresh count"
);
