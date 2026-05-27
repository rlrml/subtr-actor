use super::*;

pub(crate) const DEMOLISH_APPEARANCE_FRAME_COUNT: usize = 30;

build_player_feature_adder!(
    PlayerDemolishedBy,
    |_,
     player_id: &PlayerId,
     processor: &dyn ProcessorView,
     _frame,
     frame_number,
     _current_time: f32| {
        let demolisher_index = processor
            .demolishes()
            .iter()
            .find(|demolish_info| {
                &demolish_info.victim == player_id
                    && frame_number - demolish_info.frame < DEMOLISH_APPEARANCE_FRAME_COUNT
            })
            .map(|demolish_info| {
                processor
                    .iter_player_ids_in_order()
                    .position(|player_id| player_id == &demolish_info.attacker)
                    .unwrap_or_else(|| processor.iter_player_ids_in_order().count())
            })
            .and_then(|v| i32::try_from(v).ok())
            .unwrap_or(-1);
        convert_all_floats!(demolisher_index as f32)
    },
    "player demolished by"
);
