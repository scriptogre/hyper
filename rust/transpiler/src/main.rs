use clap::{Parser, Subcommand};
use hyper_transpiler::{Pipeline, GenerateOptions};
use std::io::{self, Read, IsTerminal};
use std::fs;
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "hyper")]
#[command(about = "Hyper - Python templates with HTML and control flow")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Python from .hyper files
    Generate {
        /// .hyper files to transpile (if none specified, finds all in current directory)
        files: Vec<String>,

        /// Read from stdin
        #[arg(long)]
        stdin: bool,

        /// Output as JSON with source mappings
        #[arg(long)]
        json: bool,

        /// Include injection pieces for IDE integration
        #[arg(long)]
        injection: bool,

        /// Function name (defaults to "Template")
        #[arg(long)]
        name: Option<String>,

        /// Run as daemon: read length-prefixed messages from stdin
        #[arg(long)]
        daemon: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { files, stdin, json, injection, name, daemon } => {
            if daemon {
                run_daemon();
            } else if stdin {
                generate_stdin(json, injection, name);
            } else {
                generate_files(files, json, injection, name);
            }
        }
    }
}

fn generate_stdin(json_output: bool, include_injections: bool, name: Option<String>) {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source).expect("Failed to read stdin");

    let options = GenerateOptions {
        function_name: name,
        include_ranges: include_injections,
    };

    let mut pipeline = Pipeline::standard();
    let result = match pipeline.compile(&source, &options) {
        Ok(r) => r,
        Err(e) => {
            if json_output {
                println!(r#"{{"error":"{}"}}"#, e.to_string().replace('"', "\\\""));
            } else {
                if io::stderr().is_terminal() {
                    eprint!("{}", e.render_color(&source, "stdin"));
                } else {
                    eprint!("{}", e.render(&source, "stdin"));
                }
            }
            std::process::exit(1);
        }
    };

    if json_output {
        let output = DaemonResponse {
            compiled: result.code,
            mappings: result.mappings.into_iter().map(|m| DaemonMapping {
                gen_line: m.gen_line,
                gen_col: m.gen_col,
                src_line: m.src_line,
                src_col: m.src_col,
            }).collect(),
            ranges: if include_injections {
                Some(result.ranges.into_iter().map(|r| DaemonRange {
                    range_type: format!("{:?}", r.range_type).to_lowercase(),
                    source_start: r.source_start,
                    source_end: r.source_end,
                    compiled_start: r.compiled_start,
                    compiled_end: r.compiled_end,
                }).collect())
            } else {
                None
            },
            injections: if include_injections {
                Some(result.injections.into_iter().map(|i| DaemonInjection {
                    injection_type: i.injection_type,
                    start: i.start,
                    end: i.end,
                    prefix: i.prefix,
                    suffix: i.suffix,
                }).collect())
            } else {
                None
            },
        };
        println!("{}", serde_json::to_string(&output).unwrap());
    } else {
        print!("{}", result.code);
    }
}

fn generate_files(files: Vec<String>, _json_output: bool, _include_injections: bool, _name: Option<String>) {
    let start = Instant::now();

    let files_to_process: Vec<String> = if files.is_empty() {
        // Recursively discover all .hyper files starting from current directory
        discover_hyper_files(".")
    } else {
        let mut result = Vec::new();
        for arg in &files {
            let path = Path::new(arg);
            if path.is_dir() {
                result.extend(discover_hyper_files(arg));
            } else {
                result.push(arg.clone());
            }
        }
        result
    };

    if files_to_process.is_empty() {
        eprintln!("No .hyper files found");
        std::process::exit(1);
    }

    let mut has_errors = false;
    let mut success_count = 0;

    for file_path in files_to_process {
        let source = match fs::read_to_string(&file_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading {}: {}", file_path, e);
                has_errors = true;
                continue;
            }
        };

        // Extract function name from filename
        let function_name = Path::new(&file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        let options = GenerateOptions {
            function_name,
            include_ranges: false,
        };

        let mut pipeline = Pipeline::standard();
        let result = match pipeline.compile(&source, &options) {
            Ok(r) => r,
            Err(e) => {
                if io::stderr().is_terminal() {
                    eprint!("{}", e.render_color(&source, &file_path));
                } else {
                    eprint!("{}", e.render(&source, &file_path));
                }
                has_errors = true;
                continue;
            }
        };

        // Write to .py file
        let output_path = Path::new(&file_path).with_extension("py");
        if let Err(e) = fs::write(&output_path, &result.code) {
            eprintln!("Error writing {}: {}", output_path.display(), e);
            has_errors = true;
            continue;
        }

        print_generated(&output_path.to_string_lossy());
        success_count += 1;
    }

    if success_count > 0 {
        let elapsed = start.elapsed();
        print_summary(success_count, elapsed);
    }

    if has_errors {
        std::process::exit(1);
    }
}

fn discover_hyper_files(dir: &str) -> Vec<String> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "hyper"))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect()
}

fn print_generated(path: &str) {
    let is_tty = io::stderr().is_terminal();
    if is_tty {
        eprintln!("  \x1b[32m✓\x1b[0m {}", path);
    } else {
        eprintln!("  ✓ {}", path);
    }
}

fn print_summary(count: usize, elapsed: std::time::Duration) {
    let is_tty = io::stderr().is_terminal();
    let time_str = format_duration(elapsed);
    let files_word = if count == 1 { "file" } else { "files" };

    if is_tty {
        eprintln!("\n\x1b[1m✨ Generated {} {} in {}\x1b[0m", count, files_word, time_str);
    } else {
        eprintln!("\n✨ Generated {} {} in {}", count, files_word, time_str);
    }
}

fn format_duration(d: std::time::Duration) -> String {
    let micros = d.as_micros();
    if micros < 1000 {
        format!("{}μs", micros)
    } else if micros < 1_000_000 {
        format!("{:.1}ms", micros as f64 / 1000.0)
    } else {
        format!("{:.2}s", d.as_secs_f64())
    }
}

/// Run daemon mode for IDE integration
///
/// Protocol:
///   Request:  <4-byte big-endian length><JSON payload>
///   Response: <4-byte big-endian length><JSON payload>
///
/// Request JSON: {"content": "...", "injection": bool, "name": "..."}
/// Response JSON: Same as normal --json output
fn run_daemon() {
    use std::io::{stdin, stdout, Write};

    let stdin = stdin();
    let mut stdin = stdin.lock();
    let stdout = stdout();
    let mut stdout = stdout.lock();

    // Signal ready
    let ready = b"{\"ready\":true}\n";
    let len = (ready.len() as u32).to_be_bytes();
    if let Err(e) = stdout.write_all(&len) {
        eprintln!("[DAEMON] Failed to write ready length: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = stdout.write_all(ready) {
        eprintln!("[DAEMON] Failed to write ready message: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = stdout.flush() {
        eprintln!("[DAEMON] Failed to flush stdout: {}", e);
        std::process::exit(1);
    }

    eprintln!("[DAEMON] Ready message sent successfully, entering main loop");

    loop {
        // Read 4-byte length prefix
        let mut len_buf = [0u8; 4];
        match stdin.read_exact(&mut len_buf) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Daemon exiting: failed to read length prefix: {}", e);
                break; // EOF or error
            }
        }

        let msg_len = u32::from_be_bytes(len_buf) as usize;
        if msg_len == 0 || msg_len > 10_000_000 {
            eprintln!("Daemon exiting: invalid message length: {}", msg_len);
            break; // Invalid length
        }

        // Read JSON payload
        let mut msg_buf = vec![0u8; msg_len];
        if stdin.read_exact(&mut msg_buf).is_err() {
            eprintln!("Daemon exiting: failed to read message payload");
            break;
        }

        let response = match std::str::from_utf8(&msg_buf) {
            Ok(json_str) => process_request(json_str),
            Err(_) => r#"{"error":"Invalid UTF-8"}"#.to_string(),
        };

        // Write response with length prefix
        let response_bytes = response.as_bytes();
        let response_len = (response_bytes.len() as u32).to_be_bytes();
        if stdout.write_all(&response_len).is_err() || stdout.write_all(response_bytes).is_err() || stdout.flush().is_err() {
            eprintln!("Daemon exiting: failed to write response");
            break;
        }
    }

    eprintln!("Daemon shutdown cleanly");
}

#[derive(serde::Deserialize)]
struct DaemonRequest {
    content: String,
    #[serde(default)]
    injection: bool,
    #[serde(default)]
    name: Option<String>,
}

fn process_request(json: &str) -> String {
    let req: DaemonRequest = match serde_json::from_str(json) {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"Invalid JSON: {}"}}"#, e),
    };

    let options = GenerateOptions {
        function_name: req.name,
        include_ranges: req.injection,
    };

    let mut pipeline = Pipeline::standard();
    let result = match pipeline.compile(&req.content, &options) {
        Ok(r) => r,
        Err(e) => return format!(r#"{{"error":"{}"}}"#, e.to_string().replace('"', "\\\"")),
    };

    serde_json::to_string(&DaemonResponse {
        compiled: result.code,
        mappings: result.mappings.into_iter().map(|m| DaemonMapping {
            gen_line: m.gen_line,
            gen_col: m.gen_col,
            src_line: m.src_line,
            src_col: m.src_col,
        }).collect(),
        ranges: if req.injection {
            Some(result.ranges.into_iter().map(|r| DaemonRange {
                range_type: format!("{:?}", r.range_type).to_lowercase(),
                source_start: r.source_start,
                source_end: r.source_end,
                compiled_start: r.compiled_start,
                compiled_end: r.compiled_end,
            }).collect())
        } else {
            None
        },
        injections: if req.injection {
            Some(result.injections.into_iter().map(|i| DaemonInjection {
                injection_type: i.injection_type,
                start: i.start,
                end: i.end,
                prefix: i.prefix,
                suffix: i.suffix,
            }).collect())
        } else {
            None
        },
    }).unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e))
}

#[derive(serde::Serialize)]
struct DaemonResponse {
    compiled: String,
    mappings: Vec<DaemonMapping>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ranges: Option<Vec<DaemonRange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    injections: Option<Vec<DaemonInjection>>,
}

#[derive(serde::Serialize)]
struct DaemonMapping {
    gen_line: usize,
    gen_col: usize,
    src_line: usize,
    src_col: usize,
}

#[derive(serde::Serialize)]
struct DaemonRange {
    #[serde(rename = "type")]
    range_type: String,
    source_start: usize,
    source_end: usize,
    compiled_start: usize,
    compiled_end: usize,
}

#[derive(serde::Serialize)]
struct DaemonInjection {
    #[serde(rename = "type")]
    injection_type: String,
    start: usize,
    end: usize,
    prefix: String,
    suffix: String,
}
