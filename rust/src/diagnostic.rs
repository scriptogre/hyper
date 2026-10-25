use crate::error::{CompileError, ParseError, highlight_inline_tags, highlight_syntax};
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};
use ruff_source_file::{LineIndex, OneIndexed, PositionEncoding, SourceLocation};
use ruff_text_size::TextSize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticData {
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    related_range: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    related_label: Option<String>,
}

pub fn create_diagnostic(error: &CompileError, source: &str) -> Diagnostic {
    match error {
        CompileError::Parse(error) => create_parse_diagnostic(error, source),
        CompileError::Generate(message) => Diagnostic {
            range: Range::default(),
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("generation-error".into())),
            source: Some("hyper".into()),
            message: format!("Generation error: {message}"),
            ..Diagnostic::default()
        },
    }
}

fn create_parse_diagnostic(error: &ParseError, source: &str) -> Diagnostic {
    let line_index = LineIndex::from_source_text(source);
    let data = DiagnosticData {
        help: error.help.clone(),
        related_range: error
            .related_range
            .map(|range| create_range(range.start.byte, range.end.byte, source, &line_index)),
        related_label: error.related_label.clone(),
    };

    Diagnostic {
        range: create_range(
            error.range.start.byte,
            error.range.end.byte,
            source,
            &line_index,
        ),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String(error.kind.code().into())),
        source: Some("hyper".into()),
        message: error.message.clone(),
        data: serde_json::to_value(data).ok(),
        ..Diagnostic::default()
    }
}

fn create_range(start: usize, end: usize, source: &str, line_index: &LineIndex) -> Range {
    Range::new(
        create_position(start, source, line_index),
        create_position(end, source, line_index),
    )
}

fn create_position(offset: usize, source: &str, line_index: &LineIndex) -> Position {
    let offset = TextSize::try_from(offset.min(source.len())).expect("source exceeds 4 GiB");
    let location = line_index.source_location(offset, source, PositionEncoding::Utf16);
    Position::new(
        location.line.to_zero_indexed() as u32,
        location.character_offset.to_zero_indexed() as u32,
    )
}

pub fn render_diagnostic(
    diagnostic: &Diagnostic,
    source: &str,
    filename: &str,
    color: bool,
) -> String {
    let red = if color { "\x1b[1;31m" } else { "" };
    let dim = if color { "\x1b[2m" } else { "" };
    let underline = if color { "\x1b[4m" } else { "" };
    let cyan = if color { "\x1b[1;38;5;73m" } else { "" };
    let reset = if color { "\x1b[0m" } else { "" };
    let data = read_data(diagnostic);
    let location_range = data.related_range.as_ref().unwrap_or(&diagnostic.range);
    let (location_line, location_col) = display_position(location_range.start, source);
    let mut output = String::new();

    output.push('\n');
    let location = format!("{}:{}:{}", filename, location_line + 1, location_col + 1);
    if color {
        let absolute_path = std::path::Path::new(filename)
            .canonicalize()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| filename.to_string());
        output.push_str(&format!(
            " {}file:{} \x1b]8;;file://{}\x07{}{}{}\x1b]8;;\x07\n",
            dim, reset, absolute_path, underline, location, reset
        ));
    } else {
        output.push_str(&format!(" file: {location}\n"));
    }

    let message = if color {
        highlight_inline_tags(&diagnostic.message)
    } else {
        diagnostic.message.clone()
    };
    output.push_str(&format!("{red}error:{reset} {message}\n"));

    render_range(
        &mut output,
        &diagnostic.range,
        None,
        source,
        color,
        red,
        dim,
        reset,
    );

    if let Some(related_range) = data.related_range {
        render_range(
            &mut output,
            &related_range,
            Some(data.related_label.as_deref().unwrap_or("opened here")),
            source,
            color,
            red,
            dim,
            reset,
        );
    }

    if let Some(help) = data.help {
        output.push('\n');
        for (index, help_line) in help.lines().enumerate() {
            let content = if color {
                highlight_inline_tags(help_line)
            } else {
                help_line.to_string()
            };
            if index == 0 {
                output.push_str(&format!(" {cyan}help:{reset} {content}\n"));
            } else {
                output.push_str(&format!("       {content}\n"));
            }
        }
    }

    output.push('\n');
    output
}

#[allow(clippy::too_many_arguments)]
fn render_range(
    output: &mut String,
    range: &Range,
    label: Option<&str>,
    source: &str,
    color: bool,
    red: &str,
    dim: &str,
    reset: &str,
) {
    let (start_line, start_col) = display_position(range.start, source);
    let (end_line, end_col) = display_position(range.end, source);
    let line_number = start_line + 1;
    let Some(source_line) = source.lines().nth(start_line) else {
        return;
    };
    let width = line_number.to_string().len().max(2);
    let highlighted = if color {
        highlight_syntax(source_line)
    } else {
        source_line.to_string()
    };

    if label.is_none() {
        output.push_str(&format!("{dim}{:>width$} |{reset}\n", ""));
    }
    output.push_str(&format!(
        "{dim}{line_number:>width$} |{reset} {highlighted}\n"
    ));

    let underline_length = if end_line == start_line {
        end_col.saturating_sub(start_col).max(1)
    } else {
        source_line.chars().count().saturating_sub(start_col).max(1)
    };
    let spaces = " ".repeat(start_col);
    let carets = "^".repeat(underline_length);
    if let Some(label) = label {
        output.push_str(&format!(
            "{dim}{:>width$} |{reset} {spaces}{dim}{carets} {label}{reset}\n",
            ""
        ));
    } else {
        output.push_str(&format!(
            "{dim}{:>width$} |{reset} {spaces}{red}{carets}{reset}\n",
            ""
        ));
    }
}

fn display_position(position: Position, source: &str) -> (usize, usize) {
    let line_index = LineIndex::from_source_text(source);
    let offset = line_index.offset(
        SourceLocation {
            line: OneIndexed::from_zero_indexed(position.line as usize),
            character_offset: OneIndexed::from_zero_indexed(position.character as usize),
        },
        source,
        PositionEncoding::Utf16,
    );
    let location = line_index.source_location(offset, source, PositionEncoding::Utf32);
    (
        location.line.to_zero_indexed(),
        location.character_offset.to_zero_indexed(),
    )
}

fn read_data(diagnostic: &Diagnostic) -> DiagnosticData {
    diagnostic
        .data
        .clone()
        .and_then(|data| serde_json::from_value(data).ok())
        .unwrap_or_default()
}
