use subtr_actor::*;

use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(about = "Print feature min/max summaries for a replay-derived ndarray.")]
struct Args {
    /// Replay path to summarize.
    replay_path: PathBuf,
}

fn main() {
    let Args { replay_path } = Args::parse();
    let data = std::fs::read(&replay_path).unwrap();
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
