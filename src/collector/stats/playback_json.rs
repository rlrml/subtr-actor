use boxcars::{Ps4Id, PsyNetId, RemoteId, SwitchId};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::*;

use super::serialize_to_json_value;

#[path = "playback_json_core.rs"]
mod playback_json_core;
#[path = "playback_json_events.rs"]
mod playback_json_events;
#[path = "playback_json_values.rs"]
mod playback_json_values;

pub(in crate::collector::stats::playback) use playback_json_core::*;
pub(in crate::collector::stats::playback) use playback_json_events::*;
pub(in crate::collector::stats::playback) use playback_json_values::*;
