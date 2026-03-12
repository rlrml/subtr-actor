from __future__ import annotations

import importlib
import importlib.util
import pathlib
import sys


def _load_native_module():
    try:
        return importlib.import_module(".subtr_actor", __name__)
    except ModuleNotFoundError as initial_error:
        package_dir = pathlib.Path(__file__).resolve().parent
        candidates = sorted(package_dir.glob("subtr_actor*.so")) + sorted(
            package_dir.glob("subtr_actor*.pyd")
        )
        load_errors: list[str] = []
        for candidate in candidates:
            spec = importlib.util.spec_from_file_location(
                f"{__name__}.subtr_actor", candidate
            )
            if spec is None or spec.loader is None:
                continue
            module = importlib.util.module_from_spec(spec)
            sys.modules[spec.name] = module
            try:
                spec.loader.exec_module(module)
                return module
            except Exception as error:  # pragma: no cover - exercised in packaging failures
                sys.modules.pop(spec.name, None)
                load_errors.append(f"{candidate.name}: {error}")
        load_error_text = "; ".join(load_errors) if load_errors else str(initial_error)
        raise ModuleNotFoundError(
            "Unable to import the packaged subtr_actor extension. "
            f"Looked in {package_dir} and found candidates: "
            f"{[path.name for path in candidates]}. Errors: {load_error_text}"
        ) from initial_error


_native = _load_native_module()
_exports = getattr(_native, "__all__", None)
if _exports is None:
    _exports = [name for name in dir(_native) if not name.startswith("_")]

globals().update({name: getattr(_native, name) for name in _exports})

__all__ = list(_exports)
__doc__ = getattr(_native, "__doc__", None)
