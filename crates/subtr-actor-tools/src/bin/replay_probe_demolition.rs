use subtr_actor::PlayerFrame;

use super::replay::collect_replay_data;
use super::{median, vec_length};

pub(crate) fn print_demolition(path: &str) {
    let replay_data = collect_replay_data(path);
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
                    if valid_ratio_inputs(demo_speed, rigid_body_speed) {
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
                    if valid_ratio_inputs(demo_speed, rigid_body_speed) {
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

fn valid_ratio_inputs(demo_speed: f32, rigid_body_speed: f32) -> bool {
    demo_speed.is_finite()
        && rigid_body_speed.is_finite()
        && demo_speed > 0.0
        && rigid_body_speed > 0.0
}
