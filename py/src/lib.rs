use original_jsondiffpatch::diff;
use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};

/// A Python module implemented in Rust.
#[pymodule]
fn jsondiffpatch(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(from_string, m)?)?;
    Ok(())
}

#[pyfunction]
fn from_string(left: &Bound<'_, PyAny>, right: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    // Convert Python objects to JSON strings first
    let left_json: String = left.call_method0("__str__")?.extract()?;
    let right_json: String = right.call_method0("__str__")?.extract()?;

    // Parse JSON strings with proper error handling
    let left_value: serde_json::Value = serde_json::from_str(&left_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    let right_value: serde_json::Value = serde_json::from_str(&right_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    Python::with_gil(|py| match diff(&left_value, &right_value) {
        Some(delta) => {
            let serializable = delta.to_serializable();
            let py_dict = pythonize(py, &serializable)?;
            Ok(py_dict.into())
        }
        None => Ok(py.None()),
    })
}
