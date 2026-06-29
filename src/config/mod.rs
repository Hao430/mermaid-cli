#![cfg(feature = "json")]

/// Mermaid JSON configuration support.
///
/// Parses Mermaid configuration files (typically `mermaid.json` or
/// passed via `--configFile`) and applies the settings to the Renderer.
///
/// This module is feature-gated behind the `json` feature.

use std::fs;
use std::path::Path;

/// Configuration options read from a Mermaid JSON config file.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MermaidConfig {
    /// Theme: default, forest, dark, neutral
    pub theme: Option<String>,
    /// Theme variables
    #[serde(rename = "themeVariables")]
    pub theme_variables: Option<ThemeVariables>,
    /// Base font size
    #[serde(rename = "fontSize")]
    pub font_size: Option<u32>,
}

/// Theme variables from Mermaid config.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ThemeVariables {
    #[serde(rename = "primaryColor")]
    pub primary_color: Option<String>,
    #[serde(rename = "secondaryColor")]
    pub secondary_color: Option<String>,
    #[serde(rename = "tertiaryColor")]
    pub tertiary_color: Option<String>,
    #[serde(rename = "primaryTextColor")]
    pub primary_text_color: Option<String>,
    #[serde(rename = "lineColor")]
    pub line_color: Option<String>,
    #[serde(rename = "fontFamily")]
    pub font_family: Option<String>,
}

impl Default for MermaidConfig {
    fn default() -> Self {
        MermaidConfig {
            theme: Some("default".to_string()),
            theme_variables: None,
            font_size: Some(14),
        }
    }
}

impl MermaidConfig {
    /// Load and parse a Mermaid JSON configuration file.
    pub fn from_file(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => Self::from_str(&content),
            Err(_) => {
                eprintln!("Warning: Could not read config file: {}", path.display());
                MermaidConfig::default()
            }
        }
    }

    /// Parse a Mermaid JSON configuration string.
    pub fn from_str(json: &str) -> Self {
        match serde_json::from_str::<MermaidConfig>(json) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: Could not parse Mermaid config JSON: {}", e);
                MermaidConfig::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = MermaidConfig::default();
        assert_eq!(config.theme.as_deref(), Some("default"));
    }

    #[test]
    fn test_config_parse_minimal() {
        let json = r##"{"theme": "dark"}"##;
        let config = MermaidConfig::from_str(json);
        assert_eq!(config.theme.as_deref(), Some("dark"));
    }

    #[test]
    fn test_config_parse_full() {
        let json = r##"{
            "theme": "forest",
            "themeVariables": {
                "primaryColor": "#ff0000",
                "fontFamily": "monospace"
            },
            "fontSize": 16
        }"##;
        let config = MermaidConfig::from_str(json);
        assert_eq!(config.theme.as_deref(), Some("forest"));
        assert_eq!(config.font_size, Some(16));
    }

    #[test]
    fn test_config_invalid_json() {
        let json = r##"{invalid json}"##;
        let config = MermaidConfig::from_str(json);
        assert_eq!(config.theme.as_deref(), Some("default"));
    }
}
