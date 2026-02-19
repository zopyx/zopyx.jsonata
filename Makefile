UV_CACHE_DIR ?= /tmp/uv-cache
PYTHON ?= 3.13
VENV ?= .venv
PYTHON_BIN := $(VENV)/bin/python

.PHONY: venv build wheel test-python publish

venv:
	UV_CACHE_DIR=$(UV_CACHE_DIR) uv venv $(VENV) --python $(PYTHON) --clear

build: venv
	UV_CACHE_DIR=$(UV_CACHE_DIR) uv pip install --python $(PYTHON_BIN) maturin
	UV_CACHE_DIR=$(UV_CACHE_DIR) $(PYTHON_BIN) -m maturin develop --features python

wheel: venv
	UV_CACHE_DIR=$(UV_CACHE_DIR) uv pip install --python $(PYTHON_BIN) maturin
	UV_CACHE_DIR=$(UV_CACHE_DIR) $(PYTHON_BIN) -m maturin build --release --features python --compatibility manylinux_2_28 --interpreter python3.13

test-python: build
	UV_CACHE_DIR=$(UV_CACHE_DIR) uv pip install --python $(PYTHON_BIN) pytest
	$(PYTHON_BIN) -m pytest

publish: wheel
	UV_CACHE_DIR=$(UV_CACHE_DIR) uv publish dist/*
