use boxcars;
use boxcars_frames::*;

fn main() {
    let data = include_bytes!("../../029103f9-4d58-4964-b47a-539b32f6fb33.replay");
    let parsing = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse();
    let replay = parsing.unwrap();

    // println!("{:?}", replay.properties);

    let collector = NDArrayCollector::<f32>::with_jump_availabilities()
        .process_replay(&replay)
        .unwrap();

    let (player_infos, columns, array) = collector.get_meta_and_ndarray().unwrap();

    for i in 0..array.shape()[1] {
        println!(
            "{}: {:?}",
            columns[i],
            array
                .slice(::ndarray::s![.., i])
                .iter()
                .cloned()
                .map(float_ord::FloatOrd)
                .max()
        );
    }

    println!("{:?}", player_infos);
}
