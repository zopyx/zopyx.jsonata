#![allow(clippy::useless_conversion)]

use std::collections::HashMap;

use bumpalo::Bump;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::sync::GILOnceCell;
use pyo3::types::{PyAny, PyBool, PyDict, PyFloat, PyList, PyString, PyTuple};

use crate::{Error, JsonAta, Value};

#[pyclass(module = "zopyx.pyjsonata")]
#[derive(Clone)]
struct UndefinedType;

#[pymethods]
impl UndefinedType {
    fn __repr__(&self) -> &'static str {
        "Undefined"
    }

    fn __bool__(&self) -> bool {
        false
    }
}

#[pyclass(module = "zopyx.pyjsonata")]
struct Jsonata {
    expr: String,
}

#[pymethods]
impl Jsonata {
    #[new]
    fn new(expr: String) -> Self {
        Self { expr }
    }

    #[pyo3(signature = (input=None, bindings=None, max_depth=None, time_limit=None))]
    #[allow(clippy::useless_conversion)]
    fn evaluate<'py>(
        &self,
        py: Python<'py>,
        input: Option<Bound<'py, PyAny>>,
        bindings: Option<Bound<'py, PyDict>>,
        max_depth: Option<usize>,
        time_limit: Option<usize>,
    ) -> PyResult<PyObject> {
        let arena = Bump::new();
        let jsonata = JsonAta::new(&self.expr, &arena).map_err(to_py_err)?;

        let mut bindings_owned: HashMap<String, serde_json::Value> = HashMap::new();
        let bindings_ref: Option<HashMap<&str, &serde_json::Value>> = if let Some(dict) = bindings {
            for (key, value) in dict.iter() {
                let key_str = key
                    .downcast::<PyString>()
                    .map_err(|_| PyTypeError::new_err("bindings keys must be strings"))?
                    .to_str()?
                    .to_string();
                let json_value = py_to_json(&value)?;
                bindings_owned.insert(key_str, json_value);
            }

            let mut refs = HashMap::with_capacity(bindings_owned.len());
            for (key, value) in bindings_owned.iter() {
                refs.insert(key.as_str(), value);
            }
            Some(refs)
        } else {
            None
        };

        let input_value = match input {
            None => None,
            Some(obj) => {
                if obj.is_instance_of::<UndefinedType>() {
                    None
                } else if obj.is_none() {
                    Some(serde_json::Value::Null)
                } else {
                    Some(py_to_json(&obj)?)
                }
            }
        };

        let result = jsonata.evaluate_json(
            input_value.as_ref(),
            bindings_ref.as_ref(),
            max_depth,
            time_limit,
        );
        let value = result.map_err(to_py_err)?;
        value_to_py(py, value)
    }
}

#[pymodule]
fn _native(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Jsonata>()?;
    m.add_class::<UndefinedType>()?;

    let undefined = Py::new(py, UndefinedType)?;
    UNDEFINED_SINGLETON.set(py, undefined.clone_ref(py)).ok();
    m.add("UNDEFINED", undefined)?;

    Ok(())
}

fn to_py_err(err: Error) -> PyErr {
    PyValueError::new_err(err.to_string())
}

fn py_to_json(obj: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    if obj.is_none() {
        return Ok(serde_json::Value::Null);
    }

    if obj.is_instance_of::<PyBool>() {
        return Ok(serde_json::Value::Bool(obj.extract::<bool>()?));
    }

    if let Ok(i) = obj.extract::<i64>() {
        return Ok(serde_json::Value::Number(i.into()));
    }

    if let Ok(u) = obj.extract::<u64>() {
        return Ok(serde_json::Value::Number(u.into()));
    }

    if obj.is_instance_of::<PyFloat>() {
        let f = obj.extract::<f64>()?;
        let number = serde_json::Number::from_f64(f)
            .ok_or_else(|| PyValueError::new_err("float value is not finite"))?;
        return Ok(serde_json::Value::Number(number));
    }

    if let Ok(s) = obj.downcast::<PyString>() {
        return Ok(serde_json::Value::String(s.to_str()?.to_string()));
    }

    if let Ok(list) = obj.downcast::<PyList>() {
        let mut items = Vec::with_capacity(list.len());
        for item in list.iter() {
            items.push(py_to_json(&item)?);
        }
        return Ok(serde_json::Value::Array(items));
    }

    if let Ok(tuple) = obj.downcast::<PyTuple>() {
        let mut items = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            items.push(py_to_json(&item)?);
        }
        return Ok(serde_json::Value::Array(items));
    }

    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = serde_json::Map::with_capacity(dict.len());
        for (key, value) in dict.iter() {
            let key_str = key
                .downcast::<PyString>()
                .map_err(|_| PyTypeError::new_err("dict keys must be strings"))?
                .to_str()?;
            map.insert(key_str.to_string(), py_to_json(&value)?);
        }
        return Ok(serde_json::Value::Object(map));
    }

    Err(PyTypeError::new_err("unsupported type for JSON conversion"))
}

static UNDEFINED_SINGLETON: GILOnceCell<Py<UndefinedType>> = GILOnceCell::new();

fn value_to_py(py: Python<'_>, value: &Value<'_>) -> PyResult<PyObject> {
    match value {
        Value::Undefined => {
            let undefined = UNDEFINED_SINGLETON
                .get(py)
                .ok_or_else(|| PyValueError::new_err("UNDEFINED not initialized"))?;
            Ok(undefined.clone_ref(py).to_object(py))
        }
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok(b.to_object(py)),
        Value::Number(n) => Ok(n.to_object(py)),
        Value::String(s) => Ok(s.as_str().to_object(py)),
        Value::Array(values, _) => {
            let mut out = Vec::with_capacity(values.len());
            for item in values.iter() {
                out.push(value_to_py(py, item)?);
            }
            Ok(PyList::new_bound(py, out).to_object(py))
        }
        Value::Object(obj) => {
            let dict = PyDict::new_bound(py);
            for (key, val) in obj.iter() {
                dict.set_item(key.as_str(), value_to_py(py, val)?)?;
            }
            Ok(dict.to_object(py))
        }
        Value::Range(range) => {
            let mut out = Vec::with_capacity(range.len());
            for i in 0..range.len() {
                if let Some(item) = range.nth(i) {
                    out.push(value_to_py(py, item)?);
                }
            }
            Ok(PyList::new_bound(py, out).to_object(py))
        }
        Value::Regex(_)
        | Value::Lambda { .. }
        | Value::NativeFn { .. }
        | Value::Transformer { .. } => Err(PyTypeError::new_err(
            "unsupported JSONata value type for Python conversion",
        )),
    }
}
