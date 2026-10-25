use hyper::{CompileOptions, compile};

#[test]
fn only_generated_names_are_imported() {
    let source = r#"def Page(*, value: Any, items: Iterable[Any], callback: Callable[[Any], Any]) -> Component:
    @subcomponent
    def Label() -> Component:
        <span>{safe(callback(value))}</span>

    <{Label} />
"#;

    let code = compile(source, &CompileOptions::default())
        .into_python()
        .unwrap()
        .code;
    let imports: Vec<_> = code
        .lines()
        .filter(|line| line.starts_with("from "))
        .collect();

    assert_eq!(imports, ["from hyper import component, escape"]);
}

#[test]
fn generated_slot_annotations_import_iterable() {
    let code = compile(
        "---\n<section>{...}</section>\n",
        &CompileOptions::default(),
    )
    .into_python()
    .unwrap()
    .code;

    assert!(code.contains("from collections.abc import Iterable"));
}
