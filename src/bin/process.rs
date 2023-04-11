use boxcars;
use boxcars_frames::*;

fn main() {
    let data = include_bytes!("../../aeda154d-a79c-490c-8c7f-0b8e9e43479d.replay");
    let parsing = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse();
    let replay = parsing.unwrap();

    // ReplayDataCollector::process_replay(&parsing.unwrap()).unwrap();
    let collector = NDArrayCollector::<f32>::with_jump_availabilities()
        .process_replay(&replay)
        .unwrap();

    let array = collector.get_ndarray().unwrap();

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
