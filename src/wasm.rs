//! WASM bindings for mermaid-cli
//!
//! This module provides WebAssembly bindings so the Mermaid parser and
//! renderer can be used from JavaScript/TypeScript in the browser or Node.js.
//!
//! # Usage (after wasm-pack build)
//!
//! ```javascript
//! import { render, parse, check } from "mermaid-cli";
//!
//! // Render a diagram to SVG
//! const svg = render("graph TD; A-->B");
//! console.log(svg);
//!
//! // Parse to JSON AST
//! const ast = parse("graph TD; A-->B");
//! console.log(JSON.parse(ast));
//!
//! // Check syntax
//! const result = check("graph TD; A-->B");
//! console.log(result.valid ? "✓ Valid" : "✗ Invalid");
//! ```

#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;

/// Render a Mermaid diagram to SVG string.
///
/// # Arguments
/// * `code` - Mermaid diagram source code
///
/// # Returns
/// SVG string on success, error message on failure.
#[wasm_bindgen]
pub fn render(code: &str) -> Result<String, JsValue> {
    crate::render(code).map_err(|e| JsValue::from_str(&e))
}

/// Parse Mermaid source code to JSON AST string.
///
/// # Arguments
/// * `code` - Mermaid diagram source code
///
/// # Returns
/// JSON string of the AST on success.
#[wasm_bindgen]
pub fn parse(code: &str) -> Result<String, JsValue> {
    let diagram = crate::parse(code).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    let json =
        serde_json::to_string_pretty(&diagram).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(json)
}

/// Check Mermaid syntax and return JSON result.
///
/// Returns JSON with `{ "valid": bool, "errors": [...] }`.
#[wasm_bindgen]
pub fn check(code: &str) -> String {
    match crate::check(code) {
        Ok(result) => {
            #[cfg(feature = "json")]
            {
                serde_json::to_string(&result).unwrap_or_default()
            }
            #[cfg(not(feature = "json"))]
            {
                if result.valid {
                    r#"{"valid":true,"errors":[]}"#.to_string()
                } else {
                    format!(
                        r#"{{"valid":false,"errors":[{{"message":"{}"}}]}}"#,
                        result
                            .errors
                            .first()
                            .map(|e| e.to_string())
                            .unwrap_or_default()
                    )
                }
            }
        }
        Err(e) => format!(r#"{{"valid":false,"errors":[{{"message":"{}"}}]}}"#, e),
    }
}

/// Auto-fix common syntax errors in Mermaid source code.
///
/// Returns JSON with `{ "fixed": string, "fixes": string[] }`.
#[wasm_bindgen]
pub fn fix(code: &str) -> String {
    let (fixed, fixes) = crate::fix(code);
    #[cfg(feature = "json")]
    {
        serde_json::to_string(&serde_json::json!({
            "fixed": fixed,
            "fixes": fixes,
        }))
        .unwrap_or_else(|_| format!(r#"{{"fixed":{:?},"fixes":[]}}"#, fixed))
    }
    #[cfg(not(feature = "json"))]
    {
        format!(r#"{{"fixed":{:?},"fixes":[]}}"#, fixed)
    }
}
