use subtr_actor::*;

struct BoostUnitVerifier {
    sample_interval: usize,
    samples_checked: usize,
}

impl BoostUnitVerifier {
    fn new(sample_interval: usize) -> Self {
        Self {
            sample_interval,
            samples_checked: 0,
        }
    }
}

impl Collector for BoostUnitVerifier {
    fn process_frame(
        &mut self,
        processor: &ReplayProcessor,
        _frame: &boxcars::Frame,
        frame_number: usize,
        _current_time: f32,
    ) -> SubtrActorResult<TimeAdvance> {
        if frame_number.is_multiple_of(self.sample_interval) {
            for player_id in processor.iter_player_ids_in_order() {
                let raw_boost = match processor.get_player_boost_level(player_id) {
                    Ok(raw_boost) => raw_boost,
                    Err(_) => continue,
                };
                let boost_percent = processor.get_player_boost_percentage(player_id)?;
                let converted_percent = boost_amount_to_percent(raw_boost);

                assert!(
                    (boost_percent - converted_percent).abs() < 1e-4,
                    "percent conversion mismatch for player {player_id:?}: raw={raw_boost}, percent={boost_percent}, converted={converted_percent}",
                );
                assert!(
                    (0.0..=BOOST_MAX_AMOUNT).contains(&raw_boost),
                    "raw boost out of range for player {player_id:?}: {raw_boost}",
                );
                assert!(
                    (0.0..=100.0).contains(&boost_percent),
                    "boost percent out of range for player {player_id:?}: {boost_percent}",
                );
                self.samples_checked += 1;
            }
        }

        Ok(TimeAdvance::NextFrame)
    }
}

fn verify_replay_boost_units(replay_path: &str) {
    let data = std::fs::read(replay_path).expect("Failed to read replay file");
    let replay = boxcars::ParserBuilder::new(&data[..])
        .always_check_crc()
        .must_parse_network_data()
        .parse()
        .expect("Failed to parse replay");

    let verifier = BoostUnitVerifier::new(50)
        .process_replay(&replay)
        .expect("Failed to process replay");

    assert!(
        verifier.samples_checked > 0,
        "expected to verify at least one boost sample",
    );
}

#[test]
fn test_boost_unit_conversion_helpers() {
    assert_eq!(BOOST_MAX_AMOUNT, 255.0);
    assert!((boost_amount_to_percent(BOOST_MAX_AMOUNT) - 100.0).abs() < 1e-6);
    assert!((boost_percent_to_amount(100.0) - BOOST_MAX_AMOUNT).abs() < 1e-6);
    assert!((boost_amount_to_percent(boost_percent_to_amount(33.3)) - 33.3).abs() < 1e-4);
    assert!(
        (BOOST_USED_PERCENT_PER_SECOND - boost_amount_to_percent(BOOST_USED_RAW_UNITS_PER_SECOND))
            .abs()
            < 1e-6
    );
}

#[test]
fn test_boost_percentage_helper_new_replay() {
    verify_replay_boost_units("assets/replays/new_boost_format.replay");
}

#[test]
fn test_boost_percentage_helper_old_replay() {
    verify_replay_boost_units("assets/replays/old_boost_format.replay");
}
