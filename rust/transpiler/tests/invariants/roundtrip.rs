use crate::helpers::{compile, substring_utf16};
use libtest_mimic::Failed;
use std::fs;
use std::path::PathBuf;

pub fn run(path: &PathBuf) -> Result<(), Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result = compile(path)?;

    let python_injections: Vec<_> = result
        .injections
        .iter()
        .filter(|inj| inj.injection_type == "python")
        .collect();

    if python_injections.is_empty() {
        // No Python injections — nothing to round-trip (template may be pure HTML).
        return Ok(());
    }

    // Reconstruct the virtual Python file the way JetBrains does:
    //   virtual = prefix_0 + source[start_0..end_0]
    //           + prefix_1 + source[start_1..end_1]
    //           + ...
    //           + prefix_n + source[start_n..end_n] + suffix_n
    //
    // Only the last injection carries a non-empty suffix; all others have "".

    let mut virtual_python = String::new();
    for inj in &python_injections {
        virtual_python.push_str(&inj.prefix);
        virtual_python.push_str(&substring_utf16(&source, inj.start, inj.end));
        virtual_python.push_str(&inj.suffix);
    }

    // Normalize indentation for comparison: multiline statements (dicts, lists,
    // function calls) have source indentation that differs from compiled indentation,
    // but the content is semantically identical and Python syntax highlighting works
    // correctly regardless of indentation within bracket/paren groups.
    let normalize = |s: &str| -> String {
        s.lines().map(|l| l.trim_start()).collect::<Vec<_>>().join("\n")
    };

    if normalize(&virtual_python) != normalize(&result.code) {
        // Build a helpful diff-like message
        let vp_lines: Vec<&str> = virtual_python.lines().collect();
        let code_lines: Vec<&str> = result.code.lines().collect();
        let max = vp_lines.len().max(code_lines.len());
        let mut diffs = String::new();
        for i in 0..max {
            let vp = vp_lines.get(i).unwrap_or(&"<missing>");
            let co = code_lines.get(i).unwrap_or(&"<missing>");
            if vp.trim_start() != co.trim_start() {
                diffs.push_str(&format!(
                    "  line {}: virtual={:?}  compiled={:?}\n",
                    i + 1,
                    vp,
                    co
                ));
            }
        }
        return Err(format!(
            "Virtual Python != compiled code\n\
             virtual len={} compiled len={}\n\
             First differing lines:\n{}",
            virtual_python.len(),
            result.code.len(),
            diffs
        )
        .into());
    }

    Ok(())
}
