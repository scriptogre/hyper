use hyper::{CompileOptions, compile};

fn assert_opening_tag_ranges_cover_source(source: &str, opening_start: usize, opening_end: usize) {
    assert!(source.is_ascii());

    let result = compile(
        source,
        &CompileOptions {
            function_name: Some("Template".to_string()),
            include_ranges: true,
        },
    )
    .expect("multiline opening tag should compile");

    let mut covered = vec![false; source.len()];
    for segment in result.segments {
        for offset in segment.source_start..segment.source_end {
            covered[offset] = true;
        }
    }
    for braces in result.expression_braces {
        covered[braces.open] = true;
        covered[braces.close] = true;
    }

    for (offset, character) in source.char_indices() {
        if !(opening_start..opening_end).contains(&offset) || character.is_whitespace() {
            continue;
        }

        // Grammar highlighting owns delimiters that cannot form meaningful injections alone.
        let prefix = &source[opening_start..offset];
        let grammar_owned = (character == '<' && source[offset..].starts_with("<{"))
            || (character == '=' && source[offset..].starts_with("={"))
            || (character == '*' && (prefix.ends_with('{') || prefix.ends_with("{*")));
        if !grammar_owned {
            assert!(
                covered[offset],
                "opening-tag source byte {offset} ({character:?}) has no source range"
            );
        }
    }
}

#[test]
fn multiline_html_opening_tag_has_complete_source_ranges() {
    let source = r#"active: bool
attrs: dict
---
<div
    id="profile"
    class={["card", {"active": active}]}
    {**attrs}
>
    Content
</div>
"#;
    let start = source.find("<div").unwrap();
    let end = source[start..].find('>').unwrap() + start + 1;

    assert_opening_tag_ranges_cover_source(source, start, end);
}

#[test]
fn multiline_component_opening_tag_has_complete_source_ranges() {
    let source = r#"Card: object
title: str
---
<{Card}
    title={title}
    selected
>
    Content
</{Card}>
"#;
    let start = source.find("<{Card}").unwrap();
    let end = source[start..].find('>').unwrap() + start + 1;

    assert_opening_tag_ranges_cover_source(source, start, end);
}
