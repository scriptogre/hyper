use clap::{Parser, Subcommand};
use hyper_transpiler::{transpile_with, Options};
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
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
        /// Path to .hyper file or directory
        #[arg(required_unless_present = "stdin")]
        file: Option<PathBuf>,

        /// Read from stdin
        #[arg(long)]
        stdin: bool,

        /// Output as JSON with source mappings
        #[arg(long)]
        json: bool,

        /// Include injection pieces for IDE integration
        #[arg(long)]
        injection: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { file, stdin, json, injection } => {
            if stdin {
                generate_stdin(json, injection);
            } else if let Some(path) = file {
                generate_path(&path);
            } else {
                eprintln!("Error: provide a file/directory or use --stdin");
                std::process::exit(1);
            }
        }
    }
}

fn generate_stdin(json_output: bool, include_injections: bool) {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source).expect("Failed to read stdin");

    let options = Options {
        include_injections,
        ..Options::default()
    };
    let result = transpile_with(&source, options);

    if json_output {
        println!("{}", serde_json::to_string(&result).unwrap());
    } else {
        print!("{}", result.code);
    }
}

fn generate_path(path: &PathBuf) {
    if path.is_file() {
        if path.extension().map_or(true, |ext| ext != "hyper") {
            eprintln!("Error: {} is not a .hyper file", path.display());
            std::process::exit(1);
        }
        let start = Instant::now();
        generate_file(path);
        let elapsed = start.elapsed();
        print_summary(1, elapsed);
    } else if path.is_dir() {
        generate_directory(path);
    } else {
        eprintln!("Error: {} does not exist", path.display());
        std::process::exit(1);
    }
}

fn generate_directory(dir: &PathBuf) {
    use std::collections::HashMap;

    let start = Instant::now();

    // Map: directory -> list of component names generated in that directory
    let mut components_by_dir: HashMap<PathBuf, Vec<String>> = HashMap::new();
    let mut file_count = 0;

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "hyper"))
    {
        let path = entry.path();
        if let Some(component_name) = generate_file(path) {
            file_count += 1;
            if let Some(parent) = path.parent() {
                components_by_dir
                    .entry(parent.to_path_buf())
                    .or_default()
                    .push(component_name);
            }
        }
    }

    if file_count == 0 {
        eprintln!("No .hyper files found in {}", dir.display());
        std::process::exit(1);
    }

    // Generate __init__.py for each directory that has components
    for (dir_path, mut components) in components_by_dir {
        components.sort();
        let init_path = dir_path.join("__init__.py");

        let imports: Vec<String> = components
            .iter()
            .map(|name| format!("from .{name} import {name}"))
            .collect();

        let all_list: Vec<String> = components
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect();

        let content = format!(
            "{}\n\n__all__ = [{}]\n",
            imports.join("\n"),
            all_list.join(", ")
        );

        fs::write(&init_path, content).expect("Failed to write __init__.py");
        print_generated(&init_path.display().to_string());
    }

    let elapsed = start.elapsed();
    print_summary(file_count, elapsed);
}

fn generate_file(path: &std::path::Path) -> Option<String> {
    let source = fs::read_to_string(path).expect("Failed to read file");

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Template");

    let options = Options {
        function_name: name.to_string(),
        include_injections: false,
    };
    let result = transpile_with(&source, options);

    let output = path.with_extension("py");
    fs::write(&output, &result.code).expect("Failed to write file");
    print_generated(&output.display().to_string());

    Some(name.to_string())
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
