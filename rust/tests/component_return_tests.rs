use hyper::{CompileOptions, compile};

fn compile_source(source: &str) -> Result<String, String> {
    compile(
        source,
        &CompileOptions {
            function_name: Some("Page".to_string()),
            include_ranges: false,
        },
    )
    .map(|result| result.code)
    .map_err(|error| error.to_string())
}

#[test]
fn bare_return_stops_component_rendering() {
    let code = compile_source(
        r#"component Guard(*, visible: bool):
    if not visible:
        return
    end
    <p>Visible</p>
end
"#,
    )
    .expect("bare return should compile");

    assert!(code.contains("        return"));
}

#[test]
fn component_return_value_is_rejected() {
    let error = compile_source(
        r#"component Invalid():
    return "value"
end
"#,
    )
    .expect_err("return values should fail");

    assert!(error.contains("cannot return a value"));
}

#[test]
fn component_yield_is_rejected() {
    let error = compile_source(
        r#"component Invalid():
    yield "value"
end
"#,
    )
    .expect_err("explicit yield should fail");

    assert!(error.contains("cannot use explicit yield"));
}

#[test]
fn nested_python_function_keeps_return_and_yield() {
    let code = compile_source(
        r#"component Valid():
    def values():
        yield "value"
        return
    end
    for value in values():
        <p>{value}</p>
    end
end
"#,
    )
    .expect("nested Python control flow should compile");

    assert!(code.contains("        yield \"value\""));
    assert!(code.contains("        return"));
}
