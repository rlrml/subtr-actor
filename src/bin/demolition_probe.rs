use subtr_actor::{PlayerFrame, ReplayDataCollector};

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/rlcs.replay".to_string());
    let data =
        std::fs::read(&path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    let replay = boxcars::ParserBuilder::new(&data[..])
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .unwrap_or_else(|error| panic!("failed to parse {path}: {error}"));
    let replay_data = ReplayDataCollector::new()
        .get_replay_data(&replay)
        .unwrap_or_else(|error| panic!("failed to collect replay data for {path}: {error:?}"));

    let mut attacker_ratios = Vec::new();
    let mut victim_ratios = Vec::new();

    for demolish in &replay_data.demolish_infos {
        if let Some(player_data) = replay_data
            .frame_data
            .players
            .iter()
            .find(|(player_id, _)| player_id == &demolish.attacker)
            .map(|(_, player_data)| player_data)
        {
            if let Some(PlayerFrame::Data { rigid_body, .. }) =
                player_data.frames().get(demolish.frame)
            {
                if let Some(linear_velocity) = rigid_body.linear_velocity {
                    let demo_speed = vec_length(demolish.attacker_velocity);
                    let rigid_body_speed = vec_length(linear_velocity);
                    if demo_speed.is_finite()
                        && rigid_body_speed.is_finite()
                        && demo_speed > 0.0
                        && rigid_body_speed > 0.0
                    {
                        attacker_ratios.push(demo_speed / rigid_body_speed);
                    }
                }
            }
        }

        if let Some(player_data) = replay_data
            .frame_data
            .players
            .iter()
            .find(|(player_id, _)| player_id == &demolish.victim)
            .map(|(_, player_data)| player_data)
        {
            if let Some(PlayerFrame::Data { rigid_body, .. }) =
                player_data.frames().get(demolish.frame)
            {
                if let Some(linear_velocity) = rigid_body.linear_velocity {
                    let demo_speed = vec_length(demolish.victim_velocity);
                    let rigid_body_speed = vec_length(linear_velocity);
                    if demo_speed.is_finite()
                        && rigid_body_speed.is_finite()
                        && demo_speed > 0.0
                        && rigid_body_speed > 0.0
                    {
                        victim_ratios.push(demo_speed / rigid_body_speed);
                    }
                }
            }
        }
    }

    println!(
        "replay={path} demolishes={} attacker_ratio_median={:?} victim_ratio_median={:?}",
        replay_data.demolish_infos.len(),
        median(&mut attacker_ratios),
        median(&mut victim_ratios)
    );
}

fn vec_length(vector: boxcars::Vector3f) -> f32 {
    glam::Vec3::new(vector.x, vector.y, vector.z).length()
}

fn median(values: &mut [f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    Some(values[values.len() / 2])
}
