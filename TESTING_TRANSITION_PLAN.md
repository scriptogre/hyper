# Testing Transition Plan

## Current Situation

**Unstaged changes** (62 files):
- Core transpiler modifications: `ast.rs`, `tree_builder.rs`, `python.rs`, `output.rs`
- 50+ snapshot file updates
- These changes implement `close_span` tracking and HTML injection ranges

**Commits since master**:
- `cccd139` - Fix expression injection to include f-string braces
- `6113dcd` - Improve error message formatting and syntax highlighting

## Git Strategy

### Option A: Preserve current work (Recommended)

```bash
# 1. Create branch for current injection work
git checkout -b injection-html-ranges
git add -A
git commit -m "Add HTML injection ranges and close_span tracking"

# 2. Return to master for testing transition
git checkout master

# 3. Create branch for testing transition
git checkout -b testing-infrastructure
```

This preserves the injection work. Once testing infrastructure is solid, we can rebase/merge the injection branch on top.

### Option B: Stash and continue

```bash
git stash push -m "HTML injection range work in progress"
git checkout -b testing-infrastructure
# Later: git stash pop
```

Riskier - stashes can be lost.

---

## Testing Transition: Insta → `.expected.py`

### What We're Replacing

Current system (`golden_tests.rs`):
- `.hyper` files in `tests/<category>/`
- Snapshots in `snapshots/<category>@<name>@<suffix>.snap`
- Three test functions: `test_transpile_output`, `test_transpile_injections`, `test_transpile_errors`

### New System Design

```
tests/
├── basic/
│   ├── expression.hyper
│   ├── expression.expected.py       # Expected Python output
│   └── expression.expected.json     # Expected injections (optional)
├── control_flow/
│   ├── if_else.hyper
│   ├── if_else.expected.py
│   └── if_else.expected.json
└── errors/
    ├── invalid_nesting.hyper
    └── invalid_nesting.expected.err  # Expected error message
```

### Implementation Steps

#### Phase 1: Create `.expected.py` generator

```rust
// In tests/generate_expected.rs or a binary
// Run once to generate initial .expected.py from current snapshots

fn generate_expected_files() {
    for entry in glob("tests/**/*.hyper") {
        let source = fs::read_to_string(&entry);
        let result = pipeline.compile(&source, &opts);

        // Write .expected.py
        let expected_path = entry.with_extension("expected.py");
        fs::write(&expected_path, &result.code);

        // Write .expected.json for injections
        if !result.injections.is_empty() {
            let json_path = entry.with_extension("expected.json");
            fs::write(&json_path, serde_json::to_string_pretty(&result)?);
        }
    }
}
```

#### Phase 2: Create new test runner

```rust
// tests/expected_tests.rs

#[test]
fn test_all_expected() {
    for hyper_file in glob("tests/**/*.hyper").unwrap() {
        let name = hyper_file.file_stem().unwrap();
        let dir = hyper_file.parent().unwrap();

        // Skip error tests
        if dir.ends_with("errors") { continue; }

        let source = fs::read_to_string(&hyper_file).unwrap();
        let expected_py = dir.join(format!("{}.expected.py", name));
        let expected_json = dir.join(format!("{}.expected.json", name));

        let result = pipeline.compile(&source, &opts).unwrap();

        // Compare Python output
        if expected_py.exists() {
            let expected = fs::read_to_string(&expected_py).unwrap();
            assert_eq!(result.code.trim(), expected.trim(),
                "Output mismatch for {}", hyper_file.display());
        }

        // Compare injections
        if expected_json.exists() {
            let expected: ExpectedOutput = serde_json::from_str(
                &fs::read_to_string(&expected_json).unwrap()
            ).unwrap();
            assert_eq!(result.injections, expected.injections,
                "Injections mismatch for {}", hyper_file.display());
        }
    }
}

#[test]
fn test_all_errors() {
    for hyper_file in glob("tests/errors/*.hyper").unwrap() {
        let source = fs::read_to_string(&hyper_file).unwrap();
        let expected_err = hyper_file.with_extension("expected.err");

        let result = pipeline.compile(&source, &opts);
        assert!(result.is_err(), "Expected error for {}", hyper_file.display());

        if expected_err.exists() {
            let expected = fs::read_to_string(&expected_err).unwrap();
            let actual = result.unwrap_err().render();
            assert_eq!(actual.trim(), expected.trim());
        }
    }
}
```

#### Phase 3: Accept/update workflow

Create a `just` recipe or binary:

```bash
# just accept-expected
# Regenerates all .expected.* files from current transpiler output

# just diff-expected
# Shows diff between expected and actual (like git diff)

# just test-expected
# Runs the comparison tests
```

```rust
// tools/accept_expected.rs (standalone binary)
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filter = args.get(1); // Optional: specific test name

    for hyper_file in glob("tests/**/*.hyper") {
        if let Some(f) = filter {
            if !hyper_file.to_str().unwrap().contains(f) { continue; }
        }

        let result = pipeline.compile(&source, &opts);

        match result {
            Ok(output) => {
                fs::write(py_path, &output.code);
                if !output.injections.is_empty() {
                    fs::write(json_path, serde_json::to_string_pretty(&output)?);
                }
            }
            Err(e) => {
                fs::write(err_path, e.render());
            }
        }
    }
}
```

#### Phase 4: Cleanup

1. Delete `snapshots/` directory
2. Delete insta dependency from `Cargo.toml`
3. Remove `golden_tests.rs`
4. Update `justfile` recipes

---

## Injection Testing Format

### `.expected.json` structure

```json
{
  "injections": [
    {
      "type": "python",
      "start": 0,
      "end": 15,
      "prefix": "def Foo(",
      "suffix": ") -> str:"
    },
    {
      "type": "html",
      "start": 20,
      "end": 25,
      "prefix": "",
      "suffix": ""
    }
  ],
  "ranges": [
    {
      "range_type": "Python",
      "source_start": 0,
      "source_end": 15,
      "compiled_start": 8,
      "compiled_end": 23
    }
  ]
}
```

### Validation checks (in test runner)

```rust
fn validate_injections(injections: &[Injection], source: &str) {
    for inj in injections {
        assert!(inj.start <= inj.end, "start > end");
        assert!(inj.end <= source.len(), "end out of bounds");

        // Non-overlapping check
        for other in injections {
            if inj != other {
                let overlaps = inj.start < other.end && other.start < inj.end;
                assert!(!overlaps, "Overlapping injections");
            }
        }
    }
}
```

---

## Migration Checklist

- [ ] Commit/branch current injection work
- [ ] Create `accept_expected` binary
- [ ] Generate initial `.expected.py` files from current snapshots
- [ ] Generate initial `.expected.json` files
- [ ] Create new test runner (`expected_tests.rs`)
- [ ] Add `just` recipes for accept/diff/test
- [ ] Verify all tests pass with new system
- [ ] Remove insta dependency and snapshots
- [ ] Merge injection work branch
- [ ] Run full test suite to confirm everything works
