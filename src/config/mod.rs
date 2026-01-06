//! Configuration types for Mermaid parsing.

use serde::{Deserialize, Serialize};

/// Options for parsing a Mermaid diagram.
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    /// Base configuration to use for parsing.
    pub base_config: Option<MermaidConfig>,
    /// Whether to suppress errors and return ok=false instead of throwing.
    pub suppress_errors: bool,
}

impl ParseOptions {
    /// Creates new parse options with a base configuration.
    pub fn with_config(config: MermaidConfig) -> Self {
        Self {
            base_config: Some(config),
            suppress_errors: false,
        }
    }
}

/// Mermaid configuration.
///
/// This mirrors relevant parts of Mermaid's configuration that affect parsing behavior.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MermaidConfig {
    /// Flowchart-specific configuration.
    #[serde(default)]
    pub flowchart: FlowchartConfig,

    /// Class diagram-specific configuration.
    #[serde(default, rename = "class")]
    pub class: ClassConfig,

    /// State diagram-specific configuration.
    #[serde(default)]
    pub state: StateConfig,

    /// Gantt chart-specific configuration.
    #[serde(default)]
    pub gantt: GanttConfig,

    /// Whether to wrap text.
    #[serde(default)]
    pub wrap: bool,

    /// General layout engine.
    #[serde(default)]
    pub layout: Option<String>,
}

impl MermaidConfig {
    /// Merges another config into this one.
    /// Values from `other` override values in `self`.
    pub fn merge(&mut self, other: &MermaidConfig) {
        // Merge flowchart config
        if other.flowchart.default_renderer.is_some() {
            self.flowchart.default_renderer = other.flowchart.default_renderer.clone();
        }

        // Merge class config
        if other.class.default_renderer.is_some() {
            self.class.default_renderer = other.class.default_renderer.clone();
        }

        // Merge state config
        if other.state.default_renderer.is_some() {
            self.state.default_renderer = other.state.default_renderer.clone();
        }

        // Merge gantt config
        if other.gantt.display_mode.is_some() {
            self.gantt.display_mode = other.gantt.display_mode.clone();
        }

        // Merge simple fields
        if other.wrap {
            self.wrap = true;
        }
        if other.layout.is_some() {
            self.layout = other.layout.clone();
        }
    }
}

/// Flowchart-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowchartConfig {
    /// Default renderer for flowcharts.
    /// Can be "dagre-d3", "dagre-wrapper", or "elk".
    #[serde(default)]
    pub default_renderer: Option<String>,
}

/// Class diagram-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassConfig {
    /// Default renderer for class diagrams.
    #[serde(default)]
    pub default_renderer: Option<String>,
}

/// State diagram-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateConfig {
    /// Default renderer for state diagrams.
    #[serde(default)]
    pub default_renderer: Option<String>,
}

/// Gantt chart-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GanttConfig {
    /// Display mode for Gantt charts.
    #[serde(default)]
    pub display_mode: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = MermaidConfig::default();
        assert!(!config.wrap);
        assert!(config.flowchart.default_renderer.is_none());
    }

    #[test]
    fn test_config_merge() {
        let mut base = MermaidConfig::default();
        let other = MermaidConfig {
            wrap: true,
            flowchart: FlowchartConfig {
                default_renderer: Some("elk".to_string()),
            },
            ..Default::default()
        };

        base.merge(&other);

        assert!(base.wrap);
        assert_eq!(
            base.flowchart.default_renderer,
            Some("elk".to_string())
        );
    }

    #[test]
    fn test_config_deserialize() {
        let json = r#"{
            "flowchart": {
                "defaultRenderer": "dagre-wrapper"
            },
            "wrap": true
        }"#;

        let config: MermaidConfig = serde_json::from_str(json).unwrap();
        assert!(config.wrap);
        assert_eq!(
            config.flowchart.default_renderer,
            Some("dagre-wrapper".to_string())
        );
    }
}
