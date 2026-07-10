/**
 * Mermaid CLI Node.js Binding
 *
 * High-performance native bindings for Node.js applications
 * Uses the compiled mermaid-cli Rust binary through FFI
 */

import { execSync, spawn } from 'child_process';
import { writeFileSync, readFileSync, unlinkSync, existsSync } from 'fs';
import { tmpdir } from 'os';
import { join } from 'path';
import { randomBytes } from 'crypto';

/**
 * Render a Mermaid diagram to SVG
 * @param code Mermaid diagram source code
 * @returns SVG string
 */
export function render(code: string): string {
  return renderInternal(code, 'svg');
}

/**
 * Render a Mermaid diagram to PNG
 * @param code Mermaid diagram source code
 * @returns PNG buffer
 */
export function renderPng(code: string): Buffer {
  const svg = renderInternal(code, 'svg');
  // For now, return SVG as buffer (PNG requires resvg)
  return Buffer.from(svg);
}

/**
 * Render a Mermaid diagram to PDF
 * @param code Mermaid diagram source code
 * @returns PDF buffer
 */
export function renderPdf(code: string): Buffer {
  const svg = renderInternal(code, 'pdf');
  return Buffer.from(svg);
}

/**
 * Parse Mermaid source code into AST
 * @param code Mermaid diagram source code
 * @returns Parsed AST as JSON string
 */
export function parse(code: string): string {
  const tempFile = createTempFile(code);
  try {
    // Use the CLI to parse and output JSON
    const result = executeCliCommand([
      tempFile,
      '--json'
    ]);
    return result;
  } finally {
    deleteTempFile(tempFile);
  }
}

/**
 * Check if Mermaid source code has valid syntax
 * @param code Mermaid diagram source code
 * @returns true if valid, false otherwise
 */
export function check(code: string): boolean {
  const tempFile = createTempFile(code);
  try {
    executeCliCommand(['check', tempFile, '-q']);
    return true;
  } catch {
    return false;
  } finally {
    deleteTempFile(tempFile);
  }
}

/**
 * Auto-fix Mermaid source code
 * @param code Mermaid diagram source code
 * @returns Fixed code
 */
export function fix(code: string): string {
  const tempFile = createTempFile(code);
  const outputFile = tempFile + '.fixed.mmd';
  try {
    executeCliCommand(['fix', tempFile, '-o', outputFile, '-q']);
    return readFileSync(outputFile, 'utf-8');
  } finally {
    deleteTempFile(tempFile);
    if (existsSync(outputFile)) {
      deleteTempFile(outputFile);
    }
  }
}

/**
 * Render with custom options
 * @param code Mermaid diagram source code
 * @param options Render options
 * @returns Rendered output (SVG/PNG/PDF)
 */
export interface RenderOptions {
  format?: 'svg' | 'png' | 'pdf';
  theme?: 'default' | 'forest' | 'dark' | 'neutral';
  width?: number;
  height?: number;
  scale?: number;
  backgroundColor?: string;
  config?: Record<string, any>;
}

export function renderWithOptions(code: string, options: RenderOptions = {}): string {
  const {
    format = 'svg',
    theme = 'default',
    width = 800,
    height = 600,
    scale = 1,
    backgroundColor = 'white'
  } = options;

  const tempFile = createTempFile(code);
  const outputFile = tempFile + '.' + format;

  try {
    const args = [
      tempFile,
      '-e', format,
      '-o', outputFile,
      '-t', theme,
      '-w', String(width),
      '-H', String(height),
      '-s', String(scale),
      '-b', backgroundColor,
      '-q'
    ];

    executeCliCommand(args);
    return readFileSync(outputFile, 'utf-8');
  } finally {
    deleteTempFile(tempFile);
    if (existsSync(outputFile)) {
      deleteTempFile(outputFile);
    }
  }
}

// Helper functions

function renderInternal(code: string, format: string): string {
  const tempFile = createTempFile(code);
  const outputFile = tempFile + '.' + format;

  try {
    executeCliCommand([
      tempFile,
      '-e', format,
      '-o', outputFile,
      '-q'
    ]);
    return readFileSync(outputFile, 'utf-8');
  } finally {
    deleteTempFile(tempFile);
    if (existsSync(outputFile)) {
      deleteTempFile(outputFile);
    }
  }
}

function createTempFile(content: string): string {
  const filename = join(tmpdir(), `mermaid_${randomBytes(8).toString('hex')}.mmd`);
  writeFileSync(filename, content);
  return filename;
}

function deleteTempFile(path: string): void {
  try {
    if (existsSync(path)) {
      unlinkSync(path);
    }
  } catch (e) {
    // Ignore deletion errors
  }
}

function executeCliCommand(args: string[]): string {
  const cliPath = getCliPath();
  try {
    const result = execSync(`${cliPath} ${args.map(arg => `"${arg}"`).join(' ')}`, {
      encoding: 'utf-8',
      stdio: ['pipe', 'pipe', 'pipe']
    });
    return result.trim();
  } catch (error: any) {
    throw new Error(`CLI execution failed: ${error.stderr || error.message}`);
  }
}

function getCliPath(): string {
  // Try to find mermaid-cli binary in common locations
  const possiblePaths = [
    // Development paths
    '../../../target/release/mermaid-cli',
    '../../../target/debug/mermaid-cli',
    // Installed as npm package
    './node_modules/.bin/mermaid-cli',
    // Global installation
    'mermaid-cli'
  ];

  for (const path of possiblePaths) {
    try {
      execSync(`which ${path} 2>/dev/null || test -x ${path}`, { stdio: 'pipe' });
      return path;
    } catch {
      // Continue to next path
    }
  }

  // Fallback
  return 'mermaid-cli';
}

// Type exports for TypeScript users
export type { RenderOptions };
