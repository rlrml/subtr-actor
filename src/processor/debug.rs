use super::*;

impl<'a> ReplayProcessor<'a> {
    /// Rewrites an attribute map to use object-name keys instead of object ids.
    pub fn map_attribute_keys(
        &self,
        hash_map: &HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
    ) -> HashMap<String, boxcars::Attribute> {
        hash_map
            .iter()
            .map(|(k, (v, _updated))| {
                self.object_id_to_name
                    .get(k)
                    .map(|name| (name.clone(), v.clone()))
                    .unwrap()
            })
            .collect()
    }

    /// Returns a formatted dump of the processor's main actor-link mappings.
    pub fn all_mappings_string(&self) -> String {
        let pairs = [
            ("player_to_car", &self.player_to_car),
            ("player_to_team", &self.player_to_team),
            ("car_to_player", &self.car_to_player),
            ("car_to_boost", &self.car_to_boost),
            ("car_to_jump", &self.car_to_jump),
            ("car_to_double_jump", &self.car_to_double_jump),
            ("car_to_dodge", &self.car_to_dodge),
        ];
        let mut strings: Vec<_> = pairs
            .iter()
            .map(|(map_name, map)| format!("{map_name:?}: {map:?}"))
            .collect();
        strings.push(format!("name_to_object_id: {:?}", &self.name_to_object_id));
        strings.join("\n")
    }

    /// Returns a formatted dump of a single actor's current attribute state.
    pub fn actor_state_string(&self, actor_id: &boxcars::ActorId) -> String {
        if let Ok(actor_state) = self.get_actor_state(actor_id) {
            format!("{:?}", self.map_attribute_keys(&actor_state.attributes))
        } else {
            String::from("error")
        }
    }

    /// Prints the named actor states for the provided actor ids.
    pub fn print_actors_by_id<'b>(&self, actor_ids: impl Iterator<Item = &'b boxcars::ActorId>) {
        actor_ids.for_each(|actor_id| {
            let state = self.get_actor_state(actor_id).unwrap();
            println!(
                "{:?}\n\n\n",
                self.object_id_to_name.get(&state.object_id).unwrap()
            );
            println!("{:?}", self.map_attribute_keys(&state.attributes))
        })
    }

    /// Logs all actors of a specific object type with their mapped attributes.
    pub fn print_actors_of_type(&self, actor_type: &'static str) {
        self.iter_actors_by_type(actor_type)
            .unwrap()
            .for_each(|(_actor_id, state)| {
                log::debug!("{:?}", self.map_attribute_keys(&state.attributes));
            });
    }

    /// Logs the set of actor object types currently present in the state model.
    pub fn print_actor_types(&self) {
        let types: Vec<_> = self
            .actor_state
            .actor_ids_by_type
            .keys()
            .filter_map(|id| self.object_id_to_name.get(id))
            .collect();
        log::debug!("{types:?}");
    }

    /// Logs every currently known actor with its mapped state.
    pub fn print_all_actors(&self) {
        self.actor_state
            .actor_states
            .iter()
            .for_each(|(actor_id, actor_state)| {
                log::debug!(
                    "{}: {:?}",
                    self.object_id_to_name
                        .get(&actor_state.object_id)
                        .unwrap_or(&String::from("unknown")),
                    self.actor_state_string(actor_id)
                )
            });
    }
}
