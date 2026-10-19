import ast
import inspect
import pathlib

import subtr_actor


def _stub_signatures():
    package_dir = pathlib.Path(subtr_actor.__file__).parent
    assert (package_dir / "py.typed").exists()
    tree = ast.parse((package_dir / "__init__.pyi").read_text())
    return {
        node.name: [arg.arg for arg in node.args.args]
        for node in tree.body
        if isinstance(node, ast.FunctionDef)
    }


def test_stubs_match_native_signatures():
    native = {
        name: list(inspect.signature(getattr(subtr_actor, name)).parameters)
        for name in subtr_actor.__all__
    }

    assert _stub_signatures() == native
