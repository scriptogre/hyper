use hyper_transpiler::transpile as rust_transpile;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct SourceMapping {
    #[pyo3(get)]
    pub gen_line: usize,
    #[pyo3(get)]
    pub gen_col: usize,
    #[pyo3(get)]
    pub src_line: usize,
    #[pyo3(get)]
    pub src_col: usize,
}

#[pyclass]
#[derive(Clone)]
pub struct TranspileResult {
    #[pyo3(get)]
    pub python_code: String,
    #[pyo3(get)]
    pub source_mappings: Vec<SourceMapping>,
}

#[pyfunction]
fn transpile(source: &str) -> TranspileResult {
    let result = rust_transpile(source);
    TranspileResult {
        python_code: result.python_code,
        source_mappings: result
            .source_mappings
            .into_iter()
            .map(|m| SourceMapping {
                gen_line: m.gen_line,
                gen_col: m.gen_col,
                src_line: m.src_line,
                src_col: m.src_col,
            })
            .collect(),
    }
}

#[pymodule]
fn _hyper_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(transpile, m)?)?;
    m.add_class::<TranspileResult>()?;
    m.add_class::<SourceMapping>()?;
    Ok(())
}
