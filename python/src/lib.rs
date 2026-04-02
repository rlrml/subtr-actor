#![allow(clippy::useless_conversion)]

use numpy::IntoPyArray;
use pyo3::prelude::*;
use pyo3::*;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use subtr_actor::*;

#[allow(clippy::useless_conversion)]
#[pyfunction]
fn parse_replay<'p>(py: Python<'p>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let replay = serde_json::to_value(replay_from_data(data)?).map_err(to_py_error)?;
    Ok(convert_to_py(py, &replay))
}

fn replay_from_data(data: &[u8]) -> PyResult<boxcars::Replay> {
    boxcars::ParserBuilder::new(data)
        .must_parse_network_data()
        .on_error_check_crc()
        .parse()
        .map_err(to_py_error)
}

#[pymodule]
#[pyo3(name = "subtr_actor")]
fn subtr_actor_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_replay, m)?)?;
    m.add_function(wrap_pyfunction!(
        get_ndarray_with_info_from_replay_filepath,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(get_replay_meta, m)?)?;
    m.add_function(wrap_pyfunction!(get_column_headers, m)?)?;
    m.add_function(wrap_pyfunction!(get_replay_frames_data, m)?)?;
    m.add_function(wrap_pyfunction!(get_stats_timeline, m)?)?;
    m.add_function(wrap_pyfunction!(get_dynamic_stats_timeline, m)?)?;
    Ok(())
}

fn to_py_error<E: std::error::Error>(e: E) -> PyErr {
    PyErr::new::<exceptions::PyException, _>(format!("{e}"))
}

fn handle_frames_exception(e: subtr_actor::SubtrActorError) -> PyErr {
    PyErr::new::<exceptions::PyException, _>(format!("{:?} {}", e.variant, e.backtrace))
}

fn convert_to_py(py: Python, value: &Value) -> Py<PyAny> {
    match value {
        Value::Null => py.None(),
        Value::Bool(b) => b.into_pyobject(py).unwrap().into_any().unbind(),
        Value::Number(n) => match n {
            n if n.is_u64() => n
                .as_u64()
                .unwrap()
                .into_pyobject(py)
                .unwrap()
                .into_any()
                .unbind(),
            n if n.is_i64() => n
                .as_i64()
                .unwrap()
                .into_pyobject(py)
                .unwrap()
                .into_any()
                .unbind(),
            n if n.is_f64() => n
                .as_f64()
                .unwrap()
                .into_pyobject(py)
                .unwrap()
                .into_any()
                .unbind(),
            _ => py.None(),
        },
        Value::String(s) => s.into_pyobject(py).unwrap().into_any().unbind(),
        Value::Array(list) => {
            let list: Vec<Py<PyAny>> = list.iter().map(|e| convert_to_py(py, e)).collect();
            list.into_pyobject(py).unwrap().into_any().unbind()
        }
        Value::Object(m) => {
            let mut map = BTreeMap::new();
            m.iter().for_each(|(k, v)| {
                map.insert(k, convert_to_py(py, v));
            });
            map.into_pyobject(py).unwrap().into_any().unbind()
        }
    }
}

static DEFAULT_GLOBAL_FEATURE_ADDERS: [&str; 1] = ["BallRigidBody"];

static DEFAULT_PLAYER_FEATURE_ADDERS: [&str; 3] =
    ["PlayerRigidBody", "PlayerBoost", "PlayerAnyJump"];

enum NDArrayDType {
    Float16,
    Float32,
    Float64,
}

fn parse_ndarray_dtype(dtype: Option<String>) -> PyResult<NDArrayDType> {
    match dtype {
        None => Ok(NDArrayDType::Float32),
        Some(dtype) => match dtype.trim().to_ascii_lowercase().as_str() {
            "f16" | "float16" | "half" => Ok(NDArrayDType::Float16),
            "f32" | "float32" => Ok(NDArrayDType::Float32),
            "f64" | "float64" | "double" => Ok(NDArrayDType::Float64),
            invalid => Err(PyErr::new::<exceptions::PyValueError, _>(format!(
                "Unsupported dtype '{invalid}'. Expected one of: f16, float16, half, f32, float32, f64, float64, double"
            ))),
        },
    }
}

fn get_ndarray_with_info_for_type<'p, F>(
    py: Python<'p>,
    replay: &boxcars::Replay,
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
    fps: Option<f32>,
) -> PyResult<(Py<PyAny>, Py<PyAny>)>
where
    F: TryFrom<f32> + Send + Sync + 'static + numpy::Element,
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    let mut collector = build_ndarray_collector::<F>(global_feature_adders, player_feature_adders)
        .map_err(handle_frames_exception)?;

    FrameRateDecorator::new_from_fps(fps.unwrap_or(10.0), &mut collector)
        .process_replay(replay)
        .map_err(handle_frames_exception)?;

    let (replay_meta_with_headers, rust_nd_array) = collector
        .get_meta_and_ndarray()
        .map_err(handle_frames_exception)?;

    let python_replay_meta = convert_to_py(
        py,
        &serde_json::to_value(&replay_meta_with_headers).map_err(to_py_error)?,
    );
    let python_nd_array = rust_nd_array.into_pyarray(py).into_any().unbind();

    Ok((python_replay_meta, python_nd_array))
}

/// Convert a replay file to a `numpy` ndarray with additional metadata in Python.
///
/// This function takes a replay file path, reads the file and processes it. It
/// constructs an ndarray with global and player features and collects metadata
/// about the replay. The constructed ndarray and metadata are then converted
/// into Python objects and returned.
///
/// The replay file processing can optionally be run at a different
/// frames-per-second (fps) rate. By default, it processes replays at 10 fps.
///
/// # Arguments
///
/// * `py`: A Python interpreter instance.
/// * `filepath`: A path to the replay file.
/// * `global_feature_adders`: An optional vector of global feature adders. Each
/// adder is a string that represents a feature to be added to the global
/// features ndarray.
/// * `player_feature_adders`: An optional vector of player feature adders. Each
/// adder is a string that represents a feature to be added to the player
/// features ndarray.
/// * `fps`: An optional float representing the frames-per-second to use when
/// processing the replay. Default is 10.0 fps.
///
/// Refer to the [struct definitions provided in the ndarray
/// collector](https://docs.rs/subtr-actor/latest/subtr_actor/collector/ndarray/index.html)
/// documentation for valid string names to use within the global_feature_adders
/// and player_feature_adders arguments. These strings correspond to the
/// features that will be added to the global and player ndarrays respectively.
///
///
/// # Returns
///
/// * A Python tuple containing metadata about the replay and the ndarray of
/// features. If there was an error reading the file or processing the replay,
/// this will be an Err variant with the Python error.
#[allow(clippy::useless_conversion)]
#[pyfunction]
#[pyo3(signature = (filepath, global_feature_adders=None, player_feature_adders=None, fps=None, dtype=None))]
fn get_ndarray_with_info_from_replay_filepath<'p>(
    py: Python<'p>,
    filepath: PathBuf,
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
    fps: Option<f32>,
    dtype: Option<String>,
) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(filepath.as_path()).map_err(to_py_error)?;
    let replay = replay_from_data(&data)?;

    let (python_replay_meta, python_nd_array) = match parse_ndarray_dtype(dtype)? {
        NDArrayDType::Float16 => {
            let (python_replay_meta, python_nd_array) = get_ndarray_with_info_for_type::<f32>(
                py,
                &replay,
                global_feature_adders,
                player_feature_adders,
                fps,
            )?;
            let np = py.import("numpy")?;
            let float16 = np.getattr("float16")?;
            let casted_nd_array = python_nd_array
                .bind(py)
                .call_method1("astype", (float16,))?
                .unbind();
            Ok((python_replay_meta, casted_nd_array))
        }
        NDArrayDType::Float32 => get_ndarray_with_info_for_type::<f32>(
            py,
            &replay,
            global_feature_adders,
            player_feature_adders,
            fps,
        ),
        NDArrayDType::Float64 => get_ndarray_with_info_for_type::<f64>(
            py,
            &replay,
            global_feature_adders,
            player_feature_adders,
            fps,
        ),
    }?;

    Ok((python_replay_meta, python_nd_array)
        .into_pyobject(py)?
        .into_any()
        .unbind())
}

#[allow(clippy::result_large_err)]
fn build_ndarray_collector<F>(
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
) -> subtr_actor::SubtrActorResult<subtr_actor::NDArrayCollector<F>>
where
    F: TryFrom<f32> + Send + Sync + 'static,
    <F as TryFrom<f32>>::Error: std::fmt::Debug,
{
    let global_feature_adders = global_feature_adders.unwrap_or_else(|| {
        DEFAULT_GLOBAL_FEATURE_ADDERS
            .iter()
            .map(|i| i.to_string())
            .collect()
    });
    let player_feature_adders = player_feature_adders.unwrap_or_else(|| {
        DEFAULT_PLAYER_FEATURE_ADDERS
            .iter()
            .map(|i| i.to_string())
            .collect()
    });
    let global_feature_adders: Vec<&str> = global_feature_adders.iter().map(|s| &s[..]).collect();
    let player_feature_adders: Vec<&str> = player_feature_adders.iter().map(|s| &s[..]).collect();
    subtr_actor::NDArrayCollector::<F>::from_strings_typed(
        &global_feature_adders,
        &player_feature_adders,
    )
}

#[allow(clippy::useless_conversion)]
#[pyfunction]
#[pyo3(signature = (filepath, global_feature_adders=None, player_feature_adders=None))]
fn get_replay_meta<'p>(
    py: Python<'p>,
    filepath: PathBuf,
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(filepath.as_path()).map_err(to_py_error)?;
    let replay = replay_from_data(&data)?;

    let mut collector =
        build_ndarray_collector::<f32>(global_feature_adders, player_feature_adders)
            .map_err(handle_frames_exception)?;

    let replay_meta = collector
        .process_and_get_meta_and_headers(&replay)
        .map_err(handle_frames_exception)?;

    Ok(convert_to_py(
        py,
        &serde_json::to_value(&replay_meta).map_err(to_py_error)?,
    ))
}

#[allow(clippy::useless_conversion)]
#[pyfunction]
#[pyo3(signature = (global_feature_adders=None, player_feature_adders=None))]
fn get_column_headers<'p>(
    py: Python<'p>,
    global_feature_adders: Option<Vec<String>>,
    player_feature_adders: Option<Vec<String>>,
) -> PyResult<Py<PyAny>> {
    let header_info = build_ndarray_collector::<f32>(global_feature_adders, player_feature_adders)
        .map_err(handle_frames_exception)?
        .get_column_headers();
    Ok(convert_to_py(
        py,
        &serde_json::to_value(&header_info).map_err(to_py_error)?,
    ))
}

#[allow(clippy::useless_conversion)]
#[pyfunction]
fn get_replay_frames_data<'p>(py: Python<'p>, filepath: PathBuf) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(filepath.as_path()).map_err(to_py_error)?;
    let replay = replay_from_data(&data)?;

    let mut processor = ReplayProcessor::new(&replay).map_err(handle_frames_exception)?;
    let mut replay_data_collector = ReplayDataCollector::new();
    let mut flip_reset_tracker = FlipResetTracker::new();
    let mut boost_pad_collector = ReducerCollector::new(BoostReducer::new());

    processor
        .process_all(&mut [
            &mut replay_data_collector,
            &mut flip_reset_tracker,
            &mut boost_pad_collector,
        ])
        .map_err(handle_frames_exception)?;

    let supplemental_data = ReplayDataSupplementalData::from_flip_reset_tracker(flip_reset_tracker)
        .with_boost_pads(boost_pad_collector.into_inner().resolved_boost_pads());

    let replay_data = replay_data_collector
        .into_replay_data_with_supplemental_data(processor, supplemental_data)
        .map_err(handle_frames_exception)?;

    Ok(convert_to_py(
        py,
        &serde_json::to_value(replay_data).map_err(to_py_error)?,
    ))
}

#[allow(clippy::useless_conversion)]
#[pyfunction]
#[pyo3(signature = (filepath))]
fn get_stats_timeline<'p>(py: Python<'p>, filepath: PathBuf) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(filepath.as_path()).map_err(to_py_error)?;
    let replay = replay_from_data(&data)?;
    let timeline = subtr_actor::StatsCollector::new()
        .get_stats_timeline_value(&replay)
        .map_err(handle_frames_exception)?;

    Ok(convert_to_py(py, &timeline))
}

#[allow(clippy::useless_conversion)]
#[pyfunction]
#[pyo3(signature = (filepath))]
fn get_dynamic_stats_timeline<'p>(py: Python<'p>, filepath: PathBuf) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(filepath.as_path()).map_err(to_py_error)?;
    let replay = replay_from_data(&data)?;
    let timeline = subtr_actor::StatsCollector::new()
        .get_dynamic_replay_stats_timeline(&replay)
        .map_err(handle_frames_exception)?;

    Ok(convert_to_py(
        py,
        &serde_json::to_value(timeline).map_err(to_py_error)?,
    ))
}
