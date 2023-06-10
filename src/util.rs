use boxcars::{HeaderProp, RemoteId};
use serde::Serialize;

use crate::*;

#[macro_export]
macro_rules! fmt_err {
    ($( $item:expr ),* $(,)?) => {
        Err(format!($( $item ),*))
    };
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReplayMeta {
    pub team_zero: Vec<PlayerInfo>,
    pub team_one: Vec<PlayerInfo>,
    pub all_headers: Vec<(String, HeaderProp)>,
}

impl ReplayMeta {
    pub fn player_count(&self) -> usize {
        self.team_one.len() + self.team_zero.len()
    }

    pub fn player_order(&self) -> impl Iterator<Item = &PlayerInfo> {
        self.team_zero.iter().chain(self.team_one.iter())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerInfo {
    pub remote_id: RemoteId,
    pub stats: Option<std::collections::HashMap<String, HeaderProp>>,
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
            "Player not found {:?} {:?}",
            player_id, all_player_stats
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

fn name_matches(name: &String, props: &Vec<(String, HeaderProp)>) -> bool {
    if let Ok((_, HeaderProp::Str(stat_name))) = get_prop("Name", props) {
        *name == stat_name
    } else {
        false
    }
}

fn online_id_matches(id: u64, props: &Vec<(String, HeaderProp)>) -> bool {
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

fn get_prop(prop: &str, props: &Vec<(String, HeaderProp)>) -> Result<(String, HeaderProp), String> {
    props
        .iter()
        .find(|(attr, _)| attr == prop)
        .ok_or("Coudn't find name property".to_string())
        .cloned()
}

pub trait VecMapEntry<K: PartialEq, V> {
    fn get_entry(&mut self, key: K) -> Entry<K, V>;
}

pub enum Entry<'a, K: PartialEq, V> {
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

pub struct OccupiedEntry<'a, K: PartialEq, V> {
    entry: &'a mut (K, V),
}

pub struct VacantEntry<'a, K: PartialEq, V> {
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

fn vec_to_glam(v: &boxcars::Vector3f) -> glam::f32::Vec3 {
    glam::f32::Vec3::new(v.x, v.y, v.z)
}

fn glam_to_vec(v: &glam::f32::Vec3) -> boxcars::Vector3f {
    boxcars::Vector3f {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

pub fn apply_velocities_to_rigid_body(
    rigid_body: &boxcars::RigidBody,
    time_delta: f32,
) -> boxcars::RigidBody {
    let mut interpolated = rigid_body.clone();
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
    let rbav = rigid_body
        .angular_velocity
        .unwrap_or_else(|| boxcars::Vector3f {
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

pub fn ball_rigid_body_exists(
    processor: &ReplayProcessor,
    _f: &boxcars::Frame,
    _: usize,
) -> Result<bool, String> {
    Ok(processor
        .get_ball_rigid_body()
        .map(|rb| !rb.sleeping)
        .unwrap_or(false))
}
