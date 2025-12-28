use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineType {
    Html,
    Control,
    End,
    Python,
}

#[derive(Debug, Clone)]
pub struct Line {
    pub line_type: LineType,
    pub text: String,
    pub line_number: usize,
    pub byte_offset: usize, // Byte offset in source
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceMapping {
    pub gen_line: usize,
    pub gen_col: usize,
    pub src_line: usize,
    pub src_col: usize,
}

/// Python injection piece - maps a source range to prefix/suffix for IDE injection
#[derive(Debug, Clone, Serialize)]
pub struct PythonPiece {
    pub prefix: String,
    pub suffix: String,
    pub src_start: usize,
    pub src_end: usize,
}


#[derive(Debug, Clone, Serialize)]
pub struct TranspileResult {
    pub python_code: String,
    pub source_mappings: Vec<SourceMapping>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python_pieces: Option<Vec<PythonPiece>>,
}

lazy_static! {
    static ref HTML_LINE: Regex = Regex::new(r"^[ \t]*<").unwrap();
    static ref CONTROL_LINE: Regex = Regex::new(
        r"^[ \t]*(async[ \t]+)?(if|for|while|match|def|class|elif|else|case|try|except|finally|with)[ \t(:]"
    ).unwrap();
    static ref END_LINE: Regex = Regex::new(r"^[ \t]*end[ \t]*$").unwrap();
    static ref TYPE_ANNOTATION: Regex = Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*(.+)$").unwrap();
    static ref AWAIT_START: Regex = Regex::new(r"^await\s").unwrap();
    static ref AWAIT_INLINE: Regex = Regex::new(r"[=(\[,:]\s*await\s").unwrap();
    static ref ASYNC_FOR: Regex = Regex::new(r"^\s*async\s+for\s").unwrap();
    static ref ASYNC_WITH: Regex = Regex::new(r"^\s*async\s+with\s").unwrap();
}

fn classify_line(text: &str) -> LineType {
    if END_LINE.is_match(text) {
        LineType::End
    } else if HTML_LINE.is_match(text) {
        LineType::Html
    } else if CONTROL_LINE.is_match(text) {
        LineType::Control
    } else {
        LineType::Python
    }
}

fn lex(source: &str) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut byte_offset = 0;

    for (i, text) in source.lines().enumerate() {
        lines.push(Line {
            line_type: classify_line(text),
            text: text.to_string(),
            line_number: i,
            byte_offset,
        });
        // Account for line content plus newline character
        byte_offset += text.len() + 1; // +1 for \n
    }

    lines
}

fn has_await(lines: &[Line]) -> bool {
    lines.iter().any(|line| {
        if line.line_type == LineType::Python {
            let text = line.text.trim();
            !text.starts_with('#') && (AWAIT_START.is_match(text) || AWAIT_INLINE.is_match(text))
        } else {
            false
        }
    })
}

fn has_async_construct(lines: &[Line]) -> bool {
    lines.iter().any(|line| {
        line.line_type == LineType::Control
            && (ASYNC_FOR.is_match(&line.text) || ASYNC_WITH.is_match(&line.text))
    })
}

fn find_structure(lines: &[Line]) -> (Vec<&Line>, Vec<&Line>, usize) {
    let mut leading = Vec::new();
    let mut params = Vec::new();
    let mut seen_param = false;
    let mut body_start = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.text.trim();

        if trimmed.is_empty() && !seen_param {
            leading.push(line);
            body_start = i + 1;
            continue;
        }

        if !seen_param && trimmed.starts_with('#') {
            leading.push(line);
            body_start = i + 1;
            continue;
        }

        if line.line_type == LineType::Python && TYPE_ANNOTATION.is_match(trimmed) {
            seen_param = true;
            params.push(line);
            body_start = i + 1;
            continue;
        }

        break;
    }

    (leading, params, body_start)
}

fn content_bounds(text: &str) -> (usize, usize) {
    let bytes = text.as_bytes();
    let start = bytes.iter().take_while(|&&b| b == b' ' || b == b'\t').count();
    let end = text.trim_end_matches(['\r', '\n']).len();
    (start, end)
}

/// Transpile source, optionally generating injection pieces for IDE integration
pub fn transpile_ext(source: &str, include_injection: bool) -> TranspileResult {
    let lines = lex(source);
    let (leading, params, body_start) = find_structure(&lines);
    let is_async = has_await(&lines) || has_async_construct(&lines);

    let mut output = Vec::new();
    let mut mappings = Vec::new();
    let mut python_pieces = if include_injection { Some(Vec::new()) } else { None };
    let mut out_line = 0;

    let indent = "    ";
    let def_kw = if is_async { "async def" } else { "def" };

    // Leading content (comments and empty lines)
    for line in &leading {
        let trimmed = line.text.trim();
        if !trimmed.is_empty() {
            let (start, end) = content_bounds(&line.text);
            output.push(line.text[start..end].to_string());
            mappings.push(SourceMapping {
                gen_line: out_line,
                gen_col: 0,
                src_line: line.line_number,
                src_col: start,
            });
            if let Some(ref mut p) = python_pieces {
                p.push(PythonPiece {
                    prefix: String::new(),
                    suffix: "\n".to_string(),
                    src_start: line.byte_offset + start,
                    src_end: line.byte_offset + end,
                });
            }
        } else {
            output.push(String::new());
            mappings.push(SourceMapping {
                gen_line: out_line,
                gen_col: 0,
                src_line: line.line_number,
                src_col: 0,
            });
            if let Some(ref mut p) = python_pieces {
                p.push(PythonPiece {
                    prefix: String::new(),
                    suffix: "\n".to_string(),
                    src_start: line.byte_offset,
                    src_end: line.byte_offset,
                });
            }
        }
        out_line += 1;
    }

    // Function signature with parameters
    if !params.is_empty() {
        let param_strs: Vec<_> = params
            .iter()
            .map(|p| {
                let (s, e) = content_bounds(&p.text);
                p.text[s..e].to_string()
            })
            .collect();

        output.push(format!("{} __hyper_template__({}):", def_kw, param_strs.join(", ")));

        let first = params[0];
        let (start, _) = content_bounds(&first.text);
        mappings.push(SourceMapping {
            gen_line: out_line,
            gen_col: def_kw.len() + " __hyper_template__(".len(),
            src_line: first.line_number,
            src_col: start,
        });

        // Generate injection pieces for parameters
        if let Some(ref mut p) = python_pieces {
            for (i, param) in params.iter().enumerate() {
                let (s, e) = content_bounds(&param.text);
                let prefix = if i == 0 {
                    format!("{} __hyper_template__(", def_kw)
                } else {
                    ", ".to_string()
                };
                let suffix = if i == params.len() - 1 {
                    "):\n".to_string()
                } else {
                    String::new()
                };
                p.push(PythonPiece {
                    prefix,
                    suffix,
                    src_start: param.byte_offset + s,
                    src_end: param.byte_offset + e,
                });
            }
        }
    } else {
        output.push(format!("{} __hyper_template__():", def_kw));
        let body_offset = if body_start < lines.len() {
            lines[body_start].byte_offset
        } else if !lines.is_empty() {
            lines.last().unwrap().byte_offset
        } else {
            0
        };
        mappings.push(SourceMapping {
            gen_line: out_line,
            gen_col: 0,
            src_line: body_start.min(lines.len().saturating_sub(1)),
            src_col: 0,
        });
        if let Some(ref mut p) = python_pieces {
            p.push(PythonPiece {
                prefix: format!("{} __hyper_template__():\n", def_kw),
                suffix: String::new(),
                src_start: body_offset,
                src_end: body_offset,
            });
        }
    }
    out_line += 1;

    // Body
    let mut level = 1usize;
    let mut stack: Vec<&str> = Vec::new();

    for i in body_start..lines.len() {
        let line = &lines[i];
        let (start, end) = content_bounds(&line.text);

        if start >= end {
            output.push(String::new());
            mappings.push(SourceMapping {
                gen_line: out_line,
                gen_col: 0,
                src_line: line.line_number,
                src_col: 0,
            });
            if let Some(ref mut p) = python_pieces {
                p.push(PythonPiece {
                    prefix: String::new(),
                    suffix: "\n".to_string(),
                    src_start: line.byte_offset,
                    src_end: line.byte_offset,
                });
            }
            out_line += 1;
            continue;
        }

        let trimmed = &line.text[start..end];
        let src_start = line.byte_offset + start;
        let src_end = line.byte_offset + end;

        match line.line_type {
            LineType::Control => {
                let is_dedent = ["else", "elif", "except", "finally"]
                    .iter()
                    .any(|kw| trimmed.starts_with(kw));
                let is_case = trimmed.starts_with("case");

                if is_dedent {
                    let print_level = level.saturating_sub(1).max(1);
                    output.push(format!("{}{}", indent.repeat(print_level), trimmed));
                    mappings.push(SourceMapping {
                        gen_line: out_line,
                        gen_col: indent.len() * print_level,
                        src_line: line.line_number,
                        src_col: start,
                    });
                    if let Some(ref mut p) = python_pieces {
                        p.push(PythonPiece {
                            prefix: indent.repeat(print_level),
                            suffix: "\n".to_string(),
                            src_start,
                            src_end,
                        });
                    }
                } else if is_case {
                    if stack.last() == Some(&"case") {
                        stack.pop();
                        level = level.saturating_sub(1);
                    }
                    output.push(format!("{}{}", indent.repeat(level), trimmed));
                    mappings.push(SourceMapping {
                        gen_line: out_line,
                        gen_col: indent.len() * level,
                        src_line: line.line_number,
                        src_col: start,
                    });
                    if let Some(ref mut p) = python_pieces {
                        p.push(PythonPiece {
                            prefix: indent.repeat(level),
                            suffix: "\n".to_string(),
                            src_start,
                            src_end,
                        });
                    }
                    stack.push("case");
                    level += 1;
                } else {
                    output.push(format!("{}{}", indent.repeat(level), trimmed));
                    mappings.push(SourceMapping {
                        gen_line: out_line,
                        gen_col: indent.len() * level,
                        src_line: line.line_number,
                        src_col: start,
                    });
                    if let Some(ref mut p) = python_pieces {
                        p.push(PythonPiece {
                            prefix: indent.repeat(level),
                            suffix: "\n".to_string(),
                            src_start,
                            src_end,
                        });
                    }
                    stack.push(if trimmed.starts_with("match") { "match" } else { "block" });
                    level += 1;
                }
            }

            LineType::End => {
                while stack.last() == Some(&"case") {
                    stack.pop();
                    level = level.saturating_sub(1);
                }
                if !stack.is_empty() {
                    stack.pop();
                    level = level.saturating_sub(1);
                }
                level = level.max(1);
                output.push(format!("{}pass", indent.repeat(level)));
                mappings.push(SourceMapping {
                    gen_line: out_line,
                    gen_col: 0,
                    src_line: line.line_number,
                    src_col: 0,
                });
                if let Some(ref mut p) = python_pieces {
                    p.push(PythonPiece {
                        prefix: format!("{}pass\n", indent.repeat(level)),
                        suffix: String::new(),
                        src_start: line.byte_offset,
                        src_end: line.byte_offset,
                    });
                }
            }

            LineType::Html => {
                output.push(format!("{}t\"\"\"{}\"\"\"", indent.repeat(level), trimmed));
                mappings.push(SourceMapping {
                    gen_line: out_line,
                    gen_col: indent.len() * level + 4,
                    src_line: line.line_number,
                    src_col: start,
                });
                if let Some(ref mut p) = python_pieces {
                    p.push(PythonPiece {
                        prefix: format!("{}t\"\"\"", indent.repeat(level)),
                        suffix: "\"\"\"\n".to_string(),
                        src_start,
                        src_end,
                    });
                }
            }

            LineType::Python => {
                output.push(format!("{}{}", indent.repeat(level), trimmed));
                mappings.push(SourceMapping {
                    gen_line: out_line,
                    gen_col: indent.len() * level,
                    src_line: line.line_number,
                    src_col: start,
                });
                if let Some(ref mut p) = python_pieces {
                    p.push(PythonPiece {
                        prefix: indent.repeat(level),
                        suffix: "\n".to_string(),
                        src_start,
                        src_end,
                    });
                }
            }
        }

        out_line += 1;
    }

    let mut code = output.join("\n");
    if !code.is_empty() && !code.ends_with('\n') {
        code.push('\n');
    }

    TranspileResult {
        python_code: code,
        source_mappings: mappings,
        python_pieces,
    }
}

/// Transpile source to Python (without injection pieces)
pub fn transpile(source: &str) -> TranspileResult {
    transpile_ext(source, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let result = transpile("name: str\n\n<div>Hello {name}</div>\n");
        assert!(result.python_code.contains("def __hyper_template__(name: str):"));
        assert!(result.python_code.contains("t\"\"\"<div>Hello {name}</div>\"\"\""));
    }

    #[test]
    fn test_async() {
        let result = transpile("id: int\n\ndata = await fetch(id)\n<div>{data}</div>\n");
        assert!(result.python_code.contains("async def __hyper_template__"));
    }

    #[test]
    fn test_control_flow() {
        let result = transpile("items: list\n\nfor item in items:\n    <li>{item}</li>\nend\n");
        assert!(result.python_code.contains("for item in items:"));
        assert!(result.python_code.contains("pass"));
    }
}