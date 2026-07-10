"""
Mermaid CLI Python Binding

High-performance Mermaid diagram renderer for Python, powered by Rust.
"""

import subprocess
import tempfile
import os
from pathlib import Path
from typing import Optional, Literal

__version__ = "0.1.0a1"
__author__ = "Hao430"
__license__ = "MIT"

OutputFormat = Literal["svg", "png", "pdf"]
Theme = Literal["default", "forest", "dark", "neutral"]


class MermaidError(Exception):
    """Base exception for mermaid-cli errors."""
    pass


class InvalidSyntaxError(MermaidError):
    """Raised when Mermaid syntax is invalid."""
    pass


def _get_cli_path() -> str:
    """Get the path to mermaid-cli binary."""
    # Try to find mermaid-cli in various locations
    possible_paths = [
        # Development paths
        "../../../target/release/mermaid-cli",
        "../../../target/debug/mermaid-cli",
        # System PATH
        "mermaid-cli",
    ]

    for path in possible_paths:
        try:
            result = subprocess.run(
                f"which {path} 2>/dev/null || test -x {path}",
                shell=True,
                capture_output=True,
            )
            if result.returncode == 0:
                return path
        except Exception:
            continue

    # Fallback to PATH
    return "mermaid-cli"


def _run_cli(args: list[str]) -> str:
    """Run mermaid-cli with given arguments."""
    cli_path = _get_cli_path()
    try:
        result = subprocess.run(
            [cli_path] + args,
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            raise MermaidError(f"CLI error: {result.stderr}")
        return result.stdout.strip()
    except FileNotFoundError:
        raise MermaidError(
            "mermaid-cli binary not found. Please install mermaid-cli first."
        )


def render(code: str) -> str:
    """
    Render a Mermaid diagram to SVG.

    Args:
        code: Mermaid diagram source code

    Returns:
        SVG string

    Raises:
        MermaidError: If rendering fails
    """
    return render_with_options(code, format="svg")


def render_png(code: str) -> bytes:
    """
    Render a Mermaid diagram to PNG.

    Args:
        code: Mermaid diagram source code

    Returns:
        PNG bytes

    Raises:
        MermaidError: If rendering fails
    """
    # For now, return SVG as bytes (PNG requires resvg feature)
    return render(code).encode("utf-8")


def render_pdf(code: str) -> bytes:
    """
    Render a Mermaid diagram to PDF.

    Args:
        code: Mermaid diagram source code

    Returns:
        PDF bytes

    Raises:
        MermaidError: If rendering fails
    """
    svg = render_with_options(code, format="pdf")
    return svg.encode("utf-8")


def parse(code: str) -> dict:
    """
    Parse Mermaid source code into AST.

    Args:
        code: Mermaid diagram source code

    Returns:
        Parsed AST as dictionary

    Raises:
        MermaidError: If parsing fails
    """
    with tempfile.NamedTemporaryFile(mode="w", suffix=".mmd", delete=False) as f:
        f.write(code)
        temp_path = f.name

    try:
        result = _run_cli([temp_path, "--json"])
        import json
        return json.loads(result)
    except Exception as e:
        raise MermaidError(f"Parse error: {e}") from e
    finally:
        os.unlink(temp_path)


def check(code: str) -> bool:
    """
    Check if Mermaid source code has valid syntax.

    Args:
        code: Mermaid diagram source code

    Returns:
        True if valid, False otherwise
    """
    with tempfile.NamedTemporaryFile(mode="w", suffix=".mmd", delete=False) as f:
        f.write(code)
        temp_path = f.name

    try:
        _run_cli(["check", temp_path, "-q"])
        return True
    except MermaidError:
        return False
    finally:
        os.unlink(temp_path)


def fix(code: str) -> str:
    """
    Automatically fix common errors in Mermaid code.

    Args:
        code: Mermaid diagram source code

    Returns:
        Fixed code

    Raises:
        MermaidError: If fixing fails
    """
    with tempfile.NamedTemporaryFile(mode="w", suffix=".mmd", delete=False) as f:
        f.write(code)
        input_path = f.name

    output_path = input_path + ".fixed.mmd"

    try:
        _run_cli(["fix", input_path, "-o", output_path, "-q"])
        with open(output_path, "r") as f:
            return f.read()
    finally:
        os.unlink(input_path)
        if os.path.exists(output_path):
            os.unlink(output_path)


def render_with_options(
    code: str,
    format: OutputFormat = "svg",
    theme: Theme = "default",
    width: int = 800,
    height: int = 600,
    scale: float = 1.0,
    background_color: str = "white",
    config: Optional[dict] = None,
) -> str:
    """
    Render with custom options.

    Args:
        code: Mermaid diagram source code
        format: Output format (svg, png, pdf)
        theme: Theme name
        width: Output width
        height: Output height
        scale: Scale factor
        background_color: Background color
        config: Custom configuration dictionary

    Returns:
        Rendered output as string

    Raises:
        MermaidError: If rendering fails
    """
    with tempfile.NamedTemporaryFile(mode="w", suffix=".mmd", delete=False) as f:
        f.write(code)
        input_path = f.name

    output_path = f"{input_path}.{format}"

    try:
        args = [
            input_path,
            "-e", format,
            "-o", output_path,
            "-t", theme,
            "-w", str(width),
            "-H", str(height),
            "-s", str(scale),
            "-b", background_color,
            "-q",
        ]

        _run_cli(args)

        with open(output_path, "r") as f:
            return f.read()
    finally:
        os.unlink(input_path)
        if os.path.exists(output_path):
            os.unlink(output_path)


# Public API
__all__ = [
    "render",
    "render_png",
    "render_pdf",
    "parse",
    "check",
    "fix",
    "render_with_options",
    "MermaidError",
    "InvalidSyntaxError",
]
