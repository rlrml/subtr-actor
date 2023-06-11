use boxcars;
use boxcars_frames::*;

use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    let data = std::fs::read(&args[1]).unwrap();
    let parsing = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse();
    let replay = parsing.unwrap();

    println!("{:?}", replay.properties);

    let mut collector = NDArrayCollector::<f32>::from_strings(
        &["InterpolatedBallRigidBodyNoVelocities"],
        &[
            "InterpolatedPlayerRigidBodyNoVelocities",
            // "PlayerRigidBodyNoVelocities",
            "PlayerBoost",
            "PlayerAnyJump",
            // "PlayerDemolishedBy",
        ],
    )
    .unwrap();

    FrameRateDecorator::new_from_fps(30.0, &mut collector)
        .process_replay(&replay)
        .unwrap();

    let (meta, array) = collector.get_meta_and_ndarray().unwrap();

    let position_columns: Vec<_> = meta
        .headers_vec()
        .into_iter()
        .enumerate()
        .filter(|(_index, name)| name.contains("rotation"))
        .filter(|(_index, name)| name.contains("Player 4"))
        .collect();

    println!("{:?}", position_columns);

    return;

    let mut last: std::collections::HashMap<usize, (f32, usize)> = std::collections::HashMap::new();

    let mut same_value_frames = 0;

    for frame_index in 0..array.shape()[0] {
        let mut same_as_last = Vec::new();
        for (index, column_name) in position_columns.iter() {
            let (last_value, same_count) = last.get(&index).unwrap_or(&(0.0, 1));
            let this_value = array.get((frame_index, *index)).unwrap();
            if this_value == last_value {
                let new_count = same_count + 1;
                same_as_last.push((column_name, new_count));
                last.insert(*index, (*last_value, same_count + 1));
            } else {
                last.insert(*index, (*this_value, 1));
            }
        }
        if true {
            println!("{:?}", same_as_last);
            println!("{}", frame_index);
            for (index, column_name) in position_columns.iter() {
                println!(
                    "{}: {}",
                    column_name,
                    array.get((frame_index, *index)).unwrap()
                );
            }
            same_value_frames += 1;
            println!("");
        }
    }

    println!("");
    println!("Total same value frames: {}", same_value_frames);

    println!("Total frames {}", array.shape()[0]);

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

    // println!("{:?}", meta);
    println!("Array shape is {:?}", array.shape());
}
