use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn compile_file(
    source: &str,
    filename: Option<&str>,
    color: bool,
) -> PyResult<crate::CompileResult> {
    crate::compile_python_file(source, filename).map_err(|err| {
        let filename = filename.unwrap_or("<string>");
        let diagnostic = if color {
            err.render_color(source, filename)
        } else {
            err.render(source, filename)
        };
        PyValueError::new_err(diagnostic)
    })
}

#[pyfunction]
#[pyo3(signature = (source, filename=None, *, color=false))]
fn transpile(source: &str, filename: Option<&str>, color: bool) -> PyResult<String> {
    compile_file(source, filename, color).map(|result| result.code)
}

#[pyfunction]
#[pyo3(signature = (source, filename=None, *, color=false))]
fn transpile_file(
    source: &str,
    filename: Option<&str>,
    color: bool,
) -> PyResult<(String, Option<String>)> {
    compile_file(source, filename, color).map(|result| (result.code, result.component_name))
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(transpile, m)?)?;
    m.add_function(wrap_pyfunction!(transpile_file, m)?)?;
    Ok(())
}
