use pyo3::exceptions::{PySyntaxError, PyValueError};
use pyo3::prelude::*;

fn compile_file(
    py: Python<'_>,
    source: &str,
    filename: Option<&str>,
    color: bool,
) -> PyResult<Option<crate::CompileResult>> {
    let display_filename = filename.unwrap_or("<string>");
    let builtins = PyModule::import(py, "builtins")?;
    match builtins
        .getattr("compile")?
        .call1((source, display_filename, "exec"))
    {
        Ok(_) => return Ok(None),
        Err(error) if error.is_instance_of::<PySyntaxError>(py) => {}
        Err(error) => return Err(error),
    }

    crate::compile_python_file(source, filename)
        .map(Some)
        .map_err(|err| {
            let diagnostic = if color {
                err.render_color(source, display_filename)
            } else {
                err.render(source, display_filename)
            };
            PyValueError::new_err(diagnostic)
        })
}

#[pyfunction]
#[pyo3(signature = (source, filename=None, *, color=false))]
fn transpile(
    py: Python<'_>,
    source: &str,
    filename: Option<&str>,
    color: bool,
) -> PyResult<String> {
    compile_file(py, source, filename, color)
        .map(|result| result.map_or_else(|| source.to_string(), |result| result.code))
}

#[pyfunction]
#[pyo3(signature = (source, filename=None, *, color=false))]
fn transpile_file(
    py: Python<'_>,
    source: &str,
    filename: Option<&str>,
    color: bool,
) -> PyResult<(String, Option<String>)> {
    compile_file(py, source, filename, color).map(|result| {
        result.map_or_else(
            || (source.to_string(), None),
            |result| (result.code, result.component_name),
        )
    })
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(transpile, m)?)?;
    m.add_function(wrap_pyfunction!(transpile_file, m)?)?;
    Ok(())
}
