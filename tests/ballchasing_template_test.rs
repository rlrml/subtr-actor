#![allow(unused_macros)]

macro_rules! ballchasing_fixture_test {
    ($test_name:ident, $fixture_dir:literal) => {
        #[test]
        #[ignore = "Ballchasing fixtures are opt-in and should be enabled fixture-by-fixture"]
        fn $test_name() {
            let report = subtr_actor::ballchasing::compare_fixture_directory(
                std::path::Path::new(concat!("assets/ballchasing-fixtures/", $fixture_dir)),
                &subtr_actor::ballchasing::recommended_match_config(),
            )
            .expect("Failed to compare Ballchasing fixture");
            report.assert_matches();
        }
    };
}

ballchasing_fixture_test!(
    compare_recent_ranked_doubles_2026_03_10,
    "recent-ranked-doubles-2026-03-10"
);

ballchasing_fixture_test!(
    compare_recent_ranked_standard_2026_03_10_a,
    "recent-ranked-standard-2026-03-10-a"
);

ballchasing_fixture_test!(
    compare_recent_ranked_standard_2026_03_10_b,
    "recent-ranked-standard-2026-03-10-b"
);
