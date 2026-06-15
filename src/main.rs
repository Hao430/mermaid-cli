use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use mermaid_cli::render;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            print_help();
        }
        "--version" | "-V" => {
            println!("mermaid-cli 0.1.0-alpha");
        }
        "--stdin" => {
            handle_stdin_render(&args);
        }
        _ => {
            handle_file_render(&args);
        }
    }
}

fn handle_file_render(args: &[String]) {
    let input_path = PathBuf::from(&args[1]);
    let mut output_path: Option<PathBuf> = None;

    // 解析 -o 或 --output 选项
    let mut i = 2;
    while i < args.len() {
        if args[i] == "-o" || args[i] == "--output" {
            if i + 1 < args.len() {
                output_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            } else {
                eprintln!("Error: -o requires an argument");
                std::process::exit(1);
            }
        } else {
            i += 1;
        }
    }

    match fs::read_to_string(&input_path) {
        Ok(code) => match render(&code) {
            Ok(svg) => {
                if let Some(output) = output_path {
                    match fs::write(&output, &svg) {
                        Ok(_) => {
                            println!("✓ Rendered to: {}", output.display());
                        }
                        Err(e) => {
                            eprintln!("Error writing to {}: {}", output.display(), e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    println!("{}", svg);
                }
            }
            Err(e) => {
                eprintln!("Error rendering: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error reading file {}: {}", input_path.display(), e);
            std::process::exit(1);
        }
    }
}

fn handle_stdin_render(args: &[String]) {
    let mut code = String::new();
    let mut output_path: Option<PathBuf> = None;

    // 解析 -o 或 --output 选项
    let mut i = 2;
    while i < args.len() {
        if args[i] == "-o" || args[i] == "--output" {
            if i + 1 < args.len() {
                output_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            } else {
                eprintln!("Error: -o requires an argument");
                std::process::exit(1);
            }
        } else {
            i += 1;
        }
    }

    match io::stdin().read_to_string(&mut code) {
        Ok(_) => match render(&code) {
            Ok(svg) => {
                if let Some(output) = output_path {
                    match fs::write(&output, &svg) {
                        Ok(_) => {
                            println!("✓ Rendered to: {}", output.display());
                        }
                        Err(e) => {
                            eprintln!("Error writing to {}: {}", output.display(), e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    println!("{}", svg);
                }
            }
            Err(e) => {
                eprintln!("Error rendering: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        r#"mermaid-cli 0.1.0-alpha
High-performance Mermaid diagram CLI

USAGE:
    mermaid-cli [COMMAND] [OPTIONS] [INPUT]

COMMANDS:
    <FILE>              Render file to SVG (default)
    --stdin             Read from standard input
    --help              Show this help message
    --version           Show version

OPTIONS:
    -o, --output <FILE> Output file path (default: stdout)

EXAMPLES:
    # Render from file
    mermaid-cli input.mmd -o output.svg

    # Render from stdin
    echo 'graph TD; A-->B' | mermaid-cli --stdin -o output.svg

    # Output to stdout
    mermaid-cli input.mmd
"#
    );
}
