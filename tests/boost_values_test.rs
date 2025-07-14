use std::collections::HashMap;
use subtr_actor::*;

/// A custom collector that tracks boost values over time
struct BoostTracker {
    boost_values_per_player: HashMap<PlayerId, Vec<f32>>,
    sample_interval: usize,
}

impl BoostTracker {
    fn new(sample_interval: usize) -> Self {
        Self {
            boost_values_per_player: HashMap::new(),
            sample_interval,
        }
    }

    fn get_results(self) -> HashMap<PlayerId, Vec<f32>> {
        self.boost_values_per_player
    }
}

impl Collector for BoostTracker {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        frame_number: usize,
        _current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        // Sample at specified intervals
        if frame_number % self.sample_interval == 0 {
            // Get all player IDs by checking the processor's internal mappings
            for player_id in processor.iter_player_ids_in_order() {
                match processor.get_player_boost_level(player_id) {
                    Ok(boost_level) => {
                        self.boost_values_per_player
                            .entry(player_id.clone())
                            .or_default()
                            .push(boost_level);
                    }
                    Err(_) => {
                        // Player might not have boost data available at this frame
                    }
                }
            }
        }

        // Continue to next frame
        Ok(TimeAdvance::NextFrame)
    }
}

#[test]
fn test_boost_values_change_over_time_new_replay() {
    // Test with the new replay file to verify boost values change over time
    let replay_path = "assets/replays/new_boost_format.replay";
    let data = std::fs::read(replay_path).expect("Failed to read replay file");
    let replay = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .expect("Failed to parse replay");

    let tracker = BoostTracker::new(50); // Sample every 50 frames

    // Process the replay
    let tracker = tracker
        .process_replay(&replay)
        .expect("Failed to process replay");

    let boost_values_per_player = tracker.get_results();

    // Analyze the boost values
    let mut total_unique_values = 0;
    for boost_values in boost_values_per_player.values() {
        if !boost_values.is_empty() {
            let unique_values: std::collections::HashSet<_> =
                boost_values.iter().map(|&f| (f * 100.0) as i32).collect();
            total_unique_values += unique_values.len();
        }
    }

    // Check that we have some players with data
    assert!(
        !boost_values_per_player.is_empty(),
        "No player boost data collected"
    );

    // Boost values should change over time in any real replay
    assert!(
        total_unique_values > boost_values_per_player.len() * 10,
        "Boost values should change significantly over time. \
         Got {} unique values across {} players (expected > {}). \
         This indicates boost parsing is not working for newer replay formats.",
        total_unique_values,
        boost_values_per_player.len(),
        boost_values_per_player.len() * 10
    );
}

#[test]
fn test_boost_values_change_over_time_old_replay() {
    // Test with an older replay file that should work
    let replay_path = "assets/replays/old_boost_format.replay";
    let data = std::fs::read(replay_path).expect("Failed to read test replay file");
    let replay = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .expect("Failed to parse test replay");

    let tracker = BoostTracker::new(50); // Sample every 50 frames

    // Process the replay
    let tracker = tracker
        .process_replay(&replay)
        .expect("Failed to process replay");

    let boost_values_per_player = tracker.get_results();

    // Analyze boost values for old replay
    let mut total_unique_values = 0;
    for boost_values in boost_values_per_player.values() {
        if !boost_values.is_empty() {
            let unique_values: std::collections::HashSet<_> =
                boost_values.iter().map(|&f| (f * 100.0) as i32).collect();
            total_unique_values += unique_values.len();
        }
    }

    // Check that we have some players with data
    assert!(
        !boost_values_per_player.is_empty(),
        "Should have collected some boost data"
    );

    // For the old replay format, boost values should change significantly over time
    assert!(
        total_unique_values > boost_values_per_player.len() * 10,
        "Expected boost values to change significantly in old replay format. \
         Got {} unique values across {} players (expected > {})",
        total_unique_values,
        boost_values_per_player.len(),
        boost_values_per_player.len() * 10
    );
}
