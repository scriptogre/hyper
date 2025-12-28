use clap::{Parser, Subcommand};
use hyper_transpiler::{transpile, transpile_ext};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
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

fn generate_stdin(json_output: bool, include_injection: bool) {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source).expect("Failed to read stdin");

    let result = transpile_ext(&source, include_injection);

    if json_output {
        println!("{}", serde_json::to_string(&result).unwrap());
    } else {
        print!("{}", result.python_code);
    }
}

fn generate_path(path: &PathBuf) {
    if path.is_file() {
        if path.extension().map_or(true, |ext| ext != "hyper") {
            eprintln!("Error: {} is not a .hyper file", path.display());
            std::process::exit(1);
        }
        generate_file(path);
    } else if path.is_dir() {
        let mut found = false;
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "hyper"))
        {
            found = true;
            generate_file(entry.path());
        }
        if !found {
            eprintln!("No .hyper files found in {}", path.display());
            std::process::exit(1);
        }
    } else {
        eprintln!("Error: {} does not exist", path.display());
        std::process::exit(1);
    }
}

fn generate_file(path: &std::path::Path) {
    let source = fs::read_to_string(path).expect("Failed to read file");
    let result = transpile(&source);
    let output = path.with_extension("py");
    fs::write(&output, &result.python_code).expect("Failed to write file");
    println!("Generated {}", output.display());
}