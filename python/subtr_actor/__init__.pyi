import os
from collections.abc import Sequence
from typing import Any, Literal

import numpy as np
import numpy.typing as npt
from typing_extensions import TypeAlias

_Path: TypeAlias = str | os.PathLike[str]
_JsonDict: TypeAlias = dict[str, Any]
_DType: TypeAlias = Literal[
    "f16", "float16", "half", "f32", "float32", "f64", "float64", "double"
]

def parse_replay(data: bytes) -> _JsonDict: ...
def get_ndarray_with_info_from_replay_filepath(
    filepath: _Path,
    global_feature_adders: Sequence[str] | None = None,
    player_feature_adders: Sequence[str] | None = None,
    fps: float | None = None,
    dtype: _DType | None = None,
) -> tuple[_JsonDict, npt.NDArray[np.floating[Any]]]: ...
def get_replay_meta(
    filepath: _Path,
    global_feature_adders: Sequence[str] | None = None,
    player_feature_adders: Sequence[str] | None = None,
) -> _JsonDict: ...
def get_column_headers(
    global_feature_adders: Sequence[str] | None = None,
    player_feature_adders: Sequence[str] | None = None,
) -> _JsonDict: ...
def get_replay_frames_data(filepath: _Path) -> _JsonDict: ...
def get_stats_module_names() -> list[str]: ...
def get_stats_events(
    filepath: _Path,
    frame_step_seconds: float | None = None,
    include_expected_goals: bool = False,
) -> _JsonDict: ...
def get_summed_stats(
    filepath: _Path,
    module_names: Sequence[str] | None = None,
) -> _JsonDict: ...
def get_stats_snapshot_data(
    filepath: _Path,
    module_names: Sequence[str] | None = None,
    frame_step_seconds: float | None = None,
) -> _JsonDict: ...
def get_stats_timeline(
    filepath: _Path,
    module_names: None = None,
    frame_step_seconds: float | None = None,
    include_expected_goals: bool = False,
) -> _JsonDict: ...
def get_legacy_stats_timeline(
    filepath: _Path,
    module_names: Sequence[str] | None = None,
    frame_step_seconds: float | None = None,
) -> _JsonDict: ...
