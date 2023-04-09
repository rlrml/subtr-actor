use boxcars;
use std::collections::HashMap;

static BALL_TYPES: [&str; 5] = [
    "Archetypes.Ball.Ball_Default",
    "Archetypes.Ball.Ball_Basketball",
    "Archetypes.Ball.Ball_Puck",
    "Archetypes.Ball.CubeBall",
    "Archetypes.Ball.Ball_Breakout",
];

static BOOST_TYPE: &str = "Archetypes.CarComponents.CarComponent_Boost";
static JUMP_TYPE: &str = "Archetypes.CarComponents.CarComponent_Jump";
static DOUBLE_JUMP_TYPE: &str = "Archetypes.CarComponents.CarComponent_DoubleJump";
static DODGE_TYPE: &str = "Archetypes.CarComponents.CarComponent_Dodge";
static CAR_TYPE: &str = "Archetypes.Car.Car_Default";
static PLAYER_REPLICATION_KEY: &str = "Engine.Pawn:PlayerReplicationInfo";
static PLAYER_TYPE: &str = "TAGame.Default__PRI_TA";
static GAME_TYPE: &str = "Archetypes.GameEvent.GameEvent_Soccar";

static BOOST_AMOUNT_KEY: &str = "TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount";
static LAST_BOOST_AMOUNT_KEY: &str = "TAGame.CarComponent_Boost_TA:ReplicatedBoostAmount.Last";
static COMPONENT_ACTIVE_KEY: &str = "TAGame.CarComponent_TA:ReplicatedActive";
static RIGID_BODY_STATE_KEY: &str = "TAGame.RBActor_TA:ReplicatedRBState";
static TEAM_KEY: &str = "Engine.PlayerReplicationInfo:Team";
static UNIQUE_ID_KEY: &str = "Engine.PlayerReplicationInfo:UniqueId";
static VEHICLE_KEY: &str = "TAGame.CarComponent_TA:Vehicle";
static SECONDS_REMAINING_KEY: &str = "TAGame.GameEvent_Soccar_TA:SecondsRemaining";
static IGNORE_SYNCING_KEY: &str = "TAGame.RBActor_TA:bIgnoreSyncing";

static EMPTY_ACTOR_IDS: [boxcars::ActorId; 0] = [];

static BOOST_USED_PER_SECOND: f32 = 80.0 / 0.93;

#[derive(PartialEq, Debug, Clone)]
pub struct ActorState {
    pub attributes: HashMap<boxcars::ObjectId, boxcars::Attribute>,
    pub derived_attributes: HashMap<String, boxcars::Attribute>,
    pub object_id: boxcars::ObjectId,
    pub name_id: Option<i32>,
}

impl ActorState {
    fn new(new_actor: &boxcars::NewActor) -> Self {
        Self {
            attributes: HashMap::new(),
            derived_attributes: HashMap::new(),
            object_id: new_actor.object_id,
            name_id: new_actor.name_id,
        }
    }

    fn update_attribute(
        &mut self,
        update: &boxcars::UpdatedAttribute,
    ) -> Option<boxcars::Attribute> {
        self.attributes
            .insert(update.object_id, update.attribute.clone())
    }
}

struct ActorStateModeler {
    actor_states: HashMap<boxcars::ActorId, ActorState>,
    actor_ids_by_type: HashMap<boxcars::ObjectId, Vec<boxcars::ActorId>>,
}

impl ActorStateModeler {
    fn new() -> Self {
        Self {
            actor_states: HashMap::new(),
            actor_ids_by_type: HashMap::new(),
        }
    }

    fn process_frame(&mut self, frame: &boxcars::Frame) -> Result<(), String> {
        if let Some(err) = frame
            .deleted_actors
            .iter()
            .map(|n| self.delete_actor(n))
            .find(|r| r.is_err())
        {
            return err.map(|_| ());
        }
        if let Some(err) = frame
            .new_actors
            .iter()
            .map(|n| self.new_actor(n))
            .find(|r| r.is_err())
        {
            return err;
        }
        if let Some(err) = frame
            .updated_actors
            .iter()
            .map(|u| self.update_attribute(u))
            .find(|r| r.is_err())
        {
            return err.map(|_| ());
        }
        Ok(())
    }

    fn new_actor(&mut self, new_actor: &boxcars::NewActor) -> Result<(), String> {
        if let Some(state) = self.actor_states.get(&new_actor.actor_id) {
            if state.object_id != new_actor.object_id {
                return Err(format!(
                    "Tried to make new actor {:?}, existing state {:?}",
                    new_actor, state
                ));
            }
        } else {
            self.actor_states
                .insert(new_actor.actor_id, ActorState::new(new_actor));
            self.actor_ids_by_type
                .entry(new_actor.object_id)
                .or_insert_with(|| Vec::new())
                .push(new_actor.actor_id)
        }
        Ok(())
    }

    fn update_attribute(
        &mut self,
        update: &boxcars::UpdatedAttribute,
    ) -> Result<Option<boxcars::Attribute>, String> {
        self.actor_states
            .get_mut(&update.actor_id)
            .map(|state| state.update_attribute(update))
            .ok_or(format!(
                "Unable to find actor associated with update {:?}",
                update
            ))
    }

    fn delete_actor(&mut self, actor_id: &boxcars::ActorId) -> Result<ActorState, String> {
        let state = self
            .actor_states
            .remove(actor_id)
            .ok_or(format!("Unabled to delete actor id {:?}", actor_id))?;

        self.actor_ids_by_type
            .entry(state.object_id)
            .or_insert_with(|| Vec::new())
            .retain(|x| x != actor_id);

        Ok(state)
    }
}

pub type PlayerId = boxcars::UniqueId;

macro_rules! attribute_match {
    ($value:expr, $type:path, $err:expr) => {
        if let $type(value) = $value {
            Ok(value)
        } else {
            Err($err)
        }
    };
}

macro_rules! get_attribute {
    ($self:ident, $map:expr, $prop:expr, $type:path) => {
        $self.get_attribute($map, $prop).and_then(|found| {
            attribute_match!(
                found,
                $type,
                format!("Value for {:?} not of the expected type, {:?}", $prop, $map)
            )
        })
    };
}

macro_rules! get_actor_attribute_matching {
    ($self:ident, $actor:expr, $prop:expr, $type:path) => {
        $self.get_actor_attribute($actor, $prop).and_then(|found| {
            attribute_match!(
                found,
                $type,
                format!(
                    "Actor {:?} value for {:?} not of the expected type",
                    $actor, $prop
                )
            )
        })
    };
}

macro_rules! get_derived_attribute {
    ($map:expr, $key:expr, $type:path) => {
        $map.get($key)
            .ok_or(format!("No value for key: {:?}", $key))
            .and_then(|found| {
                attribute_match!(
                    found,
                    $type,
                    format!("Value for {:?} not of the expected type, {:?}", $key, $map)
                )
            })
    };
}

fn get_actor_id(active_actor: &boxcars::ActiveActor) -> boxcars::ActorId {
    active_actor.actor
}

pub type ReplayProcessorFrameHandler = dyn FnMut(&ReplayProcessor, &boxcars::Frame);

pub struct ReplayProcessor<'a> {
    replay: &'a boxcars::Replay,
    actor_state: ActorStateModeler,
    object_id_to_name: HashMap<boxcars::ObjectId, String>,
    name_to_object_id: HashMap<String, boxcars::ObjectId>,
    ball_actor_id: Option<boxcars::ActorId>,
    player_to_actor_id: HashMap<PlayerId, boxcars::ActorId>,
    player_to_car: HashMap<boxcars::ActorId, boxcars::ActorId>,
    player_to_team: HashMap<boxcars::ActorId, boxcars::ActorId>,
    car_to_boost: HashMap<boxcars::ActorId, boxcars::ActorId>,
    car_to_jump: HashMap<boxcars::ActorId, boxcars::ActorId>,
    car_to_double_jump: HashMap<boxcars::ActorId, boxcars::ActorId>,
    car_to_dodge: HashMap<boxcars::ActorId, boxcars::ActorId>,
}

impl<'a> ReplayProcessor<'a> {
    pub fn new(replay: &'a boxcars::Replay) -> Self {
        let mut object_id_to_name = HashMap::new();
        let mut name_to_object_id = HashMap::new();
        for (id, name) in replay.objects.iter().enumerate() {
            let object_id = boxcars::ObjectId(id as i32);
            object_id_to_name.insert(object_id, name.clone());
            name_to_object_id.insert(name.clone(), object_id);
        }
        Self {
            actor_state: ActorStateModeler::new(),
            replay,
            object_id_to_name,
            name_to_object_id,
            ball_actor_id: None,
            player_to_car: HashMap::new(),
            player_to_team: HashMap::new(),
            player_to_actor_id: HashMap::new(),
            car_to_boost: HashMap::new(),
            car_to_jump: HashMap::new(),
            car_to_double_jump: HashMap::new(),
            car_to_dodge: HashMap::new(),
        }
    }

    pub fn process<H>(mut self, handler: &mut H) -> Result<(), String>
    where
        H: FnMut(&ReplayProcessor, &boxcars::Frame, usize) -> Result<(), String>,
    {
        for (index, frame) in self
            .replay
            .network_frames
            .as_ref()
            .unwrap()
            .frames
            .iter()
            .enumerate()
        {
            self.actor_state.process_frame(frame)?;
            self.update_mappings(frame)?;
            self.update_ball_id(frame)?;
            self.update_boost_amounts(frame)?;
            handler(&self, frame, index)?;
        }
        Ok(())
    }

    // Update functions

    fn update_mappings(&mut self, frame: &boxcars::Frame) -> Result<(), String> {
        for update in frame.updated_actors.iter() {
            macro_rules! maintain_link {
                ($map:expr, $actor_type:expr, $attr:expr, $get_key: expr, $type:path) => {{
                    if &update.object_id == self.get_object_id_for_key(&$attr)? {
                        if self
                            .get_actor_ids_by_type($actor_type)?
                            .iter()
                            .any(|id| id == &update.actor_id)
                        {
                            let value = get_actor_attribute_matching!(
                                self,
                                &update.actor_id,
                                $attr,
                                $type
                            )?;
                            $map.insert($get_key(value), update.actor_id);
                        }
                    }
                }};
            }
            macro_rules! maintain_actor_link {
                ($map:expr, $actor_type:expr, $attr:expr) => {
                    maintain_link!(
                        $map,
                        $actor_type,
                        $attr,
                        get_actor_id,
                        boxcars::Attribute::ActiveActor
                    )
                };
            }
            macro_rules! maintain_vehicle_key_link {
                ($map:expr, $actor_type:expr) => {
                    maintain_actor_link!($map, $actor_type, VEHICLE_KEY)
                };
            }
            maintain_link!(
                self.player_to_actor_id,
                PLAYER_TYPE,
                UNIQUE_ID_KEY,
                |unique_id: &Box<boxcars::UniqueId>| *unique_id.clone(),
                boxcars::Attribute::UniqueId
            );
            maintain_actor_link!(self.player_to_car, CAR_TYPE, PLAYER_REPLICATION_KEY);
            maintain_actor_link!(self.player_to_team, PLAYER_TYPE, TEAM_KEY);
            maintain_vehicle_key_link!(self.car_to_boost, BOOST_TYPE);
            maintain_vehicle_key_link!(self.car_to_dodge, DODGE_TYPE);
            maintain_vehicle_key_link!(self.car_to_jump, JUMP_TYPE);
            maintain_vehicle_key_link!(self.car_to_double_jump, DOUBLE_JUMP_TYPE);
        }

        for actor_id in frame.deleted_actors.iter() {
            self.player_to_car.remove(actor_id).map(|car_id| {
                println!("Player actor {:?} deleted, car id: {:?}.", actor_id, car_id)
            });
        }

        Ok(())
    }

    fn update_ball_id(&mut self, frame: &boxcars::Frame) -> Result<(), String> {
        // XXX: This assumes there is only ever one ball, which is safe (I think?)
        if let Some(actor_id) = self.ball_actor_id {
            if frame.deleted_actors.contains(&actor_id) {
                self.ball_actor_id = None;
            }
        } else {
            self.ball_actor_id = self.find_ball_actor();
            if self.ball_actor_id.is_some() {
                return self.update_ball_id(frame);
            }
        }
        Ok(())
    }

    fn update_boost_amounts(&mut self, frame: &boxcars::Frame) -> Result<(), String> {
        let updates: Vec<_> = self
            .iter_actors_by_type_err(BOOST_TYPE)?
            .map(|(actor_id, actor_state)| {
                let (actor_amount_value, last_value, _, derived_value, is_active) =
                    self.get_current_boost_values(actor_state);
                let mut current_value = if actor_amount_value == last_value {
                    // If we don't have an update in the actor, just continue using our derived value
                    derived_value
                } else {
                    // If we do have an update in the actor, use that value.
                    actor_amount_value.into()
                };
                if is_active {
                    current_value -= frame.delta * BOOST_USED_PER_SECOND;
                }
                (actor_id.clone(), current_value.max(0.0), actor_amount_value)
            })
            .collect();

        for (actor_id, current_value, new_last_value) in updates {
            let derived_attributes = &mut self
                .actor_state
                .actor_states
                .get_mut(&actor_id)
                .unwrap()
                .derived_attributes;

            derived_attributes.insert(
                LAST_BOOST_AMOUNT_KEY.to_string(),
                boxcars::Attribute::Byte(new_last_value),
            );
            derived_attributes.insert(
                BOOST_AMOUNT_KEY.to_string(),
                boxcars::Attribute::Float(current_value),
            );
        }
        Ok(())
    }

    fn get_current_boost_values(&self, actor_state: &ActorState) -> (u8, u8, u8, f32, bool) {
        let amount_value = get_attribute!(
            self,
            &actor_state.attributes,
            BOOST_AMOUNT_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
        .unwrap_or(0);
        let active_value = get_attribute!(
            self,
            &actor_state.attributes,
            COMPONENT_ACTIVE_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
        .unwrap_or(0);
        let is_active = active_value % 2 == 1;
        let derived_value = actor_state
            .derived_attributes
            .get(&BOOST_AMOUNT_KEY.to_string())
            .ok_or("No boost amount value.")
            .cloned()
            .and_then(|v| {
                attribute_match!(
                    v,
                    boxcars::Attribute::Float,
                    "Expected bool for derived value"
                )
            })
            .unwrap_or(0.0);
        let last_boost_amount = attribute_match!(
            actor_state
                .derived_attributes
                .get(&LAST_BOOST_AMOUNT_KEY.to_string())
                .cloned()
                .unwrap_or_else(|| boxcars::Attribute::Byte(amount_value)),
            boxcars::Attribute::Byte,
            "Expected byte value"
        )
        .unwrap_or(0);
        (
            amount_value,
            last_boost_amount,
            active_value,
            derived_value,
            is_active,
        )
    }

    // Actor functions

    fn get_object_id_for_key(&self, name: &str) -> Result<&boxcars::ObjectId, String> {
        self.name_to_object_id
            .get(name)
            .ok_or(format!("Could not get object id for name {:?}", name))
    }

    fn get_actor_ids_by_type(&self, name: &str) -> Result<&[boxcars::ActorId], String> {
        self.get_object_id_for_key(name)
            .map(|object_id| self.get_actor_ids_by_object_id(object_id))
    }

    fn get_actor_ids_by_object_id(&self, object_id: &boxcars::ObjectId) -> &[boxcars::ActorId] {
        self.actor_state
            .actor_ids_by_type
            .get(object_id)
            .map(|v| &v[..])
            .unwrap_or_else(|| &EMPTY_ACTOR_IDS)
    }

    fn get_actor_state(&self, actor_id: &boxcars::ActorId) -> Result<&ActorState, String> {
        self.actor_state
            .actor_states
            .get(actor_id)
            .ok_or(format!("Actor id, {:?} not found", actor_id))
    }

    fn get_actor_attribute<'b>(
        &'b self,
        actor_id: &boxcars::ActorId,
        property: &'b str,
    ) -> Result<&'b boxcars::Attribute, String> {
        self.get_attribute(&self.get_actor_state(actor_id)?.attributes, property)
    }

    fn get_attribute<'b>(
        &'b self,
        map: &'b HashMap<boxcars::ObjectId, boxcars::Attribute>,
        property: &'b str,
    ) -> Result<&'b boxcars::Attribute, String> {
        let attribute_object_id = self
            .name_to_object_id
            .get(&property.to_string())
            .ok_or(format!("Could not find object_id for {:?}", property))?;
        map.get(attribute_object_id).ok_or(format!(
            "Could not find {:?} with object id {:?} on {:?}",
            property, attribute_object_id, map
        ))
    }

    fn find_ball_actor(&self) -> Option<boxcars::ActorId> {
        BALL_TYPES
            .iter()
            .filter_map(|ball_type| self.iter_actors_by_type(ball_type))
            .flat_map(|i| i)
            .map(|(actor_id, _)| actor_id.clone())
            .next()
    }

    pub fn get_metadata_actor_id(&self) -> Result<&boxcars::ActorId, String> {
        self.get_actor_ids_by_type(GAME_TYPE)
            .unwrap()
            .iter()
            .next()
            .ok_or("No game actor".to_string())
    }

    pub fn get_player_actor_id(&self, player_id: &PlayerId) -> Result<boxcars::ActorId, String> {
        self.player_to_actor_id
            .get(&player_id)
            .ok_or_else(|| format!("Could not find actor for player id {:?}", player_id))
            .cloned()
    }

    pub fn get_car_actor_id(&self, player_id: &PlayerId) -> Result<boxcars::ActorId, String> {
        self.player_to_car
            .get(&self.get_player_actor_id(player_id)?)
            .ok_or_else(|| format!("Car actor for player {:?} not found.", player_id))
            .cloned()
    }

    pub fn get_car_connected_actor_id(
        &self,
        player_id: &PlayerId,
        map: &HashMap<boxcars::ActorId, boxcars::ActorId>,
        name: &str,
    ) -> Result<boxcars::ActorId, String> {
        map.get(&self.get_car_actor_id(player_id)?)
            .ok_or_else(|| format!("{} actor for player {:?} not found", name, player_id))
            .cloned()
    }

    pub fn get_boost_actor_id(&self, player_id: &PlayerId) -> Result<boxcars::ActorId, String> {
        self.get_car_connected_actor_id(player_id, &self.car_to_boost, "Boost")
    }

    pub fn get_jump_actor_id(&self, player_id: &PlayerId) -> Result<boxcars::ActorId, String> {
        self.get_car_connected_actor_id(player_id, &self.car_to_jump, "Jump")
    }

    pub fn get_double_jump_actor_id(
        &self,
        player_id: &PlayerId,
    ) -> Result<boxcars::ActorId, String> {
        self.get_car_connected_actor_id(player_id, &self.car_to_double_jump, "Double Jump")
    }

    pub fn get_dodge_actor_id(&self, player_id: &PlayerId) -> Result<boxcars::ActorId, String> {
        self.get_car_connected_actor_id(player_id, &self.car_to_dodge, "Dodge")
    }

    // Actor iteration

    pub fn iter_player_ids(&self) -> impl Iterator<Item = &PlayerId> {
        self.player_to_actor_id.keys()
    }

    fn iter_actors_by_type_err(
        &self,
        name: &str,
    ) -> Result<impl Iterator<Item = (&boxcars::ActorId, &ActorState)>, String> {
        self.iter_actors_by_type(name)
            .ok_or_else(|| format!("Couldn't find object id for {}", name))
    }

    pub fn iter_actors_by_type(
        &self,
        name: &str,
    ) -> Option<impl Iterator<Item = (&boxcars::ActorId, &ActorState)>> {
        self.name_to_object_id
            .get(name)
            .map(|id| self.iter_actors_by_object_id(id))
    }

    pub fn iter_actors_by_object_id<'b>(
        &'b self,
        object_id: &'b boxcars::ObjectId,
    ) -> impl Iterator<Item = (&'b boxcars::ActorId, &'b ActorState)> + 'b {
        let actor_ids = self
            .actor_state
            .actor_ids_by_type
            .get(object_id)
            .map(|v| &v[..])
            .unwrap_or_else(|| &EMPTY_ACTOR_IDS);

        actor_ids
            .iter()
            .map(move |id| (id, self.actor_state.actor_states.get(id).unwrap()))
    }

    // Properties

    pub fn get_seconds_remaining(&self) -> Result<&i32, String> {
        get_actor_attribute_matching!(
            self,
            self.get_metadata_actor_id()?,
            SECONDS_REMAINING_KEY,
            boxcars::Attribute::Int
        )
    }

    pub fn get_ignore_ball_syncing(&self) -> Result<bool, String> {
        self.ball_actor_id
            .ok_or("Ball actor not known".to_string())
            .and_then(|actor_id| {
                get_actor_attribute_matching!(
                    self,
                    &actor_id,
                    IGNORE_SYNCING_KEY,
                    boxcars::Attribute::Boolean
                )
            })
            .cloned()
    }

    pub fn get_ball_rigid_body(&self) -> Result<&boxcars::RigidBody, String> {
        self.ball_actor_id
            .ok_or("Ball actor not known".to_string())
            .and_then(|actor_id| {
                get_actor_attribute_matching!(
                    self,
                    &actor_id,
                    RIGID_BODY_STATE_KEY,
                    boxcars::Attribute::RigidBody
                )
            })
    }

    pub fn get_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> Result<&boxcars::RigidBody, String> {
        self.get_car_actor_id(player_id).and_then(|actor_id| {
            get_actor_attribute_matching!(
                self,
                &actor_id,
                RIGID_BODY_STATE_KEY,
                boxcars::Attribute::RigidBody
            )
        })
    }

    pub fn get_player_boost_level(&self, player_id: &PlayerId) -> Result<&f32, String> {
        self.get_boost_actor_id(player_id).and_then(|actor_id| {
            let boost_state = self.get_actor_state(&actor_id)?;
            get_derived_attribute!(
                boost_state.derived_attributes,
                BOOST_AMOUNT_KEY,
                boxcars::Attribute::Float
            )
        })
    }

    pub fn get_component_active(&self, actor_id: &boxcars::ActorId) -> Result<&u8, String> {
        get_actor_attribute_matching!(
            self,
            &actor_id,
            COMPONENT_ACTIVE_KEY,
            boxcars::Attribute::Byte
        )
    }

    pub fn get_jump_active(&self, player_id: &PlayerId) -> Result<&u8, String> {
        self.get_jump_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    pub fn get_double_jump_active(&self, player_id: &PlayerId) -> Result<&u8, String> {
        self.get_double_jump_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    pub fn get_boost_active(&self, player_id: &PlayerId) -> Result<&u8, String> {
        self.get_boost_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    pub fn get_dodge_active(&self, player_id: &PlayerId) -> Result<&u8, String> {
        self.get_dodge_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    // Debugging

    pub fn map_attribute_keys(
        &self,
        hash_map: &HashMap<boxcars::ObjectId, boxcars::Attribute>,
    ) -> Result<HashMap<String, boxcars::Attribute>, ()> {
        hash_map
            .iter()
            .map(|(k, v)| {
                self.object_id_to_name
                    .get(k)
                    .map(|name| (name.clone(), v.clone()))
                    .ok_or(())
            })
            .collect()
    }
    pub fn actor_state_string(&self, actor_id: &boxcars::ActorId) -> String {
        format!(
            "{:?}",
            self.get_actor_state(actor_id)
                .map(|s| self.map_attribute_keys(&s.attributes))
        )
    }

    pub fn print_actors_of_type(&self, actor_type: &str) {
        self.iter_actors_by_type(actor_type)
            .unwrap()
            .for_each(|(_actor_id, state)| {
                println!("{:?}", self.map_attribute_keys(&state.attributes));
            });
    }

    pub fn print_actor_types(&self) {
        let types: Vec<_> = self
            .actor_state
            .actor_ids_by_type
            .keys()
            .filter_map(|id| self.object_id_to_name.get(id))
            .collect();
        println!("{:?}", types);
    }
}
