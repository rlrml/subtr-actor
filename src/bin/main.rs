use boxcars;
use subtr_actor::*;

use std::env;

build_global_feature_adder!(
    CurrentTime,
    |_, _processor, _frame, _index, current_time: f32| { convert_all_floats!(current_time) },
    "current time"
);

fn main() {
    let args: Vec<_> = env::args().collect();
    let data = std::fs::read(&args[1]).unwrap();
    let parsing = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse();
    let replay = parsing.unwrap();

    let mut collector = NDArrayCollector::<f32>::from_strings(
        &["InterpolatedBallRigidBodyNoVelocities"],
        &[
            "InterpolatedPlayerRigidBodyNoVelocities",
            "PlayerBoost",
            "PlayerAnyJump",
            "PlayerDemolishedBy",
        ],
    )
    .unwrap();

    FrameRateDecorator::new_from_fps(10.0, &mut collector)
        .process_replay(&replay)
        .unwrap();

    let (meta, array) = collector.get_meta_and_ndarray().unwrap();

    for i in 0..array.shape()[1] {
        println!(
            "{}: ({:?}) - ({:?})",
            meta.headers_vec()[i],
            array
                .slice(::ndarray::s![.., i])
                .iter()
                .cloned()
                .map(float_ord::FloatOrd)
                .min()
                .unwrap()
                .0,
            array
                .slice(::ndarray::s![.., i])
                .iter()
                .cloned()
                .map(float_ord::FloatOrd)
                .max()
                .unwrap()
                .0
        );
    }

    println!("Array shape is {:?}", array.shape());
}
