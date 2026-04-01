use std::env;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use subtr_actor::*;

#[derive(Default)]
struct NoopReducer;

impl StatsReducer for NoopReducer {}

#[derive(Clone)]
struct SignalProbeReducer {
    signals: Vec<DerivedSignalId>,
}

impl SignalProbeReducer {
    fn new(signals: &[DerivedSignalId]) -> Self {
        Self {
            signals: signals.to_vec(),
        }
    }
}

impl StatsReducer for SignalProbeReducer {
    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        self.signals.clone()
    }
}

#[derive(Clone, Copy)]
enum Scenario {
    ReducerNoop,
    SignalTouch,
    SignalPossession,
    SignalBackboard,
    SignalFiftyFifty,
    MatchStatsReducer,
    BackboardReducer,
    CeilingShotReducer,
    DoubleTapReducer,
    FiftyFiftyReducer,
    PossessionReducer,
    PressureReducer,
    RushReducer,
    TouchReducer,
    SpeedFlipReducer,
    MustyFlickReducer,
    DodgeResetReducer,
    BallCarryReducer,
    BoostReducer,
    MovementReducer,
    PositioningReducer,
    PowerslideReducer,
    DemoReducer,
    ComparableBundle,
    StatsTimelineEmptyTyped,
    StatsTimelineEmptyDynamic,
    StatsTimelineFullTyped,
    StatsTimelineFullDynamic,
    StatsTimelineFullDynamicValue,
    StatsCollectorPlayback,
    StatsCollectorFullTyped,
    StatsCollectorFullDynamic,
    StatsCollectorFullDynamicValue,
}

impl Scenario {
    const ALL: [Scenario; 33] = [
        Scenario::ReducerNoop,
        Scenario::SignalTouch,
        Scenario::SignalPossession,
        Scenario::SignalBackboard,
        Scenario::SignalFiftyFifty,
        Scenario::MatchStatsReducer,
        Scenario::BackboardReducer,
        Scenario::CeilingShotReducer,
        Scenario::DoubleTapReducer,
        Scenario::FiftyFiftyReducer,
        Scenario::PossessionReducer,
        Scenario::PressureReducer,
        Scenario::RushReducer,
        Scenario::TouchReducer,
        Scenario::SpeedFlipReducer,
        Scenario::MustyFlickReducer,
        Scenario::DodgeResetReducer,
        Scenario::BallCarryReducer,
        Scenario::BoostReducer,
        Scenario::MovementReducer,
        Scenario::PositioningReducer,
        Scenario::PowerslideReducer,
        Scenario::DemoReducer,
        Scenario::ComparableBundle,
        Scenario::StatsTimelineEmptyTyped,
        Scenario::StatsTimelineEmptyDynamic,
        Scenario::StatsTimelineFullTyped,
        Scenario::StatsTimelineFullDynamic,
        Scenario::StatsTimelineFullDynamicValue,
        Scenario::StatsCollectorPlayback,
        Scenario::StatsCollectorFullTyped,
        Scenario::StatsCollectorFullDynamic,
        Scenario::StatsCollectorFullDynamicValue,
    ];

    fn name(self) -> &'static str {
        match self {
            Scenario::ReducerNoop => "reducer_noop",
            Scenario::SignalTouch => "signals_touch_only",
            Scenario::SignalPossession => "signals_possession_only",
            Scenario::SignalBackboard => "signals_backboard_only",
            Scenario::SignalFiftyFifty => "signals_fifty_fifty_only",
            Scenario::MatchStatsReducer => "match_stats_reducer",
            Scenario::BackboardReducer => "backboard_reducer",
            Scenario::CeilingShotReducer => "ceiling_shot_reducer",
            Scenario::DoubleTapReducer => "double_tap_reducer",
            Scenario::FiftyFiftyReducer => "fifty_fifty_reducer",
            Scenario::PossessionReducer => "possession_reducer",
            Scenario::PressureReducer => "pressure_reducer",
            Scenario::RushReducer => "rush_reducer",
            Scenario::TouchReducer => "touch_reducer",
            Scenario::SpeedFlipReducer => "speed_flip_reducer",
            Scenario::MustyFlickReducer => "musty_flick_reducer",
            Scenario::DodgeResetReducer => "dodge_reset_reducer",
            Scenario::BallCarryReducer => "ball_carry_reducer",
            Scenario::BoostReducer => "boost_reducer",
            Scenario::MovementReducer => "movement_reducer",
            Scenario::PositioningReducer => "positioning_reducer",
            Scenario::PowerslideReducer => "powerslide_reducer",
            Scenario::DemoReducer => "demo_reducer",
            Scenario::ComparableBundle => "comparable_bundle",
            Scenario::StatsTimelineEmptyTyped => "stats_timeline_empty_typed",
            Scenario::StatsTimelineEmptyDynamic => "stats_timeline_empty_dynamic",
            Scenario::StatsTimelineFullTyped => "stats_timeline_full_typed",
            Scenario::StatsTimelineFullDynamic => "stats_timeline_full_dynamic",
            Scenario::StatsTimelineFullDynamicValue => "stats_timeline_full_dynamic_value",
            Scenario::StatsCollectorPlayback => "stats_collector_playback",
            Scenario::StatsCollectorFullTyped => "stats_collector_full_typed",
            Scenario::StatsCollectorFullDynamic => "stats_collector_full_dynamic",
            Scenario::StatsCollectorFullDynamicValue => "stats_collector_full_dynamic_value",
        }
    }

    fn run(self, replay: &boxcars::Replay) -> SubtrActorResult<()> {
        match self {
            Scenario::ReducerNoop => {
                black_box(run_reducer(replay, NoopReducer)?);
            }
            Scenario::SignalTouch => {
                black_box(run_reducer(
                    replay,
                    SignalProbeReducer::new(&[TOUCH_STATE_SIGNAL_ID]),
                )?);
            }
            Scenario::SignalPossession => {
                black_box(run_reducer(
                    replay,
                    SignalProbeReducer::new(&[POSSESSION_STATE_SIGNAL_ID]),
                )?);
            }
            Scenario::SignalBackboard => {
                black_box(run_reducer(
                    replay,
                    SignalProbeReducer::new(&[BACKBOARD_BOUNCE_STATE_SIGNAL_ID]),
                )?);
            }
            Scenario::SignalFiftyFifty => {
                black_box(run_reducer(
                    replay,
                    SignalProbeReducer::new(&[FIFTY_FIFTY_STATE_SIGNAL_ID]),
                )?);
            }
            Scenario::MatchStatsReducer => {
                black_box(run_reducer(replay, MatchStatsReducer::new())?);
            }
            Scenario::BackboardReducer => {
                black_box(run_reducer(replay, BackboardReducer::new())?);
            }
            Scenario::CeilingShotReducer => {
                black_box(run_reducer(replay, CeilingShotReducer::new())?);
            }
            Scenario::DoubleTapReducer => {
                black_box(run_reducer(replay, DoubleTapReducer::new())?);
            }
            Scenario::FiftyFiftyReducer => {
                black_box(run_reducer(replay, FiftyFiftyReducer::new())?);
            }
            Scenario::PossessionReducer => {
                black_box(run_reducer(replay, PossessionReducer::new())?);
            }
            Scenario::PressureReducer => {
                black_box(run_reducer(replay, PressureReducer::new())?);
            }
            Scenario::RushReducer => {
                black_box(run_reducer(replay, RushReducer::new())?);
            }
            Scenario::TouchReducer => {
                black_box(run_reducer(replay, TouchReducer::new())?);
            }
            Scenario::SpeedFlipReducer => {
                black_box(run_reducer(replay, SpeedFlipReducer::new())?);
            }
            Scenario::MustyFlickReducer => {
                black_box(run_reducer(replay, MustyFlickReducer::new())?);
            }
            Scenario::DodgeResetReducer => {
                black_box(run_reducer(replay, DodgeResetReducer::new())?);
            }
            Scenario::BallCarryReducer => {
                black_box(run_reducer(replay, BallCarryReducer::new())?);
            }
            Scenario::BoostReducer => {
                black_box(run_reducer(replay, BoostReducer::new())?);
            }
            Scenario::MovementReducer => {
                black_box(run_reducer(replay, MovementReducer::new())?);
            }
            Scenario::PositioningReducer => {
                black_box(run_reducer(replay, PositioningReducer::new())?);
            }
            Scenario::PowerslideReducer => {
                black_box(run_reducer(replay, PowerslideReducer::new())?);
            }
            Scenario::DemoReducer => {
                black_box(run_reducer(replay, DemoReducer::new())?);
            }
            Scenario::ComparableBundle => run_comparable_bundle(replay)?,
            Scenario::StatsTimelineEmptyTyped => {
                black_box(
                    StatsTimelineCollector::only_modules(std::iter::empty::<&str>())
                        .get_replay_data(replay)?,
                );
            }
            Scenario::StatsTimelineEmptyDynamic => {
                black_box(
                    StatsTimelineCollector::only_modules(std::iter::empty::<&str>())
                        .get_dynamic_replay_data(replay)?,
                );
            }
            Scenario::StatsTimelineFullTyped => {
                black_box(StatsTimelineCollector::new().get_replay_data(replay)?);
            }
            Scenario::StatsTimelineFullDynamic => {
                black_box(StatsTimelineCollector::new().get_dynamic_replay_data(replay)?);
            }
            Scenario::StatsTimelineFullDynamicValue => {
                black_box(
                    serde_json::to_value(
                        &StatsTimelineCollector::new().get_dynamic_replay_data(replay)?,
                    )
                    .map_err(|error| {
                        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
                            "failed to serialize dynamic stats timeline: {error}",
                        )))
                    })?,
                );
            }
            Scenario::StatsCollectorPlayback => {
                black_box(StatsCollector::new().get_playback_data(replay)?);
            }
            Scenario::StatsCollectorFullTyped => {
                black_box(StatsCollector::new().get_replay_stats_timeline(replay)?);
            }
            Scenario::StatsCollectorFullDynamic => {
                black_box(StatsCollector::new().get_dynamic_replay_stats_timeline(replay)?);
            }
            Scenario::StatsCollectorFullDynamicValue => {
                black_box(StatsCollector::new().get_dynamic_stats_timeline_value(replay)?);
            }
        }

        Ok(())
    }
}

fn run_reducer<R: StatsReducer>(replay: &boxcars::Replay, reducer: R) -> SubtrActorResult<R> {
    let mut collector = ReducerCollector::new(reducer);
    let mut processor = ReplayProcessor::new(replay)?;
    processor.process(&mut collector)?;
    Ok(collector.into_inner())
}

fn run_comparable_bundle(replay: &boxcars::Replay) -> SubtrActorResult<()> {
    let mut match_collector = ReducerCollector::new(MatchStatsReducer::new());
    let mut boost_collector = ReducerCollector::new(BoostReducer::new());
    let mut movement_collector = ReducerCollector::new(MovementReducer::new());
    let mut positioning_collector = ReducerCollector::new(PositioningReducer::new());
    let mut demo_collector = ReducerCollector::new(DemoReducer::new());
    let mut powerslide_collector = ReducerCollector::new(PowerslideReducer::new());

    let mut processor = ReplayProcessor::new(replay)?;
    let mut collectors: [&mut dyn Collector; 6] = [
        &mut match_collector,
        &mut boost_collector,
        &mut movement_collector,
        &mut positioning_collector,
        &mut demo_collector,
        &mut powerslide_collector,
    ];
    processor.process_all(&mut collectors)?;

    black_box(match_collector.into_inner());
    black_box(boost_collector.into_inner());
    black_box(movement_collector.into_inner());
    black_box(positioning_collector.into_inner());
    black_box(demo_collector.into_inner());
    black_box(powerslide_collector.into_inner());
    Ok(())
}

#[derive(Clone)]
struct ReplayFixture {
    path: PathBuf,
    replay: boxcars::Replay,
    frame_count: usize,
    player_count: usize,
}

struct ScenarioResult {
    total_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,
}

impl ScenarioResult {
    fn average_duration(&self, iterations: usize) -> Duration {
        duration_div(self.total_duration, iterations as u32)
    }
}

#[derive(Default)]
struct ProfileOptions {
    iterations: usize,
    warmup: usize,
    scenario_filter: Option<String>,
    replay_filters: Vec<String>,
}

fn duration_div(duration: Duration, divisor: u32) -> Duration {
    if divisor == 0 {
        return Duration::ZERO;
    }
    Duration::from_secs_f64(duration.as_secs_f64() / divisor as f64)
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs_f64() >= 1.0 {
        format!("{:.3}s", duration.as_secs_f64())
    } else if duration.as_millis() > 0 {
        format!("{:.3}ms", duration.as_secs_f64() * 1_000.0)
    } else {
        format!("{:.3}us", duration.as_secs_f64() * 1_000_000.0)
    }
}

fn format_per_frame(duration: Duration, frame_count: usize) -> String {
    if frame_count == 0 {
        return "n/a".to_string();
    }
    format!(
        "{:.3}us/frame",
        duration.as_secs_f64() * 1_000_000.0 / frame_count as f64
    )
}

fn parse_args() -> ProfileOptions {
    let mut args = env::args().skip(1);
    let mut options = ProfileOptions {
        iterations: 2,
        warmup: 1,
        ..ProfileOptions::default()
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--iterations" | "-n" => {
                options.iterations = args
                    .next()
                    .expect("missing value for --iterations")
                    .parse()
                    .expect("invalid value for --iterations");
            }
            "--warmup" | "-w" => {
                options.warmup = args
                    .next()
                    .expect("missing value for --warmup")
                    .parse()
                    .expect("invalid value for --warmup");
            }
            "--scenario" | "-s" => {
                options.scenario_filter = Some(
                    args.next()
                        .expect("missing value for --scenario")
                        .to_ascii_lowercase(),
                );
            }
            other => options.replay_filters.push(other.to_string()),
        }
    }

    options
}

fn discover_replay_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_replays(Path::new("assets/replays"), &mut paths);
    collect_replays(Path::new("assets/ballchasing-fixtures"), &mut paths);
    paths.sort();
    paths
}

fn collect_replays(root: &Path, paths: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_replays(&path, paths);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "replay") {
            paths.push(path);
        }
    }
}

fn load_fixture(path: PathBuf) -> SubtrActorResult<ReplayFixture> {
    let bytes = fs::read(&path).map_err(|error| {
        SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
            "failed to read {}: {error}",
            path.display()
        )))
    })?;
    let replay = boxcars::ParserBuilder::new(&bytes)
        .must_parse_network_data()
        .always_check_crc()
        .parse()
        .map_err(|error| {
            SubtrActorError::new(SubtrActorErrorVariant::CallbackError(format!(
                "failed to parse {}: {error}",
                path.display()
            )))
        })?;

    let frame_count = replay
        .network_frames
        .as_ref()
        .map(|frames| frames.frames.len())
        .unwrap_or(0);
    let player_count = ReplayProcessor::new(&replay)
        .map(|processor| processor.team_zero.len() + processor.team_one.len())
        .unwrap_or(0);

    Ok(ReplayFixture {
        path,
        replay,
        frame_count,
        player_count,
    })
}

fn should_include_replay(path: &Path, replay_filters: &[String]) -> bool {
    if replay_filters.is_empty() {
        return true;
    }

    let haystack = path.display().to_string().to_ascii_lowercase();
    replay_filters
        .iter()
        .all(|filter| haystack.contains(&filter.to_ascii_lowercase()))
}

fn selected_scenarios(filter: &Option<String>) -> Vec<Scenario> {
    Scenario::ALL
        .into_iter()
        .filter(|scenario| {
            filter
                .as_ref()
                .is_none_or(|needle| scenario.name().contains(needle))
        })
        .collect()
}

fn measure_scenario(
    replay: &ReplayFixture,
    scenario: Scenario,
    iterations: usize,
    warmup: usize,
) -> SubtrActorResult<ScenarioResult> {
    for _ in 0..warmup {
        scenario.run(&replay.replay)?;
    }

    let mut total_duration = Duration::ZERO;
    let mut min_duration = Duration::MAX;
    let mut max_duration = Duration::ZERO;

    for _ in 0..iterations {
        let start = Instant::now();
        scenario.run(&replay.replay)?;
        let duration = start.elapsed();
        total_duration += duration;
        min_duration = min_duration.min(duration);
        max_duration = max_duration.max(duration);
    }

    Ok(ScenarioResult {
        total_duration,
        min_duration,
        max_duration,
    })
}

fn main() -> SubtrActorResult<()> {
    let options = parse_args();
    let scenarios = selected_scenarios(&options.scenario_filter);
    if scenarios.is_empty() {
        eprintln!("No scenarios matched the requested filter");
        return Ok(());
    }

    let replay_paths = discover_replay_paths();
    let selected_paths: Vec<_> = replay_paths
        .into_iter()
        .filter(|path| should_include_replay(path, &options.replay_filters))
        .collect();

    if selected_paths.is_empty() {
        eprintln!("No replay fixtures matched the provided filters");
        return Ok(());
    }

    let fixtures = selected_paths
        .into_iter()
        .map(load_fixture)
        .collect::<SubtrActorResult<Vec<_>>>()?;

    println!(
        "Profiling {} scenarios across {} replay fixtures (iterations={}, warmup={})",
        scenarios.len(),
        fixtures.len(),
        options.iterations,
        options.warmup
    );
    println!();

    for fixture in &fixtures {
        println!(
            "{} | frames={} players={}",
            fixture.path.display(),
            fixture.frame_count,
            fixture.player_count
        );
        let mut rows = Vec::new();
        for scenario in &scenarios {
            let result = measure_scenario(fixture, *scenario, options.iterations, options.warmup)?;
            rows.push((*scenario, result));
        }
        rows.sort_by_key(|(_, result)| {
            std::cmp::Reverse(result.average_duration(options.iterations))
        });
        for (scenario, result) in rows {
            let avg = result.average_duration(options.iterations);
            println!(
                "  {:28} avg {:>10}  min {:>10}  max {:>10}  {}",
                scenario.name(),
                format_duration(avg),
                format_duration(result.min_duration),
                format_duration(result.max_duration),
                format_per_frame(avg, fixture.frame_count),
            );
        }
        println!();
    }

    Ok(())
}
