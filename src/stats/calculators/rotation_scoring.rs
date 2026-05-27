use super::*;

pub(crate) fn scored_players(
    team_players: &[(&PlayerSample, glam::Vec3)],
    ball_position: glam::Vec3,
) -> Vec<(PlayerId, f32)> {
    let mut scored_players: Vec<_> = team_players
        .iter()
        .map(|(player, position)| {
            (
                player.player_id.clone(),
                first_man_score(*position, ball_position),
            )
        })
        .collect();
    scored_players
        .sort_by(|(_, left_score), (_, right_score)| left_score.partial_cmp(right_score).unwrap());
    scored_players
}

fn first_man_score(player_position: glam::Vec3, ball_position: glam::Vec3) -> f32 {
    player_position
        .truncate()
        .distance(ball_position.truncate())
}

pub(crate) fn raw_first_man(
    scored_players: &[(PlayerId, f32)],
    ambiguity_margin: f32,
) -> Option<&PlayerId> {
    let [(first_id, first_score), (_, second_score), ..] = scored_players else {
        return None;
    };

    if second_score - first_score <= ambiguity_margin {
        None
    } else {
        Some(first_id)
    }
}

pub(crate) fn role_assignments(
    stable_first_man: Option<&PlayerId>,
    scored_players: &[(PlayerId, f32)],
) -> HashMap<PlayerId, RoleState> {
    let mut assignments = HashMap::new();
    let Some(stable_first_man) = stable_first_man else {
        for (player_id, _) in scored_players {
            assignments.insert(player_id.clone(), RoleState::Ambiguous);
        }
        return assignments;
    };

    assignments.insert(stable_first_man.clone(), RoleState::FirstMan);
    insert_support_roles(scored_players, stable_first_man, &mut assignments);
    assignments
}

fn insert_support_roles(
    scored_players: &[(PlayerId, f32)],
    stable_first_man: &PlayerId,
    assignments: &mut HashMap<PlayerId, RoleState>,
) {
    let mut support_rank = 0;
    for (player_id, _) in scored_players {
        if player_id == stable_first_man {
            continue;
        }
        support_rank += 1;
        let role = match support_rank {
            1 => RoleState::SecondMan,
            2 => RoleState::ThirdMan,
            _ => RoleState::Ambiguous,
        };
        assignments.insert(player_id.clone(), role);
    }
}
