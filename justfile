fix:
    cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features
    cargo +nightly fmt --all
    cargo sort --workspace --grouped

venv:
    UV_CACHE_DIR=/tmp/uv-cache uv venv .venv --python 3.13

build: venv
    UV_CACHE_DIR=/tmp/uv-cache uv pip install --python .venv/bin/python maturin
    UV_CACHE_DIR=/tmp/uv-cache .venv/bin/python -m maturin develop --features python

wheel: venv
    UV_CACHE_DIR=/tmp/uv-cache uv pip install --python .venv/bin/python maturin
    UV_CACHE_DIR=/tmp/uv-cache .venv/bin/python -m maturin build --features python
