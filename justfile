fix:
    cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features
    cargo +nightly fmt --all
    cargo sort --workspace --grouped

venv:
    UV_CACHE_DIR=/tmp/uv-cache uv venv .venv --python 3.13 --clear

build: venv
    UV_CACHE_DIR=/tmp/uv-cache uv pip install --python .venv/bin/python maturin
    UV_CACHE_DIR=/tmp/uv-cache .venv/bin/python -m maturin develop --features python

wheel: venv
    UV_CACHE_DIR=/tmp/uv-cache uv pip install --python .venv/bin/python "maturin[zig]"
    UV_CACHE_DIR=/tmp/uv-cache .venv/bin/python -m maturin build --release --features python --compatibility manylinux_2_28 --interpreter python3.11 python3.12 python3.13 --zig --auditwheel=repair

test-python: build
    UV_CACHE_DIR=/tmp/uv-cache uv pip install --python .venv/bin/python pytest
    .venv/bin/python -m pytest

publish: wheel
    UV_CACHE_DIR=/tmp/uv-cache uv publish target/wheels/*
