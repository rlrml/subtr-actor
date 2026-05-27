use super::*;

impl<'a> ReplayProcessor<'a> {
    pub fn new(replay: &'a boxcars::Replay) -> SubtrActorResult<Self> {
        let (object_id_to_name, name_to_object_id) = object_id_maps(replay);
        let spatial_normalization_factor =
            Self::rigid_body_location_normalization_factor_for_net_version(replay.net_version);
        let rigid_body_velocity_normalization_factor =
            Self::rigid_body_velocity_normalization_factor_for_net_version(replay.net_version);
        let uses_legacy_rigid_body_rotation =
            Self::uses_legacy_rigid_body_rotation_for_net_version(replay.net_version);
        let cached_object_ids = CachedObjectIds::from_name_map(&name_to_object_id);
        let mut processor = Self {
            actor_state: ActorStateModeler::new(),
            replay,
            spatial_normalization_factor,
            rigid_body_velocity_normalization_factor,
            uses_legacy_rigid_body_rotation,
            cached_object_ids,
            is_boost_pad_object: replay
                .objects
                .iter()
                .map(|name| name.contains("VehiclePickup_Boost_TA"))
                .collect(),
            object_id_to_name,
            name_to_object_id,
            team_zero: Vec::new(),
            team_one: Vec::new(),
            ball_actor_id: None,
            player_to_car: HashMap::new(),
            player_to_team: HashMap::new(),
            player_to_actor_id: HashMap::new(),
            car_to_player: HashMap::new(),
            car_to_boost: HashMap::new(),
            car_to_jump: HashMap::new(),
            car_to_double_jump: HashMap::new(),
            car_to_dodge: HashMap::new(),
            boost_pad_events: Vec::new(),
            current_frame_boost_pad_events: Vec::new(),
            boost_pad_pickup_sequence_times: HashMap::new(),
            touch_events: Vec::new(),
            current_frame_touch_events: Vec::new(),
            dodge_refreshed_events: Vec::new(),
            current_frame_dodge_refreshed_events: Vec::new(),
            dodge_refreshed_counters: HashMap::new(),
            goal_events: Vec::new(),
            current_frame_goal_events: Vec::new(),
            player_stat_events: Vec::new(),
            current_frame_player_stat_events: Vec::new(),
            player_stat_counters: HashMap::new(),
            demolishes: Vec::new(),
            known_demolishes: Vec::new(),
            demolish_format: None,
            kickoff_phase_active_last_frame: false,
        };
        processor
            .set_player_order_from_headers()
            .or_else(|_| processor.set_player_order_from_frames())?;
        Ok(processor)
    }
}

fn object_id_maps(
    replay: &boxcars::Replay,
) -> (
    HashMap<boxcars::ObjectId, String>,
    HashMap<String, boxcars::ObjectId>,
) {
    let mut object_id_to_name = HashMap::new();
    let mut name_to_object_id = HashMap::new();
    for (id, name) in replay.objects.iter().enumerate() {
        let object_id = boxcars::ObjectId(id as i32);
        object_id_to_name.insert(object_id, name.clone());
        name_to_object_id.insert(name.clone(), object_id);
    }
    (object_id_to_name, name_to_object_id)
}
