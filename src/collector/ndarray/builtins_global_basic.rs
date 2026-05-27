use super::*;

build_global_feature_adder!(
    SecondsRemaining,
    |_, processor: &dyn ProcessorView, _frame, _index, _current_time| {
        convert_all_floats!(processor.get_seconds_remaining().unwrap_or(0) as f32)
    },
    "seconds remaining"
);

build_global_feature_adder!(
    CurrentTime,
    |_, _processor, _frame, _index, current_time: f32| { convert_all_floats!(current_time) },
    "current time"
);

build_global_feature_adder!(
    ReplicatedStateName,
    |_, processor: &dyn ProcessorView, _frame, _index, _current_time| {
        convert_all_floats!(processor.get_replicated_state_name().unwrap_or(0) as f32)
    },
    "game state"
);

build_global_feature_adder!(
    ReplicatedGameStateTimeRemaining,
    |_, processor: &dyn ProcessorView, _frame, _index, _current_time| {
        convert_all_floats!(processor
            .get_replicated_game_state_time_remaining()
            .unwrap_or(0) as f32)
    },
    "kickoff countdown"
);

build_global_feature_adder!(
    BallHasBeenHit,
    |_, processor: &dyn ProcessorView, _frame, _index, _current_time| {
        convert_all_floats!(if processor.get_ball_has_been_hit().unwrap_or(false) {
            1.0
        } else {
            0.0
        })
    },
    "ball has been hit"
);

build_global_feature_adder!(
    FrameTime,
    |_, _processor, frame: &boxcars::Frame, _index, _current_time| {
        convert_all_floats!(frame.time)
    },
    "frame time"
);
