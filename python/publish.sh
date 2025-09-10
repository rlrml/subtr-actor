#!/usr/bin/env bash

# Script to build and publish the Python package to PyPI
# This script assumes you have maturin and twine installed

echo "Building the Python package with maturin..."
maturin build --release

echo "Uploading to PyPI..."
twine upload dist/subtr_actor_py-0.1.10-*.whl

echo "Done! Package should now be available on PyPI as subtr-actor-py version 0.1.10"