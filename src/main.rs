use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use mermaid_cli::{check, extract_mermaid_blocks, Fixer};

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
    let mut theme = "default".to_string();
    let mut width = 800;
    let mut height = 600;
    let mut scale = 1.0;
    let mut background_color = "white".to_string();
    let mut quiet = false;
    let mut svg_id: Option<String> = None;
    let mut css_file: Option<String> = None;
    let mut output_format: Option<String> = None;
    let mut _jobs: usize = 1;

    // 解析选项
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-e" | "--outputFormat" => {
                if i + 1 < args.len() {
                    output_format = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --outputFormat requires an argument");
                    std::process::exit(1);
                }
            }
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: -o requires an argument");
                    std::process::exit(1);
                }
            }
            "-C" | "--cssFile" => {
                if i + 1 < args.len() {
                    css_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --cssFile requires an argument");
                    std::process::exit(1);
                }
            }
            "-t" | "--theme" => {
                if i + 1 < args.len() {
                    theme = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --theme requires an argument");
                    std::process::exit(1);
                }
            }
            "-w" | "--width" => {
                if i + 1 < args.len() {
                    width = args[i + 1].parse().unwrap_or(800);
                    i += 2;
                } else {
                    eprintln!("Error: --width requires an argument");
                    std::process::exit(1);
                }
            }
            "-H" | "--height" => {
                if i + 1 < args.len() {
                    height = args[i + 1].parse().unwrap_or(600);
                    i += 2;
                } else {
                    eprintln!("Error: --height requires an argument");
                    std::process::exit(1);
                }
            }
            "-s" | "--scale" => {
                if i + 1 < args.len() {
                    scale = args[i + 1].parse().unwrap_or(1.0);
                    i += 2;
                } else {
                    eprintln!("Error: --scale requires an argument");
                    std::process::exit(1);
                }
            }
            "-b" | "--backgroundColor" => {
                if i + 1 < args.len() {
                    background_color = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --backgroundColor requires an argument");
                    std::process::exit(1);
                }
            }
            "-q" | "--quiet" => {
                quiet = true;
                i += 1;
            }
            "-I" | "--svgId" => {
                if i + 1 < args.len() {
                    svg_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --svgId requires an argument");
                    std::process::exit(1);
                }
            }
            "-j" | "--jobs" => {
                if i + 1 < args.len() {
                    _jobs = args[i + 1].parse::<usize>().unwrap_or_else(|_| {
                        eprintln!("Error: --jobs requires a number");
                        std::process::exit(1);
                    });
                    i += 2;
                } else {
                    eprintln!("Error: --jobs requires an argument");
                    std::process::exit(1);
                }
            }
            "--iconPacks" => {
                eprintln!("Warning: --iconPacks is experimental");
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    let code = match fs::read_to_string(&input_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file {}: {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    // 检测是否为 Markdown 文件 (.md / .markdown)
    let is_markdown = input_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "md" || ext == "markdown")
        .unwrap_or(false);

    // 对 Markdown 文件提取 mermaid 代码块
    let code = if is_markdown {
        let blocks = extract_mermaid_blocks(&code);
        if blocks.is_empty() {
            eprintln!(
                "Error: No mermaid code blocks found in {}",
                input_path.display()
            );
            std::process::exit(1);
        }
        // 渲染第一个代码块，如果有多个则提示
        if blocks.len() > 1 {
            eprintln!(
                "Note: {} mermaid blocks found, rendering the first one",
                blocks.len()
            );
        }
        blocks.into_iter().next().unwrap()
    } else {
        code
    };

    if show_fixes && !quiet {
        let (_fixed, fixes) = Fixer::new().fix(&code);
        if fixes.is_empty() {
            eprintln!("✓ No issues found");
        } else {
            eprintln!("{} fix(es) available:", fixes.len());
            for f in &fixes {
                eprintln!("  • {}", f);
            }
        }
    }

    let mut renderer = mermaid_cli::Renderer::with_dimensions(width, height)
        .with_theme(&theme)
        .with_background_color(&background_color)
        .with_scale(scale);

    if let Some(ref css_path) = css_file {
        match fs::read_to_string(css_path) {
            Ok(css) => {
                renderer = renderer.with_custom_css(&css);
            }
            Err(e) => {
                eprintln!("Error reading CSS file {}: {}", css_path, e);
                std::process::exit(1);
            }
        }
    }

    let diagram = match mermaid_cli::parse(&code) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // 检测输出格式
    let fmt = output_format
        .as_deref()
        .or_else(|| output_path.as_ref().and_then(|p| p.extension()?.to_str()))
        .unwrap_or("svg");

    if fmt == "png" {
        #[cfg(feature = "png")]
        {
            match mermaid_cli::render_png(&code, width, height, scale) {
                Ok(png_bytes) => {
                    if let Some(output) = output_path {
                        match fs::write(&output, &png_bytes) {
                            Ok(_) => {
                                if !quiet {
                                    println!("✓ Rendered PNG to: {}", output.display());
                                }
                            }
                            Err(e) => {
                                eprintln!("Error writing PNG: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error rendering PNG: {}", e);
                    std::process::exit(1);
                }
            }
        }
        #[cfg(not(feature = "png"))]
        {
            eprintln!(
                "PNG output requires building with --features png. Use SVG output or rebuild."
            );
            std::process::exit(1);
        }
    } else {
        match renderer.render(&diagram) {
            Ok(mut svg) => {
                if let Some(ref id) = svg_id {
                    svg = filter_svg_by_id(&svg, id);
                }
                if let Some(output) = output_path {
                    match fs::write(&output, &svg) {
                        Ok(_) => {
                            if !quiet {
                                println!("✓ Rendered to: {}", output.display());
                            }
                        }
                        Err(e) => {
                            eprintln!("Error writing to {}: {}", output.display(), e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    print!("{}", svg);
                }
            }
            Err(e) => {
                eprintln!("Error rendering: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn handle_stdin_render(args: &[String], show_fixes: bool) {
    let mut code = String::new();
    let mut output_path: Option<PathBuf> = None;
    let mut theme = "default".to_string();
    let mut width = 800;
    let mut height = 600;
    let mut scale = 1.0;
    let mut background_color = "white".to_string();
    let mut quiet = false;
    let mut svg_id: Option<String> = None;
    let mut css_file: Option<String> = None;
    let output_format: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-C" | "--cssFile" => {
                if i + 1 < args.len() {
                    css_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --cssFile requires an argument");
                    std::process::exit(1);
                }
            }
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: -o requires an argument");
                    std::process::exit(1);
                }
            }
            "-t" | "--theme" => {
                if i + 1 < args.len() {
                    theme = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --theme requires an argument");
                    std::process::exit(1);
                }
            }
            "-w" | "--width" => {
                if i + 1 < args.len() {
                    width = args[i + 1].parse().unwrap_or(800);
                    i += 2;
                } else {
                    eprintln!("Error: --width requires an argument");
                    std::process::exit(1);
                }
            }
            "-H" | "--height" => {
                if i + 1 < args.len() {
                    height = args[i + 1].parse().unwrap_or(600);
                    i += 2;
                } else {
                    eprintln!("Error: --height requires an argument");
                    std::process::exit(1);
                }
            }
            "-s" | "--scale" => {
                if i + 1 < args.len() {
                    scale = args[i + 1].parse().unwrap_or(1.0);
                    i += 2;
                } else {
                    eprintln!("Error: --scale requires an argument");
                    std::process::exit(1);
                }
            }
            "-b" | "--backgroundColor" => {
                if i + 1 < args.len() {
                    background_color = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --backgroundColor requires an argument");
                    std::process::exit(1);
                }
            }
            "-q" | "--quiet" => {
                quiet = true;
                i += 1;
            }
            "-I" | "--svgId" => {
                if i + 1 < args.len() {
                    svg_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --svgId requires an argument");
                    std::process::exit(1);
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    if let Err(e) = io::stdin().read_to_string(&mut code) {
        eprintln!("Error reading from stdin: {}", e);
        std::process::exit(1);
    }

    // 自动检测 stdin 是否为 Markdown（包含 ```mermaid 标记）
    if code.contains("```mermaid") {
        let blocks = extract_mermaid_blocks(&code);
        if blocks.is_empty() {
            eprintln!("Error: No mermaid code blocks found in stdin (detected markdown)");
            std::process::exit(1);
        }
        if blocks.len() > 1 {
            eprintln!(
                "Note: {} mermaid blocks found, rendering the first one",
                blocks.len()
            );
        }
        code = blocks.into_iter().next().unwrap();
    }

    if show_fixes && !quiet {
        let (_fixed, fixes) = Fixer::new().fix(&code);
        if fixes.is_empty() {
            eprintln!("✓ No issues found");
        } else {
            eprintln!("{} fix(es) available:", fixes.len());
            for f in &fixes {
                eprintln!("  • {}", f);
            }
        }
    }

    let mut renderer = mermaid_cli::Renderer::with_dimensions(width, height)
        .with_theme(&theme)
        .with_background_color(&background_color)
        .with_scale(scale);

    if let Some(ref css_path) = css_file {
        match fs::read_to_string(css_path) {
            Ok(css) => {
                renderer = renderer.with_custom_css(&css);
            }
            Err(e) => {
                eprintln!("Error reading CSS file {}: {}", css_path, e);
                std::process::exit(1);
            }
        }
    }

    let diagram = match mermaid_cli::parse(&code) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };

    // 检测输出格式
    let fmt = output_format
        .as_deref()
        .or_else(|| output_path.as_ref().and_then(|p| p.extension()?.to_str()))
        .unwrap_or("svg");

    if fmt == "png" {
        #[cfg(feature = "png")]
        {
            match mermaid_cli::render_png(&code, width, height, scale) {
                Ok(png_bytes) => {
                    if let Some(output) = output_path {
                        match fs::write(&output, &png_bytes) {
                            Ok(_) => {
                                if !quiet {
                                    println!("✓ Rendered PNG to: {}", output.display());
                                }
                            }
                            Err(e) => {
                                eprintln!("Error writing PNG: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error rendering PNG: {}", e);
                    std::process::exit(1);
                }
            }
        }
        #[cfg(not(feature = "png"))]
        {
            eprintln!(
                "PNG output requires building with --features png. Use SVG output or rebuild."
            );
            std::process::exit(1);
        }
    } else {
        match renderer.render(&diagram) {
            Ok(mut svg) => {
                if let Some(ref id) = svg_id {
                    svg = filter_svg_by_id(&svg, id);
                }
                if let Some(output) = output_path {
                    match fs::write(&output, &svg) {
                        Ok(_) => {
                            if !quiet {
                                println!("✓ Rendered to: {}", output.display());
                            }
                        }
                        Err(e) => {
                            eprintln!("Error writing to {}: {}", output.display(), e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    print!("{}", svg);
                }
            }
            Err(e) => {
                eprintln!("Error rendering: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// Filter SVG to only include elements with a specific ID pattern.
fn filter_svg_by_id(svg: &str, id: &str) -> String {
    let mut result = String::new();
    let _in_element = false;
    let mut current_element = String::new();
    let _keep = false;

    for line in svg.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<svg") || trimmed.starts_with("<?xml") {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        if trimmed == "</svg>" {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        // Check if element contains the id (as part of class or id attribute)
        if trimmed.contains(&format!("id=\"{}\"", id)) || trimmed.contains(&format!("#{}", id)) {
            result.push_str(line);
            result.push('\n');
        } else if trimmed.starts_with('<')
            && !trimmed.starts_with("</")
            && !trimmed.starts_with("<?")
        {
            current_element.push_str(line);
            current_element.push('\n');
        }
    }

    result
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

/// Render multiple diagram files in parallel using a thread pool.
/// Each file is processed independently and written to its output path.
#[allow(dead_code)]
fn render_files_parallel(
    files: &[PathBuf],
    outputs: &[PathBuf],
    config: &RenderConfig,
    num_jobs: usize,
    quiet: bool,
) {
    let num_jobs = num_jobs.max(1);
    let (tx, rx) = mpsc::channel();

    // Process in chunks
    let chunk_size = (files.len() + num_jobs - 1) / num_jobs;

    for chunk in files.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let outputs = outputs.to_vec();
        let tx = tx.clone();
        let config = config.clone();

        thread::spawn(move || {
            let renderer = mermaid_cli::Renderer::with_dimensions(config.width, config.height)
                .with_theme(&config.theme)
                .with_background_color(&config.background_color)
                .with_scale(config.scale);

            for (i, file) in chunk.iter().enumerate() {
                let out_path = if i < outputs.len() { &outputs[i] } else { file };
                let content = match fs::read_to_string(file) {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(format!("Error reading {}: {}", file.display(), e));
                        continue;
                    }
                };

                let code = if content.contains("```mermaid") {
                    let blocks = extract_mermaid_blocks(&content);
                    if blocks.is_empty() {
                        let _ = tx.send(format!("No mermaid blocks in {}", file.display()));
                        continue;
                    }
                    blocks.into_iter().next().unwrap()
                } else {
                    content
                };

                let diagram = match mermaid_cli::parse(&code) {
                    Ok(d) => d,
                    Err(e) => {
                        let _ = tx.send(format!("Parse error {}: {:?}", file.display(), e));
                        continue;
                    }
                };

                match renderer.render(&diagram) {
                    Ok(svg) => {
                        if let Err(e) = fs::write(out_path, &svg) {
                            let _ = tx.send(format!("Write error {}: {}", out_path.display(), e));
                        } else if !quiet {
                            let _ =
                                tx.send(format!("✓ {} -> {}", file.display(), out_path.display()));
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(format!("Render error {}: {}", file.display(), e));
                    }
                }
            }
        });
    }

    drop(tx);
    for msg in rx {
        println!("{}", msg);
    }
}

/// Configuration for rendering (cloneable for parallel dispatch).
#[derive(Clone)]
#[allow(dead_code)]
struct RenderConfig {
    width: u32,
    height: u32,
    theme: String,
    background_color: String,
    scale: f32,
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
    -o, --output <FILE>       Output file path (default: stdout)
    -t, --theme <NAME>        Theme: default/forest/dark/neutral (default: default)
    -w, --width <PX>          SVG width in pixels (default: 800)
    -H, --height <PX>         SVG height in pixels (default: 600)
    -s, --scale <FACTOR>      Scale factor for PNG output (default: 1)
    -b, --backgroundColor <C> Background color (default: white)
    -e, --outputFormat <FMT>  Output format: svg or png (default: auto-detect from extension)
    -C, --cssFile <FILE>      Path to custom CSS file for SVG styling
    -q, --quiet               Suppress non-error output (default: off)
    -j, --jobs <N>            Number of parallel render jobs (default: 1)
    -I, --svgId <ID>          Filter SVG to retain elements matching ID
    --show-fixes              Show available fixes before rendering
    --iconPacks <PKGS>        Iconify icon packs (experimental)
    --json                    Output AST as JSON (requires --features json)

EXAMPLES:
    # Render from file
    mermaid-cli input.mmd -o output.svg

    # Render with theme
    mermaid-cli input.mmd -t dark -o output.svg

    # Render with custom dimensions
    mermaid-cli input.mmd -w 1024 -H 768 -o output.svg

    # Quiet mode (no output messages)
    mermaid-cli input.mmd -o output.svg -q

    # Filter SVG by ID
    mermaid-cli input.mmd --svgId mynode -o output.svg

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
