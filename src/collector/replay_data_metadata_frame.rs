use super::*;

/// Represents game metadata for a single frame in a Rocket League replay.
///
/// Contains timing information and game state data that applies to the entire
/// game at a specific point in time.
///
/// # Fields
///
/// * `time` - The current time in seconds since the start of the replay
/// * `seconds_remaining` - The number of seconds remaining in the current game period
/// * `replicated_game_state_name` - The game state enum value (indicates countdown, playing, goal, etc.)
/// * `replicated_game_state_time_remaining` - The kickoff countdown timer, usually 3 to 0
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct MetadataFrame {
    /// The current time in seconds since the start of the replay
    pub time: f32,
    /// The number of seconds remaining in the current game period
    pub seconds_remaining: i32,
    /// The game state enum value (indicates countdown, playing, goal scored, etc.)
    pub replicated_game_state_name: i32,
    /// The kickoff countdown timer exposed by the replay metadata actor.
    pub replicated_game_state_time_remaining: i32,
}

impl MetadataFrame {
    /// Creates a new [`MetadataFrame`] from a [`ReplayProcessor`] at the specified time.
    ///
    /// # Arguments
    ///
    /// * `processor` - The [`ReplayProcessor`] containing the replay data
    /// * `time` - The current time in seconds since the start of the replay
    ///
    /// # Returns
    ///
    /// Returns a [`SubtrActorResult`] containing a [`MetadataFrame`] with the
    /// current time and remaining seconds extracted from the processor.
    ///
    /// # Errors
    ///
    /// Returns a [`SubtrActorError`] if the seconds remaining cannot be retrieved
    /// from the processor.
    pub(super) fn new_from_processor(
        processor: &dyn ProcessorView,
        time: f32,
    ) -> SubtrActorResult<Self> {
        Ok(Self::new(
            time,
            processor.get_seconds_remaining()?,
            processor.get_replicated_state_name().unwrap_or(0),
            processor
                .get_replicated_game_state_time_remaining()
                .unwrap_or(0),
        ))
    }

    /// Creates a new [`MetadataFrame`] with the specified time, seconds remaining, game state,
    /// and kickoff countdown value.
    ///
    /// # Arguments
    ///
    /// * `time` - The current time in seconds since the start of the replay
    /// * `seconds_remaining` - The number of seconds remaining in the current game period
    /// * `replicated_game_state_name` - The game state enum value
    /// * `replicated_game_state_time_remaining` - The kickoff countdown timer
    ///
    /// # Returns
    ///
    /// Returns a new [`MetadataFrame`] with the provided values.
    fn new(
        time: f32,
        seconds_remaining: i32,
        replicated_game_state_name: i32,
        replicated_game_state_time_remaining: i32,
    ) -> Self {
        MetadataFrame {
            time,
            seconds_remaining,
            replicated_game_state_name,
            replicated_game_state_time_remaining,
        }
    }
}
