/* tslint:disable */
/* eslint-disable */

/**
 * Check Mermaid syntax and return JSON result.
 *
 * Returns JSON with `{ "valid": bool, "errors": [...] }`.
 */
export function check(code: string): string;

/**
 * Auto-fix common syntax errors in Mermaid source code.
 *
 * Returns JSON with `{ "fixed": string, "fixes": string[] }`.
 */
export function fix(code: string): string;

/**
 * Parse Mermaid source code to JSON AST string.
 *
 * # Arguments
 * * `code` - Mermaid diagram source code
 *
 * # Returns
 * JSON string of the AST on success.
 */
export function parse(code: string): string;

/**
 * Render a Mermaid diagram to SVG string.
 *
 * # Arguments
 * * `code` - Mermaid diagram source code
 *
 * # Returns
 * SVG string on success, error message on failure.
 */
export function render(code: string): string;
