//! Mermaid Linter CLI
//!
//! A command-line tool for linting Mermaid diagrams.

use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use mermaid_linter::{parse, validate, detect_type, ParseResult};

/// Mermaid diagram syntax linter
#[derive(Parser)]
#[command(name = "mermaid-lint")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input file(s) to lint (reads from stdin if not provided)
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Only validate, don't output AST
    #[arg(short = 'c', long)]
    check: bool,

    /// Suppress output, only return exit code
    #[arg(short, long)]
    quiet: bool,

    /// Show AST output
    #[arg(long)]
    ast: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint Mermaid diagram files
    Lint {
        /// Input files
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Detect diagram type
    Detect {
        /// Input file (reads from stdin if not provided)
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
    },

    /// Validate diagram syntax
    Check {
        /// Input files
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,
    },

    /// Parse and output AST
    Parse {
        /// Input file (reads from stdin if not provided)
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Output format (json, yaml)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let exit_code = match cli.command {
        Some(Commands::Lint { files, format }) => lint_files(&files, &format, false),
        Some(Commands::Detect { file }) => detect_file(file),
        Some(Commands::Check { files }) => check_files(&files),
        Some(Commands::Parse { file, format }) => parse_file(file, &format),
        None => {
            if cli.files.is_empty() {
                // Read from stdin
                lint_stdin(&cli.format, cli.check, cli.quiet, cli.ast)
            } else {
                lint_files(&cli.files, &cli.format, cli.quiet)
            }
        }
    };

    process::exit(exit_code);
}

fn lint_files(files: &[PathBuf], format: &str, quiet: bool) -> i32 {
    let mut has_errors = false;

    for file in files {
        match fs::read_to_string(file) {
            Ok(content) => {
                let result = parse(&content, None);
                has_errors |= !result.ok;

                if !quiet {
                    print_result(file.to_string_lossy().as_ref(), &result, format, &content);
                }
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
                has_errors = true;
            }
        }
    }

    if has_errors { 1 } else { 0 }
}

fn lint_stdin(format: &str, check_only: bool, quiet: bool, show_ast: bool) -> i32 {
    let mut content = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut content) {
        eprintln!("Error reading stdin: {}", e);
        return 1;
    }

    if check_only {
        let valid = validate(&content, None);
        if !quiet {
            if valid {
                println!("Valid");
            } else {
                println!("Invalid");
            }
        }
        return if valid { 0 } else { 1 };
    }

    let result = parse(&content, None);

    if !quiet {
        print_result("<stdin>", &result, format, &content);

        if show_ast && result.ok {
            if let Some(ast) = &result.ast {
                println!("\nAST:");
                println!("{}", serde_json::to_string_pretty(ast).unwrap_or_default());
            }
        }
    }

    if result.ok { 0 } else { 1 }
}

fn detect_file(file: Option<PathBuf>) -> i32 {
    let content = match file {
        Some(path) => match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", path.display(), e);
                return 1;
            }
        },
        None => {
            let mut content = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut content) {
                eprintln!("Error reading stdin: {}", e);
                return 1;
            }
            content
        }
    };

    match detect_type(&content) {
        Some(diagram_type) => {
            println!("{}", diagram_type);
            0
        }
        None => {
            println!("unknown");
            1
        }
    }
}

fn check_files(files: &[PathBuf]) -> i32 {
    let mut has_errors = false;

    for file in files {
        match fs::read_to_string(file) {
            Ok(content) => {
                let valid = validate(&content, None);
                if valid {
                    println!("{}: OK", file.display());
                } else {
                    println!("{}: FAIL", file.display());
                    has_errors = true;
                }
            }
            Err(e) => {
                eprintln!("{}: ERROR - {}", file.display(), e);
                has_errors = true;
            }
        }
    }

    if has_errors { 1 } else { 0 }
}

fn parse_file(file: Option<PathBuf>, format: &str) -> i32 {
    let content = match file {
        Some(path) => match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", path.display(), e);
                return 1;
            }
        },
        None => {
            let mut content = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut content) {
                eprintln!("Error reading stdin: {}", e);
                return 1;
            }
            content
        }
    };

    let result = parse(&content, None);

    if !result.ok {
        for diag in &result.diagnostics {
            eprintln!("{}", diag.format(&content));
        }
        return 1;
    }

    if let Some(ast) = &result.ast {
        let output = match format {
            "yaml" => serde_yaml::to_string(ast).unwrap_or_default(),
            _ => serde_json::to_string_pretty(ast).unwrap_or_default(),
        };
        println!("{}", output);
    }

    0
}

fn print_result(file: &str, result: &ParseResult, format: &str, source: &str) {
    match format {
        "json" => {
            let output = serde_json::json!({
                "file": file,
                "ok": result.ok,
                "diagram_type": result.diagram_type.map(|t| t.as_str()),
                "title": result.title,
                "diagnostics": result.diagnostics.iter().map(|d| {
                    serde_json::json!({
                        "code": d.code.as_str(),
                        "message": d.message,
                        "severity": d.severity.as_str(),
                        "range": {
                            "start": d.span.start,
                            "end": d.span.end,
                        }
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
        }
        _ => {
            // Text format
            if result.ok {
                println!("{}: OK", file);
                if let Some(diagram_type) = result.diagram_type {
                    println!("  Type: {}", diagram_type);
                }
                if let Some(title) = &result.title {
                    println!("  Title: {}", title);
                }
            } else {
                println!("{}: FAIL", file);
                for diag in &result.diagnostics {
                    println!("{}", diag.format(source));
                }
            }
        }
    }
}
