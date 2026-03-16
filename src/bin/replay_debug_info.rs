use std::env;

use subtr_actor::*;

fn main() {
    let args: Vec<_> = env::args().collect();
    for path in &args[1..] {
        let data = std::fs::read(path).unwrap();
        let replay = boxcars::ParserBuilder::new(&data[..])
            .always_check_crc()
            .must_parse_network_data()
            .parse()
            .unwrap();
        let mut collector = NDArrayCollector::<f32>::from_strings(
            &["InterpolatedBallRigidBodyNoVelocities"],
            &["InterpolatedPlayerRigidBodyNoVelocities"],
        )
        .unwrap();
        FrameRateDecorator::new_from_fps(10.0, &mut collector)
            .process_replay(&replay)
            .unwrap();
        let (meta, array) = collector.get_meta_and_ndarray().unwrap();
        let headers = meta.headers_vec();
        let mut max_abs_position = 0.0f32;
        for (index, header) in headers.iter().enumerate() {
            if !header.contains("position ") {
                continue;
            }
            let col_max = array
                .column(index)
                .iter()
                .copied()
                .map(f32::abs)
                .fold(0.0f32, f32::max);
            max_abs_position = max_abs_position.max(col_max);
        }
        println!(
            "{path}: major={:?} minor={:?} net={:?} max_abs_position={max_abs_position}",
            replay.major_version, replay.minor_version, replay.net_version
        );
    }
}
