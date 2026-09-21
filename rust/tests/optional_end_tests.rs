use hyper::{CompileOptions, compile};

fn compiled(source: &str) -> String {
    compile(source, &CompileOptions::default())
        .unwrap_or_else(|error| panic!("{}", error.render(source, "Page.hyper")))
        .code
}

fn assert_optional_end(source: &str) {
    let without_end = source
        .lines()
        .filter(|line| line.trim() != "end")
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(compiled(source), compiled(&without_end));
}

#[test]
fn if_branches_close_by_dedent_inside_html() {
    assert_optional_end(
        "---\n<div>\n    if visible:\n        <p>Yes</p>\n    elif pending:\n        <p>Wait</p>\n    else:\n        <p>No</p>\n    end\n    <span>Always</span>\n</div>\n",
    );
}

#[test]
fn for_and_while_close_by_dedent() {
    assert_optional_end(
        "---\nfor item in items:\n    while item.ready:\n        <p>{item}</p>\n        break\n    end\nend\n<footer>Done</footer>\n",
    );
}

#[test]
fn with_and_async_blocks_close_by_dedent() {
    assert_optional_end(
        "---\nwith resource() as value:\n    <p>{value}</p>\nend\nasync with connection() as connection:\n    async for row in connection:\n        <p>{row}</p>\n    end\nend\n<footer>Done</footer>\n",
    );
}

#[test]
fn match_cases_share_the_match_boundary() {
    assert_optional_end(
        "---\nmatch status:\n    case 1:\n        <p>One</p>\n    case _:\n        <p>Other</p>\nend\n<footer>Done</footer>\n",
    );
}

#[test]
fn try_clauses_share_the_try_boundary() {
    assert_optional_end(
        "---\ntry:\n    <p>{load()}</p>\nexcept ValueError:\n    <p>Error</p>\nelse:\n    <p>Success</p>\nfinally:\n    cleanup()\nend\n<footer>Done</footer>\n",
    );
}

#[test]
fn header_and_library_definitions_use_the_same_boundaries() {
    let definitions = "class Theme:\n    def color(self):\n        return 'blue'\n    end\nend\n\ndef label():\n    return 'Home'\nend\n";
    assert_optional_end(definitions);
    assert_optional_end(&format!("{definitions}---\n<p>{{label()}}</p>\n"));
}

#[test]
fn template_definitions_do_not_consume_following_definitions() {
    assert_optional_end(
        "from hyper import Component\ndef First() -> Component:\n    <p>First</p>\nend\n\ndef Second() -> Component:\n    <p>Second</p>\nend\n",
    );
}

#[test]
fn end_may_close_an_outer_block_after_an_implicit_inner_dedent() {
    let implicit_inner = "---\nif visible:\n    for item in items:\n        <p>{item}</p>\nend\n<footer>Done</footer>\n";
    let explicit_inner = "---\nif visible:\n    for item in items:\n        <p>{item}</p>\n    end\nend\n<footer>Done</footer>\n";
    assert_eq!(compiled(implicit_inner), compiled(explicit_inner));
}

#[test]
fn end_accepts_a_trailing_comment() {
    assert_eq!(
        compiled("---\nif visible:\n    <p>Yes</p>\nend # visible\n"),
        compiled("---\nif visible:\n    <p>Yes</p>\nend\n"),
    );
}

#[test]
fn indentation_errors_and_unmatched_end_are_rejected() {
    let sources = [
        "---\nif visible:\n<p>Not indented</p>\n",
        "---\nif visible:\n    <p>Yes</p>\n  end\n",
        "---\nif visible:\n    <p>Yes</p>\n    end\n",
        "---\n<p>Page</p>\nend\n",
        "---\nif visible:\n    <p>Yes</p>\n  <p>Bad dedent</p>\n",
    ];
    let accepted: Vec<_> = sources
        .into_iter()
        .filter(|source| compile(source, &CompileOptions::default()).is_ok())
        .collect();
    assert!(accepted.is_empty(), "Unexpectedly accepted: {accepted:?}");
}
