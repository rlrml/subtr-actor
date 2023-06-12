use crate::constants::*;
use crate::*;
use boxcars;
use serde::Serialize;
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct ActorState {
    pub attributes: HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
    pub derived_attributes: HashMap<String, (boxcars::Attribute, usize)>,
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
        frame_index: usize,
    ) -> Option<(boxcars::Attribute, usize)> {
        self.attributes
            .insert(update.object_id, (update.attribute.clone(), frame_index))
    }
}

pub struct ActorStateModeler {
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

    fn process_frame(&mut self, frame: &boxcars::Frame, frame_index: usize) -> BoxcarsResult<()> {
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
            .map(|u| self.update_attribute(u, frame_index))
            .find(|r| r.is_err())
        {
            return err.map(|_| ());
        }
        Ok(())
    }

    fn new_actor(&mut self, new_actor: &boxcars::NewActor) -> BoxcarsResult<()> {
        if let Some(state) = self.actor_states.get(&new_actor.actor_id) {
            if state.object_id != new_actor.object_id {
                return BoxcarsError::new_result(BoxcarsErrorVariant::ActorIdAlreadyExists {
                    actor_id: new_actor.actor_id.clone(),
                    object_id: new_actor.object_id.clone(),
                });
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
        frame_index: usize,
    ) -> BoxcarsResult<Option<(boxcars::Attribute, usize)>> {
        self.actor_states
            .get_mut(&update.actor_id)
            .map(|state| state.update_attribute(update, frame_index))
            .ok_or_else(|| {
                BoxcarsError::new(BoxcarsErrorVariant::UpdatedActorIdDoesNotExist {
                    update: update.clone(),
                })
            })
    }

    fn delete_actor(&mut self, actor_id: &boxcars::ActorId) -> BoxcarsResult<ActorState> {
        let state = self.actor_states.remove(actor_id).ok_or_else(|| {
            BoxcarsError::new(BoxcarsErrorVariant::NoStateForActorId {
                actor_id: actor_id.clone(),
            })
        })?;

        self.actor_ids_by_type
            .entry(state.object_id)
            .or_insert_with(|| Vec::new())
            .retain(|x| x != actor_id);

        Ok(state)
    }
}

pub type PlayerId = boxcars::RemoteId;

pub type ReplayProcessorResult<T> = Result<T, String>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DemolishInfo {
    pub time: f32,
    pub seconds_remaining: i32,
    pub frame: usize,
    pub attacker: PlayerId,
    pub victim: PlayerId,
    pub attacker_velocity: boxcars::Vector3f,
    pub victim_velocity: boxcars::Vector3f,
}

macro_rules! attribute_match {
    ($value:expr, $type:path $(,)?) => {{
        let attribute = $value;
        if let $type(value) = attribute {
            Ok(value)
        } else {
            BoxcarsError::new_result(BoxcarsErrorVariant::UnexpectedAttributeType {
                expected_type: stringify!(path).to_string(),
                actual_type: attribute_to_tag(&attribute).to_string(),
            })
        }
    }};
}

macro_rules! get_attribute_errors_expected {
    ($self:ident, $map:expr, $prop:expr, $type:path) => {
        $self
            .get_attribute($map, $prop)
            .and_then(|found| attribute_match!(found, $type))
    };
}

macro_rules! get_attribute_and_updated {
    ($self:ident, $map:expr, $prop:expr, $type:path) => {
        $self
            .get_attribute_and_updated($map, $prop)
            .and_then(|(found, updated)| attribute_match!(found, $type).map(|v| (v, updated)))
    };
}

macro_rules! get_actor_attribute_matching {
    ($self:ident, $actor:expr, $prop:expr, $type:path) => {
        $self
            .get_actor_attribute($actor, $prop)
            .and_then(|found| attribute_match!(found, $type))
    };
}

macro_rules! get_derived_attribute {
    ($map:expr, $key:expr, $type:path) => {
        $map.get($key)
            .ok_or_else(|| {
                BoxcarsError::new(BoxcarsErrorVariant::DerivedKeyValueNotFound {
                    name: $key.to_string(),
                })
            })
            .and_then(|found| attribute_match!(&found.0, $type))
    };
}

fn get_actor_id_from_active_actor<T>(
    _: T,
    active_actor: &boxcars::ActiveActor,
) -> boxcars::ActorId {
    active_actor.actor
}

fn use_update_actor<T>(id: boxcars::ActorId, _: T) -> boxcars::ActorId {
    id
}

pub struct ReplayProcessor<'a> {
    pub replay: &'a boxcars::Replay,
    pub actor_state: ActorStateModeler,
    pub object_id_to_name: HashMap<boxcars::ObjectId, String>,
    pub name_to_object_id: HashMap<String, boxcars::ObjectId>,
    pub ball_actor_id: Option<boxcars::ActorId>,
    pub team_zero: Vec<PlayerId>,
    pub team_one: Vec<PlayerId>,
    pub player_to_actor_id: HashMap<PlayerId, boxcars::ActorId>,
    pub player_to_car: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub player_to_team: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub car_to_boost: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub car_to_jump: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub car_to_double_jump: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub car_to_dodge: HashMap<boxcars::ActorId, boxcars::ActorId>,
    pub demolishes: Vec<DemolishInfo>,
    known_demolishes: Vec<(boxcars::DemolishFx, usize)>,
}

impl<'a> ReplayProcessor<'a> {
    // Initialization
    pub fn new(replay: &'a boxcars::Replay) -> BoxcarsResult<Self> {
        let mut object_id_to_name = HashMap::new();
        let mut name_to_object_id = HashMap::new();
        for (id, name) in replay.objects.iter().enumerate() {
            let object_id = boxcars::ObjectId(id as i32);
            object_id_to_name.insert(object_id, name.clone());
            name_to_object_id.insert(name.clone(), object_id);
        }
        let mut processor = Self {
            actor_state: ActorStateModeler::new(),
            replay,
            object_id_to_name,
            name_to_object_id,
            team_zero: Vec::new(),
            team_one: Vec::new(),
            ball_actor_id: None,
            player_to_car: HashMap::new(),
            player_to_team: HashMap::new(),
            player_to_actor_id: HashMap::new(),
            car_to_boost: HashMap::new(),
            car_to_jump: HashMap::new(),
            car_to_double_jump: HashMap::new(),
            car_to_dodge: HashMap::new(),
            demolishes: Vec::new(),
            known_demolishes: Vec::new(),
        };
        processor
            .set_player_order_from_headers()
            .or_else(|_| processor.set_player_order_from_frames())?;

        Ok(processor)
    }

    pub fn reset(&mut self) {
        self.player_to_car = HashMap::new();
        self.player_to_team = HashMap::new();
        self.player_to_actor_id = HashMap::new();
        self.car_to_boost = HashMap::new();
        self.car_to_jump = HashMap::new();
        self.car_to_double_jump = HashMap::new();
        self.car_to_dodge = HashMap::new();
        self.actor_state = ActorStateModeler::new();
        self.demolishes = Vec::new();
        self.known_demolishes = Vec::new();
    }

    fn set_player_order_from_headers(&mut self) -> BoxcarsResult<()> {
        let _player_stats = self
            .replay
            .properties
            .iter()
            .find(|(key, _)| key == "PlayerStats")
            .ok_or_else(|| BoxcarsError::new(BoxcarsErrorVariant::PlayerStatsHeaderNotFound))?;
        // XXX: implementation incomplete
        BoxcarsError::new_result(BoxcarsErrorVariant::PlayerStatsHeaderNotFound)
    }

    pub(crate) fn process_long_enough_to_get_actor_ids(&mut self) -> BoxcarsResult<()> {
        let mut handler = |_p: &ReplayProcessor, _f: &boxcars::Frame, n: usize, _current_time| {
            // XXX: 10 seconds should be enough to find everyone, right?
            if n > 10 * 30 {
                BoxcarsError::new_result(BoxcarsErrorVariant::FinishProcessingEarly)
            } else {
                Ok(TimeAdvance::NextFrame)
            }
        };
        let process_result = self.process(&mut handler);
        if let Some(BoxcarsErrorVariant::FinishProcessingEarly) =
            process_result.as_ref().err().map(|e| e.variant.clone())
        {
            Ok(())
        } else {
            process_result
        }
    }

    fn set_player_order_from_frames(&mut self) -> BoxcarsResult<()> {
        self.process_long_enough_to_get_actor_ids()?;
        let result: Result<HashMap<PlayerId, bool>, _> = self
            .player_to_actor_id
            .keys()
            .map(|player_id| Ok((player_id.clone(), self.get_player_is_team_0(player_id)?)))
            .collect();

        let player_to_team_0 = result?;

        let (team_zero, team_one): (Vec<_>, Vec<_>) = player_to_team_0
            .keys()
            .cloned()
            // The unwrap here is fine because we know the get will succeed
            .partition(|player_id| *player_to_team_0.get(player_id).unwrap());

        self.team_zero = team_zero;
        self.team_one = team_one;

        self.team_zero
            .sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        self.team_one
            .sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));

        self.reset();
        Ok(())
    }

    pub fn process<H: Collector>(&mut self, handler: &mut H) -> BoxcarsResult<()> {
        // Initially, we set target_time to NextFrame to ensure the collector
        // will process the first frame.
        let mut target_time = TimeAdvance::NextFrame;
        for (index, frame) in self
            .replay
            .network_frames
            .as_ref()
            .ok_or(BoxcarsError::new(BoxcarsErrorVariant::NoNetworkFrames))?
            .frames
            .iter()
            .enumerate()
        {
            // Update the internal state of the processor based on the current frame
            self.actor_state.process_frame(frame, index)?;
            self.update_mappings(frame)?;
            self.update_ball_id(frame)?;
            self.update_boost_amounts(frame, index)?;
            self.update_demolishes(frame, index)?;

            // Get the time to process for this frame. If target_time is set to
            // NextFrame, we use the time of the current frame.
            let mut current_time = match &target_time {
                TimeAdvance::Time(t) => *t,
                TimeAdvance::NextFrame => frame.time,
            };

            while current_time <= frame.time {
                // Call the handler to process the frame and get the time for
                // the next frame the handler wants to process
                target_time = handler.process_frame(&self, frame, index, current_time)?;
                // If the handler specified a specific time, update current_time
                // to that time. If the handler specified NextFrame, we break
                // out of the loop to move on to the next frame in the replay.
                // This design allows the handler to have control over the frame
                // rate, including the possibility of skipping frames.
                if let TimeAdvance::Time(new_target) = target_time {
                    current_time = new_target;
                } else {
                    break;
                }
            }
        }
        // Make sure that we didn't encounter any players we did not know about
        // at the beggining of the replay.
        self.check_player_id_set()
    }

    fn check_player_id_set(&self) -> BoxcarsResult<()> {
        let known_players =
            std::collections::HashSet::<_>::from_iter(self.player_to_actor_id.keys());
        let original_players =
            std::collections::HashSet::<_>::from_iter(self.iter_player_ids_in_order());

        if original_players != known_players {
            return BoxcarsError::new_result(BoxcarsErrorVariant::InconsistentPlayerSet {
                found: known_players.into_iter().cloned().collect(),
                original: original_players.into_iter().cloned().collect(),
            });
        } else {
            Ok(())
        }
    }

    pub fn process_and_get_replay_meta(&mut self) -> BoxcarsResult<ReplayMeta> {
        if self.player_to_actor_id.is_empty() {
            self.process_long_enough_to_get_actor_ids()?;
        }
        self.get_replay_meta()
    }

    pub fn get_replay_meta(&self) -> BoxcarsResult<ReplayMeta> {
        let empty_player_stats = Vec::new();
        let player_stats = if let Some((_, boxcars::HeaderProp::Array(per_player))) = self
            .replay
            .properties
            .iter()
            .find(|(key, _)| key == "PlayerStats")
        {
            per_player
        } else {
            &empty_player_stats
        };
        let known_count = self.iter_player_ids_in_order().count();
        if player_stats.len() != known_count {
            log::warn!(
                "Replay does not have player stats for all players. encountered {:?} {:?}",
                known_count,
                player_stats.len()
            )
        }
        let get_player_info = |player_id| {
            let name = self.get_player_name(player_id)?;
            let stats = find_player_stats(player_id, &name, player_stats).ok();
            Ok(PlayerInfo {
                name,
                stats,
                remote_id: player_id.clone(),
            })
        };
        let team_zero: BoxcarsResult<Vec<PlayerInfo>> =
            self.team_zero.iter().map(get_player_info).collect();
        let team_one: BoxcarsResult<Vec<PlayerInfo>> =
            self.team_one.iter().map(get_player_info).collect();
        Ok(ReplayMeta {
            team_zero: team_zero?,
            team_one: team_one?,
            all_headers: self.replay.properties.clone(),
        })
    }

    fn find_update_in_direction(
        &self,
        current_index: usize,
        actor_id: &boxcars::ActorId,
        object_id: &boxcars::ObjectId,
        direction: SearchDirection,
    ) -> BoxcarsResult<(boxcars::Attribute, usize)> {
        let frames = self
            .replay
            .network_frames
            .as_ref()
            .ok_or(BoxcarsError::new(BoxcarsErrorVariant::NoNetworkFrames))?;

        let predicate = |frame: &boxcars::Frame| {
            frame
                .updated_actors
                .iter()
                .find(|update| &update.actor_id == actor_id && &update.object_id == object_id)
                .map(|update| &update.attribute)
                .cloned()
        };

        match util::find_in_direction(&frames.frames, current_index, direction, predicate) {
            Some((index, attribute)) => Ok((attribute, index)),
            None => BoxcarsError::new_result(BoxcarsErrorVariant::NoUpdateAfterFrame {
                actor_id: actor_id.clone(),
                object_id: object_id.clone(),
                frame_index: current_index,
            }),
        }
    }

    // Update functions

    fn update_mappings(&mut self, frame: &boxcars::Frame) -> BoxcarsResult<()> {
        for update in frame.updated_actors.iter() {
            macro_rules! maintain_link {
                ($map:expr, $actor_type:expr, $attr:expr, $get_key: expr, $get_value: expr, $type:path) => {{
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
                            let _key = $get_key(update.actor_id, value);
                            let _new_value = $get_value(update.actor_id, value);
                            let _old_value = $map.insert(
                                $get_key(update.actor_id, value),
                                $get_value(update.actor_id, value),
                            );
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
                        // This is slightly confusing, but in these cases we are
                        // using the attribute as the key to the current actor.
                        get_actor_id_from_active_actor,
                        use_update_actor,
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
                |_, unique_id: &Box<boxcars::UniqueId>| unique_id.remote_id.clone(),
                use_update_actor,
                boxcars::Attribute::UniqueId
            );
            maintain_link!(
                self.player_to_team,
                PLAYER_TYPE,
                TEAM_KEY,
                // In this case we are using the update actor as the key.
                use_update_actor,
                get_actor_id_from_active_actor,
                boxcars::Attribute::ActiveActor
            );
            maintain_actor_link!(self.player_to_car, CAR_TYPE, PLAYER_REPLICATION_KEY);
            maintain_vehicle_key_link!(self.car_to_boost, BOOST_TYPE);
            maintain_vehicle_key_link!(self.car_to_dodge, DODGE_TYPE);
            maintain_vehicle_key_link!(self.car_to_jump, JUMP_TYPE);
            maintain_vehicle_key_link!(self.car_to_double_jump, DOUBLE_JUMP_TYPE);
        }

        for actor_id in frame.deleted_actors.iter() {
            self.player_to_car.remove(actor_id).map(|car_id| {
                log::info!("Player actor {:?} deleted, car id: {:?}.", actor_id, car_id)
            });
        }

        Ok(())
    }

    fn update_ball_id(&mut self, frame: &boxcars::Frame) -> BoxcarsResult<()> {
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

    fn update_boost_amounts(
        &mut self,
        frame: &boxcars::Frame,
        frame_index: usize,
    ) -> BoxcarsResult<()> {
        let updates: Vec<_> = self
            .iter_actors_by_type_err(BOOST_TYPE)?
            .map(|(actor_id, actor_state)| {
                let (actor_amount_value, last_value, _, derived_value, is_active) =
                    self.get_current_boost_values(actor_state);
                let mut current_value = if actor_amount_value == last_value {
                    // If we don't have an update in the actor, just continue
                    // using our derived value
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
                // This actor is known to exist, so unwrap is fine
                .unwrap()
                .derived_attributes;

            derived_attributes.insert(
                LAST_BOOST_AMOUNT_KEY.to_string(),
                (boxcars::Attribute::Byte(new_last_value), frame_index),
            );
            derived_attributes.insert(
                BOOST_AMOUNT_KEY.to_string(),
                (boxcars::Attribute::Float(current_value), frame_index),
            );
        }
        Ok(())
    }

    fn get_current_boost_values(&self, actor_state: &ActorState) -> (u8, u8, u8, f32, bool) {
        let amount_value = get_attribute_errors_expected!(
            self,
            &actor_state.attributes,
            BOOST_AMOUNT_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
        .unwrap_or(0);
        let active_value = get_attribute_errors_expected!(
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
            .cloned()
            .and_then(|v| attribute_match!(v.0, boxcars::Attribute::Float).ok())
            .unwrap_or(0.0);
        let last_boost_amount = attribute_match!(
            actor_state
                .derived_attributes
                .get(&LAST_BOOST_AMOUNT_KEY.to_string())
                .cloned()
                .map(|v| v.0)
                .unwrap_or_else(|| boxcars::Attribute::Byte(amount_value)),
            boxcars::Attribute::Byte
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

    fn update_demolishes(&mut self, frame: &boxcars::Frame, index: usize) -> BoxcarsResult<()> {
        let new_demolishes: Vec<_> = self
            .get_active_demolish_fx()?
            .flat_map(|demolish_fx| {
                if !self.demolish_is_known(&demolish_fx, index) {
                    Some(demolish_fx.as_ref().clone())
                } else {
                    None
                }
            })
            .collect();

        for demolish in new_demolishes {
            match self.build_demolish_info(&demolish, frame, index) {
                Ok(demolish_info) => self.demolishes.push(demolish_info),
                Err(_e) => {
                    log::warn!("Error building demolish info");
                }
            }
            self.known_demolishes.push((demolish, index))
        }

        Ok(())
    }

    fn build_demolish_info(
        &self,
        demolish_fx: &boxcars::DemolishFx,
        frame: &boxcars::Frame,
        index: usize,
    ) -> BoxcarsResult<DemolishInfo> {
        let attacker = self.get_player_id_from_car_id(&demolish_fx.attacker)?;
        let victim = self.get_player_id_from_car_id(&demolish_fx.victim)?;
        Ok(DemolishInfo {
            time: frame.time,
            seconds_remaining: self.get_seconds_remaining()?,
            frame: index,
            attacker,
            victim,
            attacker_velocity: demolish_fx.attack_velocity.clone(),
            victim_velocity: demolish_fx.victim_velocity.clone(),
        })
    }

    // ID Mapping functions

    pub fn get_player_id_from_car_id(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> BoxcarsResult<PlayerId> {
        self.get_player_id_from_actor_id(&self.get_player_actor_id_from_car_actor_id(actor_id)?)
    }

    fn get_player_id_from_actor_id(&self, actor_id: &boxcars::ActorId) -> BoxcarsResult<PlayerId> {
        for (player_id, player_actor_id) in self.player_to_actor_id.iter() {
            if actor_id == player_actor_id {
                return Ok(player_id.clone());
            }
        }
        return BoxcarsError::new_result(BoxcarsErrorVariant::NoMatchingPlayerId {
            actor_id: actor_id.clone(),
        });
    }

    fn get_player_actor_id_from_car_actor_id(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> BoxcarsResult<boxcars::ActorId> {
        for (player_id, car_id) in self.player_to_car.iter() {
            if actor_id == car_id {
                return Ok(player_id.clone());
            }
        }
        return BoxcarsError::new_result(BoxcarsErrorVariant::NoMatchingPlayerId {
            actor_id: actor_id.clone(),
        });
    }

    fn demolish_is_known(&self, demolish_fx: &boxcars::DemolishFx, frame_index: usize) -> bool {
        self.known_demolishes.iter().any(|(existing, index)| {
            existing == demolish_fx
                && frame_index
                    .checked_sub(*index)
                    .or_else(|| index.checked_sub(frame_index))
                    .unwrap()
                    < MAX_DEMOLISH_KNOWN_FRAMES_PASSED
        })
    }

    pub fn get_active_demolish_fx(
        &self,
    ) -> BoxcarsResult<impl Iterator<Item = &Box<boxcars::DemolishFx>>> {
        Ok(self
            .iter_actors_by_type_err(CAR_TYPE)?
            .flat_map(|(_actor_id, state)| {
                get_attribute_errors_expected!(
                    self,
                    &state.attributes,
                    DEMOLISH_GOAL_EXPLOSION_KEY,
                    boxcars::Attribute::DemolishFx
                )
                .ok()
            }))
    }

    // Interpolation Support functions

    fn get_frame(&self, frame_index: usize) -> BoxcarsResult<&boxcars::Frame> {
        self.replay
            .network_frames
            .as_ref()
            .ok_or(BoxcarsError::new(BoxcarsErrorVariant::NoNetworkFrames))?
            .frames
            .get(frame_index)
            .ok_or(BoxcarsError::new(
                BoxcarsErrorVariant::FrameIndexOutOfBounds,
            ))
    }

    fn velocities_applied_rigid_body(
        &self,
        rigid_body: &boxcars::RigidBody,
        rb_frame_index: usize,
        target_time: f32,
    ) -> BoxcarsResult<boxcars::RigidBody> {
        let rb_frame = self.get_frame(rb_frame_index)?;
        let interpolation_amount = target_time - rb_frame.time;
        Ok(apply_velocities_to_rigid_body(
            rigid_body,
            interpolation_amount,
        ))
    }

    fn get_interpolated_actor_rigid_body(
        &self,
        actor_id: &boxcars::ActorId,
        time: f32,
        close_enough: f32,
    ) -> BoxcarsResult<boxcars::RigidBody> {
        let (frame_body, frame_index) = self.get_actor_rigid_body(actor_id)?;
        let frame_time = self.get_frame(*frame_index)?.time;
        let time_and_frame_difference = time - frame_time;

        if (time_and_frame_difference).abs() <= close_enough.abs() {
            return Ok(frame_body.clone());
        }

        let search_direction = if time_and_frame_difference > 0.0 {
            util::SearchDirection::Forward
        } else {
            util::SearchDirection::Backward
        };

        let object_id = self.get_object_id_for_key(RIGID_BODY_STATE_KEY)?;

        let (attribute, found_frame) =
            self.find_update_in_direction(*frame_index, &actor_id, object_id, search_direction)?;
        let found_time = self.get_frame(found_frame)?.time;

        let found_body = attribute_match!(attribute, boxcars::Attribute::RigidBody)?;

        if (found_time - time).abs() <= close_enough {
            return Ok(found_body.clone());
        }

        let (start_body, start_time, end_body, end_time) = match search_direction {
            util::SearchDirection::Forward => (frame_body, frame_time, &found_body, found_time),
            util::SearchDirection::Backward => (&found_body, found_time, frame_body, frame_time),
        };

        util::get_interpolated_rigid_body(start_body, start_time, end_body, end_time, time)
    }

    // Actor functions

    fn get_object_id_for_key(&self, name: &'static str) -> BoxcarsResult<&boxcars::ObjectId> {
        self.name_to_object_id
            .get(name)
            .ok_or_else(|| BoxcarsError::new(BoxcarsErrorVariant::ObjectIdNotFound { name }))
    }

    fn get_actor_ids_by_type(&self, name: &'static str) -> BoxcarsResult<&[boxcars::ActorId]> {
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

    fn get_actor_state(&self, actor_id: &boxcars::ActorId) -> BoxcarsResult<&ActorState> {
        self.actor_state.actor_states.get(actor_id).ok_or_else(|| {
            BoxcarsError::new(BoxcarsErrorVariant::NoStateForActorId {
                actor_id: actor_id.clone(),
            })
        })
    }

    fn get_actor_attribute<'b>(
        &'b self,
        actor_id: &boxcars::ActorId,
        property: &'static str,
    ) -> BoxcarsResult<&'b boxcars::Attribute> {
        self.get_attribute(&self.get_actor_state(actor_id)?.attributes, property)
    }

    fn get_attribute<'b>(
        &'b self,
        map: &'b HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
        property: &'static str,
    ) -> BoxcarsResult<&'b boxcars::Attribute> {
        self.get_attribute_and_updated(map, property).map(|v| &v.0)
    }

    fn get_attribute_and_updated<'b>(
        &'b self,
        map: &'b HashMap<boxcars::ObjectId, (boxcars::Attribute, usize)>,
        property: &'static str,
    ) -> BoxcarsResult<&'b (boxcars::Attribute, usize)> {
        let attribute_object_id = self.get_object_id_for_key(property)?;
        map.get(attribute_object_id).ok_or_else(|| {
            BoxcarsError::new(BoxcarsErrorVariant::PropertyNotFoundInState { property })
        })
    }

    fn find_ball_actor(&self) -> Option<boxcars::ActorId> {
        BALL_TYPES
            .iter()
            .filter_map(|ball_type| self.iter_actors_by_type(ball_type))
            .flat_map(|i| i)
            .map(|(actor_id, _)| actor_id.clone())
            .next()
    }

    fn get_ball_actor(&self) -> BoxcarsResult<boxcars::ActorId> {
        self.ball_actor_id
            .ok_or(BoxcarsError::new(BoxcarsErrorVariant::BallActorNotFound))
    }

    pub fn get_metadata_actor_id(&self) -> BoxcarsResult<&boxcars::ActorId> {
        self.get_actor_ids_by_type(GAME_TYPE)?
            .iter()
            .next()
            .ok_or_else(|| BoxcarsError::new(BoxcarsErrorVariant::NoGameActor))
    }

    pub fn get_player_actor_id(&self, player_id: &PlayerId) -> BoxcarsResult<boxcars::ActorId> {
        self.player_to_actor_id
            .get(&player_id)
            .ok_or_else(|| {
                BoxcarsError::new(BoxcarsErrorVariant::ActorNotFound {
                    name: "ActorId",
                    player_id: player_id.clone(),
                })
            })
            .cloned()
    }

    pub fn get_car_actor_id(&self, player_id: &PlayerId) -> BoxcarsResult<boxcars::ActorId> {
        self.player_to_car
            .get(&self.get_player_actor_id(player_id)?)
            .ok_or_else(|| {
                BoxcarsError::new(BoxcarsErrorVariant::ActorNotFound {
                    name: "Car",
                    player_id: player_id.clone(),
                })
            })
            .cloned()
    }

    pub fn get_car_connected_actor_id(
        &self,
        player_id: &PlayerId,
        map: &HashMap<boxcars::ActorId, boxcars::ActorId>,
        name: &'static str,
    ) -> BoxcarsResult<boxcars::ActorId> {
        map.get(&self.get_car_actor_id(player_id)?)
            .ok_or_else(|| {
                BoxcarsError::new(BoxcarsErrorVariant::ActorNotFound {
                    name,
                    player_id: player_id.clone(),
                })
            })
            .cloned()
    }

    pub fn get_boost_actor_id(&self, player_id: &PlayerId) -> BoxcarsResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_boost, "Boost")
    }

    pub fn get_jump_actor_id(&self, player_id: &PlayerId) -> BoxcarsResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_jump, "Jump")
    }

    pub fn get_double_jump_actor_id(
        &self,
        player_id: &PlayerId,
    ) -> BoxcarsResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_double_jump, "Double Jump")
    }

    pub fn get_dodge_actor_id(&self, player_id: &PlayerId) -> BoxcarsResult<boxcars::ActorId> {
        self.get_car_connected_actor_id(player_id, &self.car_to_dodge, "Dodge")
    }

    pub fn get_actor_rigid_body(
        &self,
        actor_id: &boxcars::ActorId,
    ) -> BoxcarsResult<(&boxcars::RigidBody, &usize)> {
        get_attribute_and_updated!(
            self,
            &self.get_actor_state(&actor_id)?.attributes,
            RIGID_BODY_STATE_KEY,
            boxcars::Attribute::RigidBody
        )
    }

    // Actor iteration functions

    pub fn iter_player_ids_in_order(&self) -> impl Iterator<Item = &PlayerId> {
        self.team_zero.iter().chain(self.team_one.iter())
    }

    pub fn player_count(&self) -> usize {
        self.iter_player_ids_in_order().count()
    }

    fn iter_actors_by_type_err(
        &self,
        name: &'static str,
    ) -> BoxcarsResult<impl Iterator<Item = (&boxcars::ActorId, &ActorState)>> {
        Ok(self.iter_actors_by_object_id(self.get_object_id_for_key(name)?))
    }

    pub fn iter_actors_by_type(
        &self,
        name: &'static str,
    ) -> Option<impl Iterator<Item = (&boxcars::ActorId, &ActorState)>> {
        self.iter_actors_by_type_err(name).ok()
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
            // This unwrap is fine because we know the actor will exist as it is
            // in the actor_ids_by_type
            .map(move |id| (id, self.actor_state.actor_states.get(id).unwrap()))
    }

    // Properties

    pub fn get_seconds_remaining(&self) -> BoxcarsResult<i32> {
        get_actor_attribute_matching!(
            self,
            self.get_metadata_actor_id()?,
            SECONDS_REMAINING_KEY,
            boxcars::Attribute::Int
        )
        .cloned()
    }

    pub fn get_ignore_ball_syncing(&self) -> BoxcarsResult<bool> {
        let actor_id = self.get_ball_actor()?;
        get_actor_attribute_matching!(
            self,
            &actor_id,
            IGNORE_SYNCING_KEY,
            boxcars::Attribute::Boolean
        )
        .cloned()
    }

    pub fn get_ball_rigid_body(&self) -> BoxcarsResult<&boxcars::RigidBody> {
        self.ball_actor_id
            .ok_or(BoxcarsError::new(BoxcarsErrorVariant::BallActorNotFound))
            .and_then(|actor_id| self.get_actor_rigid_body(&actor_id).map(|v| v.0))
    }

    pub fn ball_rigid_body_exists(&self) -> BoxcarsResult<bool> {
        Ok(self
            .get_ball_rigid_body()
            .map(|rb| !rb.sleeping)
            .unwrap_or(false))
    }

    pub fn get_ball_rigid_body_and_updated(&self) -> BoxcarsResult<(&boxcars::RigidBody, &usize)> {
        self.ball_actor_id
            .ok_or(BoxcarsError::new(BoxcarsErrorVariant::BallActorNotFound))
            .and_then(|actor_id| {
                get_attribute_and_updated!(
                    self,
                    &self.get_actor_state(&actor_id)?.attributes,
                    RIGID_BODY_STATE_KEY,
                    boxcars::Attribute::RigidBody
                )
            })
    }

    pub fn get_velocity_applied_ball_rigid_body(
        &self,
        target_time: f32,
    ) -> BoxcarsResult<boxcars::RigidBody> {
        let (current_rigid_body, frame_index) = self.get_ball_rigid_body_and_updated()?;
        self.velocities_applied_rigid_body(&current_rigid_body, *frame_index, target_time)
    }

    pub fn get_interpolated_ball_rigid_body(
        &self,
        time: f32,
        close_enough: f32,
    ) -> BoxcarsResult<boxcars::RigidBody> {
        self.get_interpolated_actor_rigid_body(&self.get_ball_actor()?, time, close_enough)
    }

    pub fn get_player_name(&self, player_id: &PlayerId) -> BoxcarsResult<String> {
        get_actor_attribute_matching!(
            self,
            &self.get_player_actor_id(player_id)?,
            PLAYER_NAME_KEY,
            boxcars::Attribute::String
        )
        .cloned()
    }

    pub fn get_player_team_key(&self, player_id: &PlayerId) -> BoxcarsResult<String> {
        let team_actor_id = self
            .player_to_team
            .get(&self.get_player_actor_id(player_id)?)
            .ok_or_else(|| {
                BoxcarsError::new(BoxcarsErrorVariant::UnknownPlayerTeam {
                    player_id: player_id.clone(),
                })
            })?;
        let state = self.get_actor_state(team_actor_id)?;
        self.object_id_to_name
            .get(&state.object_id)
            .ok_or_else(|| {
                BoxcarsError::new(BoxcarsErrorVariant::UnknownPlayerTeam {
                    player_id: player_id.clone(),
                })
            })
            .cloned()
    }

    pub fn get_player_is_team_0(&self, player_id: &PlayerId) -> BoxcarsResult<bool> {
        Ok(self
            .get_player_team_key(player_id)?
            .chars()
            .last()
            .ok_or_else(|| {
                BoxcarsError::new(BoxcarsErrorVariant::EmptyTeamName {
                    player_id: player_id.clone(),
                })
            })?
            == '0')
    }

    pub fn get_player_rigid_body(
        &self,
        player_id: &PlayerId,
    ) -> BoxcarsResult<&boxcars::RigidBody> {
        self.get_car_actor_id(player_id)
            .and_then(|actor_id| self.get_actor_rigid_body(&actor_id).map(|v| v.0))
    }

    pub fn get_player_rigid_body_and_updated(
        &self,
        player_id: &PlayerId,
    ) -> BoxcarsResult<(&boxcars::RigidBody, &usize)> {
        self.get_car_actor_id(player_id).and_then(|actor_id| {
            get_attribute_and_updated!(
                self,
                &self.get_actor_state(&actor_id)?.attributes,
                RIGID_BODY_STATE_KEY,
                boxcars::Attribute::RigidBody
            )
        })
    }

    pub fn get_velocity_applied_player_rigid_body(
        &self,
        player_id: &PlayerId,
        target_time: f32,
    ) -> BoxcarsResult<boxcars::RigidBody> {
        let (current_rigid_body, frame_index) =
            self.get_player_rigid_body_and_updated(player_id)?;
        self.velocities_applied_rigid_body(&current_rigid_body, *frame_index, target_time)
    }

    pub fn get_interpolated_player_rigid_body(
        &self,
        player_id: &PlayerId,
        time: f32,
        close_enough: f32,
    ) -> BoxcarsResult<boxcars::RigidBody> {
        self.get_interpolated_actor_rigid_body(
            &self.get_car_actor_id(player_id).unwrap(),
            time,
            close_enough,
        )
    }

    pub fn get_player_boost_level(&self, player_id: &PlayerId) -> BoxcarsResult<f32> {
        self.get_boost_actor_id(player_id).and_then(|actor_id| {
            let boost_state = self.get_actor_state(&actor_id)?;
            get_derived_attribute!(
                boost_state.derived_attributes,
                BOOST_AMOUNT_KEY,
                boxcars::Attribute::Float
            )
            .cloned()
        })
    }

    pub fn get_component_active(&self, actor_id: &boxcars::ActorId) -> BoxcarsResult<u8> {
        get_actor_attribute_matching!(
            self,
            &actor_id,
            COMPONENT_ACTIVE_KEY,
            boxcars::Attribute::Byte
        )
        .cloned()
    }

    pub fn get_boost_active(&self, player_id: &PlayerId) -> BoxcarsResult<u8> {
        self.get_boost_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    pub fn get_jump_active(&self, player_id: &PlayerId) -> BoxcarsResult<u8> {
        self.get_jump_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    pub fn get_double_jump_active(&self, player_id: &PlayerId) -> BoxcarsResult<u8> {
        self.get_double_jump_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    pub fn get_dodge_active(&self, player_id: &PlayerId) -> BoxcarsResult<u8> {
        self.get_dodge_actor_id(player_id)
            .and_then(|actor_id| self.get_component_active(&actor_id))
    }

    // Debugging

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

    pub fn all_mappings_string(&self) -> String {
        let pairs = [
            ("player_to_car", &self.player_to_car),
            ("player_to_team", &self.player_to_team),
            ("car_to_boost", &self.car_to_boost),
            ("car_to_jump", &self.car_to_jump),
            ("car_to_double_jump", &self.car_to_double_jump),
            ("car_to_dodge", &self.car_to_dodge),
        ];
        let strings: Vec<_> = pairs
            .iter()
            .map(|(map_name, map)| format!("{:?}: {:?}", map_name, map))
            .collect();
        strings.join("\n")
    }

    pub fn actor_state_string(&self, actor_id: &boxcars::ActorId) -> String {
        format!(
            "{:?}",
            self.get_actor_state(actor_id)
                .map(|s| self.map_attribute_keys(&s.attributes))
        )
    }

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

    pub fn print_actors_of_type(&self, actor_type: &'static str) {
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

    pub fn print_all_actors(&self) {
        self.actor_state
            .actor_states
            .iter()
            .for_each(|(actor_id, _actor_state)| {
                println!("{:?}", self.actor_state_string(actor_id))
            })
    }
}
