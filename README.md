# zopyx.pyjsonata

Python bindings for the Rust implementation of JSONata, powered by PyO3 and maturin. This package exposes the JSONata evaluator from `jsonata-rs` to Python with a small, focused API.

- JSONata reference docs: https://docs.jsonata.org/overview.html
- JSONata playground: https://www.stedi.com/jsonata/playground
- Rust implementation: https://github.com/Stedi/jsonata-rs

## Quick start

### Install from source (local)

```bash
UV_CACHE_DIR=/tmp/uv-cache uv venv .venv --python 3.13 --clear
UV_CACHE_DIR=/tmp/uv-cache uv pip install --python .venv/bin/python "maturin[zig]"
UV_CACHE_DIR=/tmp/uv-cache .venv/bin/python -m maturin develop --features python
```

### Use

```python
from zopyx.pyjsonata import evaluate, UNDEFINED, Jsonata

# Evaluate a simple expression
print(evaluate("1 + 1"))

# Evaluate with input data
print(evaluate('"Hello, " & name & "!"', {"name": "world"}))

# Provide variable bindings
bindings = {"x": 2, "y": 3}
print(evaluate("$x + $y", UNDEFINED, bindings))

# Reuse a compiled expression
expr = Jsonata("$sum([1,2,3])")
print(expr.evaluate())
```

## API

### `evaluate(expr, input=UNDEFINED, bindings=None, max_depth=None, time_limit=None)`

- `expr`: JSONata expression string
- `input`: JSON data for `$` (default `UNDEFINED` = no input)
- `bindings`: dict of variable bindings, e.g. `{"x": 1}`
- `max_depth`: optional evaluator depth limit
- `time_limit`: optional evaluation time limit

Returns standard Python types: `dict`, `list`, `str`, `float`, `int`, `bool`, `None`, or `UNDEFINED`.

### `Jsonata(expr)`

Constructs a reusable expression object.

- `Jsonata.evaluate(...)` has the same signature as `evaluate` but with the expression pre-parsed.

### `UNDEFINED`

Represents missing input (distinct from JSON `null`). In Python, `None` maps to JSON `null`.

## Errors

Errors raise `ValueError` and include the JSONata error code prefix (e.g. `T0410`), matching the Rust implementation.

## Build wheels (manylinux)

```bash
UV_CACHE_DIR=/tmp/uv-cache .venv/bin/python -m maturin build \
  --release \
  --features python \
  --compatibility manylinux_2_28 \
  --interpreter python3.11 python3.12 python3.13 \
  --zig \
  --auditwheel=repair
```

## Tests

- Rust tests: `cargo test`
- Python testsuite port: `make test-python` or `just test-python`

## Limitations

This is an incomplete JSONata implementation. Many reference tests are skipped under `tests/**/skip`.

## License

Apache-2.0.

## Maintainer

Andreas Jung — info@zopyx.com
