from setuptools import setup, Extension, find_packages
import os

# Read the version from pyproject.toml
version = "0.1.10"

setup(
    name="subtr-actor-py",
    version=version,
    description="Python bindings for the Rocket League replay processing library subtr-actor.",
    author="Ivan Malison",
    author_email="ivanmalison@gmail.com",
    url="https://github.com/rlrml/subtr-actor",
    packages=find_packages(),
    package_data={
        "": ["*.so"],
    },
    include_package_data=True,
    data_files=[("", ["subtr_actor.so"])],
    python_requires=">=3.7,<4.0",
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
)