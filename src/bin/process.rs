use boxcars;
use boxcars_frames::*;

fn main() {
    let data = include_bytes!("../../aeda154d-a79c-490c-8c7f-0b8e9e43479d.replay");
    let parsing = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse();

    // ReplayDataCollector::process_replay(&parsing.unwrap()).unwrap();
    let array = NDArrayCollector::<f32>::default()
        .build_ndarray(&parsing.unwrap())
        .unwrap();

    for i in 0..array.shape()[1] {
        println!(
            "{}: {:?}",
            i,
            array
                .slice(::ndarray::s![.., i])
                .iter()
                .cloned()
                .map(float_ord::FloatOrd)
                .max()
        );
    }
}

// TODO: create nd array/python stuff
// DONE: move ReplayDataBuilder to lib

// DONE: handle car sleeping

// TODO: Handle team assignment
// TODO: handle headers

// TODO: TAGame.GameEvent_Soccar_TA
// TODO: test replays

// TODO: demos

// TODO: sampling rate wrapper
// TODO: remove post-goal wrapper (using ball rigid body non-existent)

// TODO: extract data from rigid body in replay_data

// Later
// TODO: overtime, ball_has_benn_hit
// TODO: pad availability

// TODO: goal-scored feature
// TODO: who was last touch feature
// TODO: handle boost pickups
