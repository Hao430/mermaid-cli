use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use mermaid_cli::{check, render, Fixer};

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
        "check" => {
            handle_check(&args);
        }
        "fix" => {
            handle_fix(&args);
        }
        "--stdin" => {
            let show_fixes = args.contains(&"--show-fixes".to_string());
            handle_stdin_render(&args, show_fixes);
        }
        _ => {
            let show_fixes = args.contains(&"--show-fixes".to_string());
            handle_file_render(&args, show_fixes);
        }
    }
}

fn handle_file_render(args: &[String], show_fixes: bool) {
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

    let code = match fs::read_to_string(&input_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file {}: {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    if show_fixes {
        let (_fixed, fixes) = Fixer::new().fix(&code);
        if fixes.is_empty() {
            println!("✓ No issues found");
        } else {
            println!("{} fix(es) available:", fixes.len());
            for f in &fixes {
                println!("  • {}", f);
            }
        }
    }

    match render(&code) {
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
    }
}

fn handle_stdin_render(args: &[String], show_fixes: bool) {
    let mut code = String::new();
    let mut output_path: Option<PathBuf> = None;

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

    if let Err(e) = io::stdin().read_to_string(&mut code) {
        eprintln!("Error reading from stdin: {}", e);
        std::process::exit(1);
    }

    if show_fixes {
        let (_fixed, fixes) = Fixer::new().fix(&code);
        if fixes.is_empty() {
            println!("✓ No issues found");
        } else {
            println!("{} fix(es) available:", fixes.len());
            for f in &fixes {
                println!("  • {}", f);
            }
        }
    }

    match render(&code) {
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
    }
}

fn handle_check(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: check requires a file path");
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[2]);
    let code = match fs::read_to_string(&input_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file {}: {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    match check(&code) {
        Ok(result) => {
            if result.valid {
                println!("✓ Valid Mermaid diagram");
                std::process::exit(0);
            } else {
                println!(
                    "✗ Invalid Mermaid diagram ({} error(s)):",
                    result.errors.len()
                );
                for err in &result.errors {
                    println!("  • {}", err);
                }
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_fix(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Error: fix requires a file path");
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[2]);
    let mut output_path: Option<PathBuf> = None;

    let mut i = 3;
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

    let code = match fs::read_to_string(&input_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file {}: {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    let fixer = Fixer::new();
    let (fixed, fixes) = fixer.fix(&code);

    let output = if let Some(path) = output_path {
        path
    } else {
        // 默认输出到 stdout
        println!("{}", fixed);
        if !fixes.is_empty() {
            eprintln!("---");
            for f in &fixes {
                eprintln!("{}", f);
            }
        }
        return;
    };

    match fs::write(&output, &fixed) {
        Ok(_) => {
            if !fixes.is_empty() {
                println!("✓ Fixed {} issue(s) → {}", fixes.len(), output.display());
                for f in &fixes {
                    println!("  • {}", f);
                }
            } else {
                println!("✓ No fixes needed → {}", output.display());
            }
        }
        Err(e) => {
            eprintln!("Error writing to {}: {}", output.display(), e);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        r#"mermaid-cli 0.1.0-alpha
High-performance Mermaid diagram CLI

USAGE:
    mermaid-cli <FILE> [OPTIONS]
    mermaid-cli check <FILE>
    mermaid-cli fix <FILE> [-o <FILE>]

COMMANDS:
    <FILE>              Render file to SVG (default)
    check <FILE>        Check file for syntax errors
    fix <FILE>          Auto-fix syntax errors and output
    --stdin             Read from standard input
    --help              Show this help message
    --version           Show version

OPTIONS:
    -o, --output <FILE> Output file path (default: stdout)
    --show-fixes        Show available fixes before rendering

EXAMPLES:
    # Render from file
    mermaid-cli input.mmd -o output.svg

    # Check syntax
    mermaid-cli check input.mmd

    # Auto-fix and save
    mermaid-cli fix broken.mmd -o fixed.mmd

    # Render with fix suggestions
    echo 'grpah TD; A-->B' | mermaid-cli --stdin --show-fixes

    # Output to stdout
    mermaid-cli input.mmd
"#
    );
}
