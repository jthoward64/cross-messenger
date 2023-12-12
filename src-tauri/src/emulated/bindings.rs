use base64::{engine::general_purpose, Engine};
use pyo3::{prelude::*, prepare_freethreaded_python, types::PyBytes};
use tauri::ipc::InvokeError;

fn generate_validation_data_py(py: Python) -> PyResult<Py<PyBytes>> {
    let py_mparser: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/emulated/pypush/mparser.py"
    ));
    let py_jelly: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/emulated/pypush/jelly.py"
    ));
    let py_nac: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/emulated/pypush/nac.py"
    ));
    PyModule::from_code(py, py_mparser, "mparser.py", "mparser")?;
    PyModule::from_code(py, py_jelly, "jelly.py", "jelly")?;
    let nac_module = PyModule::from_code(py, py_nac, "", "")?;
    let generate_validation_data_func = nac_module.getattr("generate_validation_data")?;
    let generate_validation_data = generate_validation_data_func.call0()?;
    match generate_validation_data.extract::<Py<PyBytes>>() {
        Ok(generate_validation_data) => Ok(generate_validation_data),
        Err(e) => Err(e),
    }
}

#[derive(Debug)]
pub enum ValidationDataError {
    PyErr(PyErr),
}

impl From<PyErr> for ValidationDataError {
    fn from(e: PyErr) -> Self {
        ValidationDataError::PyErr(e)
    }
}

impl Into<InvokeError> for ValidationDataError {
    fn into(self) -> InvokeError {
        match self {
            ValidationDataError::PyErr(e) => InvokeError::from(e.to_string()),
        }
    }
}

pub fn generate_validation_data() -> Result<String, ValidationDataError> {
    prepare_freethreaded_python();
    let raw_data = match Python::with_gil(|py| -> PyResult<Vec<u8>> {
        // Print out some environment information
        let sys_module = PyModule::import(py, "sys")?;
        let version_info = sys_module.getattr("version_info")?;
        let version_info_str = version_info.str()?;
        println!("Python version: {}", version_info_str);
        let path = sys_module.getattr("path")?;
        let path_str = path.str()?;
        println!("Python path: {}", path_str);

        let py_validation_data = generate_validation_data_py(py)?;
        let validation_data = py_validation_data.as_bytes(py).to_vec();
        Ok(validation_data)
    }) {
        Ok(validation_data) => Ok(validation_data),
        Err(e) => Err(e.into()),
    };

    match raw_data {
        Ok(raw_data) => Ok(general_purpose::STANDARD.encode(&raw_data)),
        Err(e) => Err(e),
    }
}
