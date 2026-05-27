use crate::{PlayerId, ReplayProcessor, PLAYER_TYPE};
use std::collections::HashMap;

impl<'a> ReplayProcessor<'a> {
    /// Iterates over players in the stable team-zero, then team-one ordering.
    pub fn iter_player_ids_in_order(&self) -> impl Iterator<Item = &PlayerId> {
        self.team_zero.iter().chain(self.team_one.iter())
    }

    /// Counts currently in-game players per team from live actor state.
    pub fn current_in_game_team_player_counts(&self) -> [usize; 2] {
        let mut counts = [0, 0];
        let Ok(player_actor_ids) = self.get_actor_ids_by_type(PLAYER_TYPE) else {
            return counts;
        };
        let mut seen_players = std::collections::HashSet::new();

        for actor_id in player_actor_ids {
            let Ok(player_id) = self.get_player_id_from_actor_id(actor_id) else {
                continue;
            };
            if !seen_players.insert(player_id) {
                continue;
            }

            let Some(team_actor_id) = self.player_to_team.get(actor_id) else {
                continue;
            };
            let Ok(team_state) = self.get_actor_state(team_actor_id) else {
                continue;
            };
            let Some(team_name) = self.object_id_to_name.get(&team_state.object_id) else {
                continue;
            };

            match team_name.chars().last() {
                Some('0') => counts[0] += 1,
                Some('1') => counts[1] += 1,
                _ => {}
            }
        }

        counts
    }

    /// Returns the number of players in the stored replay ordering.
    pub fn player_count(&self) -> usize {
        self.iter_player_ids_in_order().count()
    }

    /// Returns a map from player ids to their resolved display names.
    pub fn get_player_names(&self) -> HashMap<PlayerId, String> {
        self.iter_player_ids_in_order()
            .filter_map(|player_id| {
                self.get_player_name(player_id)
                    .ok()
                    .map(|name| (player_id.clone(), name))
            })
            .collect()
    }
}
