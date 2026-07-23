use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn compile_file(source: &str, filename: Option<&str>) -> PyResult<crate::CompileResult> {
    crate::compile_python_file(source, filename)
        .map_err(|err| PyValueError::new_err(err.render(source, filename.unwrap_or("<string>"))))
}

#[pyfunction]
#[pyo3(signature = (source, filename=None))]
fn transpile(source: &str, filename: Option<&str>) -> PyResult<String> {
    compile_file(source, filename).map(|result| result.code)
}

#[pyfunction]
#[pyo3(signature = (source, filename=None))]
fn transpile_file(source: &str, filename: Option<&str>) -> PyResult<(String, Option<String>)> {
    compile_file(source, filename).map(|result| (result.code, result.component_name))
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(transpile, m)?)?;
    m.add_function(wrap_pyfunction!(transpile_file, m)?)?;
    Ok(())
}
