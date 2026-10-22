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

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly check: (a: number, b: number, c: number) => void;
    readonly fix: (a: number, b: number, c: number) => void;
    readonly parse: (a: number, b: number, c: number) => void;
    readonly render: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
