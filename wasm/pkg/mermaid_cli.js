/* @ts-self-types="./mermaid_cli.d.ts" */
import * as wasm from "./mermaid_cli_bg.wasm";
import { __wbg_set_wasm } from "./mermaid_cli_bg.js";

__wbg_set_wasm(wasm);

export {
    check, fix, parse, render
} from "./mermaid_cli_bg.js";
