use boxcars::{HeaderProp, RemoteId};
use serde::Serialize;

use crate::*;

macro_rules! fmt_err {
    ($( $item:expr ),* $(,)?) => {
        Err(format!($( $item ),*))
    };
}

pub type PlayerId = boxcars::RemoteId;

/// [`DemolishInfo`] struct represents data related to a demolition event in the game.
///
/// Demolition events occur when one player 'demolishes' or 'destroys' another by
/// hitting them at a sufficiently high speed. This results in the demolished player
/// being temporarily removed from play.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DemolishInfo {
    /// The exact game time (in seconds) at which the demolition event occurred.
    pub time: f32,
    /// The remaining time in the match when the demolition event occurred.
    pub seconds_remaining: i32,
    /// The frame number at which the demolition occurred.
    pub frame: usize,
    /// The [`PlayerId`] of the player who initiated the demolition.
    pub attacker: PlayerId,
    /// The [`PlayerId`] of the player who was demolished.
    pub victim: PlayerId,
    /// The velocity of the attacker at the time of demolition.
    pub attacker_velocity: boxcars::Vector3f,
    /// The velocity of the victim at the time of demolition.
    pub victim_velocity: boxcars::Vector3f,
}

/// [`ReplayMeta`] struct represents metadata about the replay being processed.
///
/// This includes information about the players in the match and all replay headers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayMeta {
    /// A vector of [`PlayerInfo`] instances representing the players on team zero.
    pub team_zero: Vec<PlayerInfo>,
    /// A vector of [`PlayerInfo`] instances representing the players on team one.
    pub team_one: Vec<PlayerInfo>,
    /// A vector of tuples containing the names and properties of all the headers in the replay.
    pub all_headers: Vec<(String, HeaderProp)>,
}

impl ReplayMeta {
    /// Returns the total number of players involved in the game.
    pub fn player_count(&self) -> usize {
        self.team_one.len() + self.team_zero.len()
    }

    /// Returns an iterator over the [`PlayerInfo`] instances representing the players,
    /// in the order they are listed in the replay file.
    pub fn player_order(&self) -> impl Iterator<Item = &PlayerInfo> {
        self.team_zero.iter().chain(self.team_one.iter())
    }
}

/// [`PlayerInfo`] struct provides detailed information about a specific player in the replay.
///
/// This includes player's unique remote ID, player stats if available, and their name.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerInfo {
    /// The unique remote ID of the player. This could be their online ID or local ID.
    pub remote_id: RemoteId,
    /// An optional HashMap containing player-specific stats.
    /// The keys of this HashMap are the names of the stats,
    /// and the values are the corresponding `HeaderProp` instances.
    pub stats: Option<std::collections::HashMap<String, HeaderProp>>,
    /// The name of the player as represented in the replay.
    pub name: String,
}

pub fn find_player_stats(
    player_id: &RemoteId,
    name: &String,
    all_player_stats: &Vec<Vec<(String, HeaderProp)>>,
) -> Result<std::collections::HashMap<String, HeaderProp>, String> {
    Ok(all_player_stats
        .iter()
        .find(|player_stats| matches_stats(player_id, name, player_stats))
        .ok_or(format!(
            "Player not found {player_id:?} {all_player_stats:?}"
        ))?
        .iter()
        .cloned()
        .collect())
}

fn matches_stats(player_id: &RemoteId, name: &String, props: &Vec<(String, HeaderProp)>) -> bool {
    if platform_matches(player_id, props) != Ok(true) {
        return false;
    }
    match player_id {
        RemoteId::Epic(_) => name_matches(name, props),
        RemoteId::Steam(id) => online_id_matches(*id, props),
        RemoteId::Xbox(id) => online_id_matches(*id, props),
        RemoteId::PlayStation(ps4id) => online_id_matches(ps4id.online_id, props),
        RemoteId::PsyNet(psynet_id) => online_id_matches(psynet_id.online_id, props),
        RemoteId::Switch(switch_id) => online_id_matches(switch_id.online_id, props),
        _ => false,
    }
}

fn name_matches(name: &String, props: &[(String, HeaderProp)]) -> bool {
    if let Ok((_, HeaderProp::Str(stat_name))) = get_prop("Name", props) {
        *name == stat_name
    } else {
        false
    }
}

fn online_id_matches(id: u64, props: &[(String, HeaderProp)]) -> bool {
    if let Ok((_, HeaderProp::QWord(props_id))) = get_prop("OnlineID", props) {
        id == props_id
    } else {
        false
    }
}

fn platform_matches(
    player_id: &RemoteId,
    props: &Vec<(String, HeaderProp)>,
) -> Result<bool, String> {
    if let (
        _,
        HeaderProp::Byte {
            kind: _,
            value: Some(value),
        },
    ) = get_prop("Platform", props)?
    {
        Ok(match (player_id, value.as_ref()) {
            (RemoteId::Steam(_), "OnlinePlatform_Steam") => true,
            (RemoteId::PlayStation(_), "OnlinePlatform_PS4") => true,
            (RemoteId::Epic(_), "OnlinePlatform_Epic") => true,
            (RemoteId::PsyNet(_), "OnlinePlatform_PS4") => true,
            (RemoteId::Xbox(_), "OnlinePlatform_Dingo") => true,
            // XXX: not sure if this is right.
            (RemoteId::Switch(_), "OnlinePlatform_Switch") => true,
            // TODO: There are still a few cases remaining.
            _ => false,
        })
    } else {
        fmt_err!("Unexpected platform value {:?}", props)
    }
}

fn get_prop(prop: &str, props: &[(String, HeaderProp)]) -> Result<(String, HeaderProp), String> {
    props
        .iter()
        .find(|(attr, _)| attr == prop)
        .ok_or("Coudn't find name property".to_string())
        .cloned()
}

pub(crate) trait VecMapEntry<K: PartialEq, V> {
    fn get_entry(&mut self, key: K) -> Entry<K, V>;
}

pub(crate) enum Entry<'a, K: PartialEq, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K: PartialEq, V> Entry<'a, K, V> {
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(occupied) => &mut occupied.entry.1,
            Entry::Vacant(vacant) => {
                vacant.vec.push((vacant.key, default()));
                &mut vacant.vec.last_mut().unwrap().1
            }
        }
    }
}

pub(crate) struct OccupiedEntry<'a, K: PartialEq, V> {
    entry: &'a mut (K, V),
}

pub(crate) struct VacantEntry<'a, K: PartialEq, V> {
    vec: &'a mut Vec<(K, V)>,
    key: K,
}

impl<K: PartialEq + Clone, V> VecMapEntry<K, V> for Vec<(K, V)> {
    fn get_entry(&mut self, key: K) -> Entry<K, V> {
        match self.iter_mut().position(|(k, _)| k == &key) {
            Some(index) => Entry::Occupied(OccupiedEntry {
                entry: &mut self[index],
            }),
            None => Entry::Vacant(VacantEntry { vec: self, key }),
        }
    }
}

pub fn vec_to_glam(v: &boxcars::Vector3f) -> glam::f32::Vec3 {
    glam::f32::Vec3::new(v.x, v.y, v.z)
}

pub fn glam_to_vec(v: &glam::f32::Vec3) -> boxcars::Vector3f {
    boxcars::Vector3f {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

pub fn quat_to_glam(q: &boxcars::Quaternion) -> glam::Quat {
    glam::Quat::from_xyzw(q.x, q.y, q.z, q.w)
}

pub fn glam_to_quat(rotation: &glam::Quat) -> boxcars::Quaternion {
    boxcars::Quaternion {
        x: rotation.x,
        y: rotation.y,
        z: rotation.z,
        w: rotation.w,
    }
}

pub fn apply_velocities_to_rigid_body(
    rigid_body: &boxcars::RigidBody,
    time_delta: f32,
) -> boxcars::RigidBody {
    let mut interpolated = *rigid_body;
    if time_delta == 0.0 {
        return interpolated;
    }
    let linear_velocity = interpolated.linear_velocity.unwrap_or(boxcars::Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    let location = vec_to_glam(&rigid_body.location) + (time_delta * vec_to_glam(&linear_velocity));
    interpolated.location = glam_to_vec(&location);
    interpolated.rotation = apply_angular_velocity(rigid_body, time_delta);
    interpolated
}

fn apply_angular_velocity(rigid_body: &boxcars::RigidBody, time_delta: f32) -> boxcars::Quaternion {
    // XXX: This approach seems to give some unexpected results. There may be a
    // unit mismatch or some other type of issue.
    let rbav = rigid_body.angular_velocity.unwrap_or(boxcars::Vector3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    let angular_velocity = glam::Vec3::new(rbav.x, rbav.y, rbav.z);
    let magnitude = angular_velocity.length();
    let angular_velocity_unit_vector = angular_velocity.normalize_or_zero();

    let mut rotation = glam::Quat::from_xyzw(
        rigid_body.rotation.x,
        rigid_body.rotation.y,
        rigid_body.rotation.z,
        rigid_body.rotation.w,
    );

    if angular_velocity_unit_vector.length() != 0.0 {
        let delta_rotation =
            glam::Quat::from_axis_angle(angular_velocity_unit_vector, magnitude * time_delta);
        rotation *= delta_rotation;
    }

    boxcars::Quaternion {
        x: rotation.x,
        y: rotation.y,
        z: rotation.z,
        w: rotation.w,
    }
}

/// Interpolates between two [`boxcars::RigidBody`] states based on the provided time.
///
/// # Arguments
///
/// * `start_body` - The initial `RigidBody` state.
/// * `start_time` - The timestamp of the initial `RigidBody` state.
/// * `end_body` - The final `RigidBody` state.
/// * `end_time` - The timestamp of the final `RigidBody` state.
/// * `time` - The desired timestamp to interpolate to.
///
/// # Returns
///
/// A new [`boxcars::RigidBody`] that represents the interpolated state at the specified time.
pub fn get_interpolated_rigid_body(
    start_body: &boxcars::RigidBody,
    start_time: f32,
    end_body: &boxcars::RigidBody,
    end_time: f32,
    time: f32,
) -> SubtrActorResult<boxcars::RigidBody> {
    if !(start_time <= time && time <= end_time) {
        return SubtrActorError::new_result(SubtrActorErrorVariant::InterpolationTimeOrderError {
            start_time,
            time,
            end_time,
        });
    }

    let duration = end_time - start_time;
    let interpolation_amount = (time - start_time) / duration;
    let start_position = util::vec_to_glam(&start_body.location);
    let end_position = util::vec_to_glam(&end_body.location);
    let interpolated_location = start_position.lerp(end_position, interpolation_amount);
    let start_rotation = quat_to_glam(&start_body.rotation);
    let end_rotation = quat_to_glam(&end_body.rotation);
    let interpolated_rotation = start_rotation.slerp(end_rotation, interpolation_amount);

    Ok(boxcars::RigidBody {
        location: glam_to_vec(&interpolated_location),
        rotation: glam_to_quat(&interpolated_rotation),
        sleeping: start_body.sleeping,
        linear_velocity: start_body.linear_velocity,
        angular_velocity: start_body.angular_velocity,
    })
}

/// Enum to define the direction of searching within a collection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// Searches for an item in a slice in a specified direction and returns the
/// first item that matches the provided predicate.
///
/// # Arguments
///
/// * `items` - The list of items to search.
/// * `current_index` - The index to start the search from.
/// * `direction` - The direction to search in.
/// * `predicate` - A function that takes an item and returns an [`Option<R>`].
///   When this function returns `Some(R)`, the item is considered a match.
///
/// # Returns
///
/// Returns a tuple of the index and the result `R` of the predicate for the first item that matches.
pub fn find_in_direction<T, F, R>(
    items: &[T],
    current_index: usize,
    direction: SearchDirection,
    predicate: F,
) -> Option<(usize, R)>
where
    F: Fn(&T) -> Option<R>,
{
    let mut iter: Box<dyn Iterator<Item = (usize, &T)>> = match direction {
        SearchDirection::Forward => Box::new(
            items[current_index + 1..]
                .iter()
                .enumerate()
                .map(move |(i, item)| (i + current_index + 1, item)),
        ),
        SearchDirection::Backward => Box::new(items[..current_index].iter().enumerate().rev()),
    };

    iter.find_map(|(i, item)| predicate(item).map(|res| (i, res)))
}
