use boxcars;
use boxcars_frames::ReplayDataCollector;

fn main() {
    let data = include_bytes!("../../aeda154d-a79c-490c-8c7f-0b8e9e43479d.replay");
    let parsing = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse();

    ReplayDataCollector::process_replay(&parsing.unwrap()).unwrap();
}

// TODO: move ReplayDataBuilder to lib

// TODO: TAGame.RBActor_TA:bIgnoreSyncing
// TODO: handle car sleeping

// TODO: Handle team assignment
// TODO: handle headers

// TODO: TAGame.GameEvent_Soccar_TA
// TODO: test replays

// TODO: demos

// TODO: create nd array/python stuff

// TODO: sampling rate wrapper
// TODO: remove post-goal wrapper

// Later
// TODO: overtime, ball_has_benn_hit
// TODO: pad availability

// TODO: goal-scored feature
// TODO: who was last touch feature
// TODO: handle boost pickups
