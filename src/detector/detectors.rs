//! Diagram type detectors.
//!
//! Each detector checks if the input text matches a specific diagram type.
//! The detection order is important and matches Mermaid.js.

use once_cell::sync::Lazy;
use regex::Regex;

use super::DiagramType;
use crate::config::MermaidConfig;

// ============================================================================
// Regex patterns for detection
// ============================================================================

static RE_GRAPH: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*graph\b").unwrap());
static RE_FLOWCHART: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*flowchart\b").unwrap());
static RE_FLOWCHART_ELK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^\s*flowchart-elk\b").unwrap());
static RE_SEQUENCE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*sequenceDiagram\b").unwrap());
static RE_CLASS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*classDiagram\b").unwrap());
static RE_CLASS_V2: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*classDiagram-v2\b").unwrap());
static RE_STATE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*stateDiagram\b").unwrap());
static RE_STATE_V2: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*stateDiagram-v2\b").unwrap());
static RE_ER: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*erDiagram\b").unwrap());
static RE_GANTT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*gantt\b").unwrap());
static RE_JOURNEY: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*journey\b").unwrap());
static RE_REQUIREMENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^\s*requirement(Diagram)?\b").unwrap());
static RE_GITGRAPH: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*gitGraph\b").unwrap());
static RE_XYCHART: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*xychart(-beta)?\b").unwrap());
static RE_QUADRANT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*quadrantChart\b").unwrap());
static RE_C4: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^\s*(C4Context|C4Container|C4Component|C4Dynamic|C4Deployment)\b").unwrap()
});
static RE_PACKET: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*packet(-beta)?\b").unwrap());
static RE_TREEMAP: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*treemap\b").unwrap());
static RE_SANKEY: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*sankey(-beta)?\b").unwrap());
static RE_KANBAN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*kanban\b").unwrap());
static RE_BLOCK: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*block(-beta)?\b").unwrap());
static RE_RADAR: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*radar(-beta)?\b").unwrap());
static RE_PIE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*pie\b").unwrap());
static RE_INFO: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*info\b").unwrap());
static RE_TIMELINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*timeline\b").unwrap());
static RE_MINDMAP: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*mindmap\b").unwrap());
static RE_ARCHITECTURE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^\s*architecture(-beta)?\b").unwrap());

// ============================================================================
// Detection functions
// ============================================================================

/// Detects the diagram type from the preprocessed text.
///
/// The detection order matches Mermaid.js:
/// 1. error (special)
/// 2. --- (bad frontmatter)
/// 3. flowchart-elk (if large features enabled)
/// 4. mindmap (if large features enabled)
/// 5. architecture (if large features enabled)
/// 6. c4
/// 7. kanban
/// 8. classDiagram-v2
/// 9. classDiagram (legacy)
/// 10. er
/// 11. gantt
/// 12. info
/// 13. pie
/// 14. requirement
/// 15. sequence
/// 16. flowchart-v2
/// 17. flowchart (legacy graph)
/// 18. timeline
/// 19. gitGraph
/// 20. stateDiagram-v2
/// 21. stateDiagram (legacy)
/// 22. journey
/// 23. quadrantChart
/// 24. sankey
/// 25. packet
/// 26. xychart
/// 27. block
/// 28. radar
/// 29. treemap
pub fn detect_type(text: &str, config: &MermaidConfig) -> Option<DiagramType> {
    let text = text.trim();

    // Special cases
    if text.to_lowercase() == "error" {
        return Some(DiagramType::Error);
    }

    if text.trim_start().starts_with("---") {
        return Some(DiagramType::BadFrontmatter);
    }

    // Large features (flowchart-elk, mindmap, architecture)
    // These are typically enabled by config, but we always support them
    if RE_FLOWCHART_ELK.is_match(text) {
        return Some(DiagramType::FlowchartElk);
    }
    if RE_MINDMAP.is_match(text) {
        return Some(DiagramType::Mindmap);
    }
    if RE_ARCHITECTURE.is_match(text) {
        return Some(DiagramType::Architecture);
    }

    // C4 diagrams
    if RE_C4.is_match(text) {
        return Some(DiagramType::C4);
    }

    // Kanban
    if RE_KANBAN.is_match(text) {
        return Some(DiagramType::Kanban);
    }

    // Class diagrams (check v2 first, then legacy)
    if RE_CLASS_V2.is_match(text) {
        return Some(DiagramType::ClassDiagram);
    }
    if RE_CLASS.is_match(text) {
        // Check config for default renderer
        if config
            .class
            .default_renderer
            .as_deref()
            == Some("dagre-wrapper")
        {
            return Some(DiagramType::ClassDiagram);
        }
        return Some(DiagramType::Class);
    }

    // ER diagram
    if RE_ER.is_match(text) {
        return Some(DiagramType::Er);
    }

    // Gantt
    if RE_GANTT.is_match(text) {
        return Some(DiagramType::Gantt);
    }

    // Info
    if RE_INFO.is_match(text) {
        return Some(DiagramType::Info);
    }

    // Pie
    if RE_PIE.is_match(text) {
        return Some(DiagramType::Pie);
    }

    // Requirement
    if RE_REQUIREMENT.is_match(text) {
        return Some(DiagramType::Requirement);
    }

    // Sequence
    if RE_SEQUENCE.is_match(text) {
        return Some(DiagramType::Sequence);
    }

    // Flowchart (check for 'flowchart' keyword first, then 'graph')
    if RE_FLOWCHART.is_match(text) {
        // Check for ELK layout
        if config.flowchart.default_renderer.as_deref() == Some("elk")
            || config.layout.as_deref() == Some("elk")
        {
            return Some(DiagramType::FlowchartElk);
        }
        return Some(DiagramType::FlowchartV2);
    }

    if RE_GRAPH.is_match(text) {
        // 'graph' keyword - check config for renderer
        let renderer = config.flowchart.default_renderer.as_deref();
        match renderer {
            Some("elk") => return Some(DiagramType::FlowchartElk),
            Some("dagre-wrapper") => return Some(DiagramType::FlowchartV2),
            _ => return Some(DiagramType::Flowchart),
        }
    }

    // Timeline
    if RE_TIMELINE.is_match(text) {
        return Some(DiagramType::Timeline);
    }

    // Git Graph
    if RE_GITGRAPH.is_match(text) {
        return Some(DiagramType::GitGraph);
    }

    // State diagrams (check v2 first, then legacy)
    if RE_STATE_V2.is_match(text) {
        return Some(DiagramType::StateDiagram);
    }
    if RE_STATE.is_match(text) {
        if config
            .state
            .default_renderer
            .as_deref()
            == Some("dagre-wrapper")
        {
            return Some(DiagramType::StateDiagram);
        }
        return Some(DiagramType::State);
    }

    // Journey
    if RE_JOURNEY.is_match(text) {
        return Some(DiagramType::Journey);
    }

    // Quadrant chart
    if RE_QUADRANT.is_match(text) {
        return Some(DiagramType::QuadrantChart);
    }

    // Sankey
    if RE_SANKEY.is_match(text) {
        return Some(DiagramType::Sankey);
    }

    // Packet
    if RE_PACKET.is_match(text) {
        return Some(DiagramType::Packet);
    }

    // XY chart
    if RE_XYCHART.is_match(text) {
        return Some(DiagramType::XyChart);
    }

    // Block
    if RE_BLOCK.is_match(text) {
        return Some(DiagramType::Block);
    }

    // Radar
    if RE_RADAR.is_match(text) {
        return Some(DiagramType::Radar);
    }

    // Treemap
    if RE_TREEMAP.is_match(text) {
        return Some(DiagramType::Treemap);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(text: &str) -> Option<DiagramType> {
        detect_type(text, &MermaidConfig::default())
    }

    #[test]
    fn test_detect_error() {
        assert_eq!(detect("error"), Some(DiagramType::Error));
        assert_eq!(detect("ERROR"), Some(DiagramType::Error));
        assert_eq!(detect("  error  "), Some(DiagramType::Error));
    }

    #[test]
    fn test_detect_bad_frontmatter() {
        assert_eq!(detect("---"), Some(DiagramType::BadFrontmatter));
        assert_eq!(detect("---\nsome content"), Some(DiagramType::BadFrontmatter));
    }

    #[test]
    fn test_detect_flowchart() {
        assert_eq!(detect("graph TD\n    A --> B"), Some(DiagramType::Flowchart));
        assert_eq!(detect("graph LR\n    A --> B"), Some(DiagramType::Flowchart));
        assert_eq!(detect("  graph TB"), Some(DiagramType::Flowchart));
    }

    #[test]
    fn test_detect_flowchart_v2() {
        assert_eq!(
            detect("flowchart TD\n    A --> B"),
            Some(DiagramType::FlowchartV2)
        );
        assert_eq!(detect("flowchart LR"), Some(DiagramType::FlowchartV2));
    }

    #[test]
    fn test_detect_flowchart_elk() {
        assert_eq!(
            detect("flowchart-elk TD\n    A --> B"),
            Some(DiagramType::FlowchartElk)
        );
    }

    #[test]
    fn test_detect_flowchart_with_config() {
        let mut config = MermaidConfig::default();
        config.flowchart.default_renderer = Some("dagre-wrapper".to_string());

        assert_eq!(
            detect_type("graph TD\n    A --> B", &config),
            Some(DiagramType::FlowchartV2)
        );

        config.flowchart.default_renderer = Some("elk".to_string());
        assert_eq!(
            detect_type("graph TD\n    A --> B", &config),
            Some(DiagramType::FlowchartElk)
        );
    }

    #[test]
    fn test_detect_sequence() {
        assert_eq!(
            detect("sequenceDiagram\n    Alice->>Bob: Hello"),
            Some(DiagramType::Sequence)
        );
    }

    #[test]
    fn test_detect_class() {
        assert_eq!(
            detect("classDiagram\n    Class01 <|-- Class02"),
            Some(DiagramType::Class)
        );
    }

    #[test]
    fn test_detect_class_v2() {
        assert_eq!(
            detect("classDiagram-v2\n    Class01 <|-- Class02"),
            Some(DiagramType::ClassDiagram)
        );
    }

    #[test]
    fn test_detect_class_with_config() {
        let mut config = MermaidConfig::default();
        config.class.default_renderer = Some("dagre-wrapper".to_string());

        assert_eq!(
            detect_type("classDiagram\n    Class01 <|-- Class02", &config),
            Some(DiagramType::ClassDiagram)
        );
    }

    #[test]
    fn test_detect_state() {
        assert_eq!(
            detect("stateDiagram\n    [*] --> State1"),
            Some(DiagramType::State)
        );
        assert_eq!(
            detect("stateDiagram-v2\n    [*] --> State1"),
            Some(DiagramType::StateDiagram)
        );
    }

    #[test]
    fn test_detect_er() {
        assert_eq!(
            detect("erDiagram\n    CUSTOMER ||--o{ ORDER : places"),
            Some(DiagramType::Er)
        );
    }

    #[test]
    fn test_detect_gantt() {
        assert_eq!(
            detect("gantt\n    title A Gantt Diagram"),
            Some(DiagramType::Gantt)
        );
    }

    #[test]
    fn test_detect_journey() {
        assert_eq!(
            detect("journey\n    title My journey"),
            Some(DiagramType::Journey)
        );
    }

    #[test]
    fn test_detect_gitgraph() {
        assert_eq!(
            detect("gitGraph\n    commit"),
            Some(DiagramType::GitGraph)
        );
    }

    #[test]
    fn test_detect_pie() {
        assert_eq!(
            detect("pie title Pets\n    \"Dogs\" : 386"),
            Some(DiagramType::Pie)
        );
    }

    #[test]
    fn test_detect_c4() {
        assert_eq!(detect("C4Context"), Some(DiagramType::C4));
        assert_eq!(detect("C4Container"), Some(DiagramType::C4));
        assert_eq!(detect("C4Component"), Some(DiagramType::C4));
    }

    #[test]
    fn test_detect_mindmap() {
        assert_eq!(detect("mindmap\n  root"), Some(DiagramType::Mindmap));
    }

    #[test]
    fn test_detect_timeline() {
        assert_eq!(
            detect("timeline\n    title Timeline"),
            Some(DiagramType::Timeline)
        );
    }

    #[test]
    fn test_detect_kanban() {
        assert_eq!(detect("kanban\n    todo"), Some(DiagramType::Kanban));
    }

    #[test]
    fn test_detect_packet() {
        assert_eq!(
            detect("packet-beta\n    0-15: Header"),
            Some(DiagramType::Packet)
        );
    }

    #[test]
    fn test_detect_sankey() {
        assert_eq!(
            detect("sankey-beta\n    A,B,10"),
            Some(DiagramType::Sankey)
        );
    }

    #[test]
    fn test_detect_block() {
        assert_eq!(detect("block-beta"), Some(DiagramType::Block));
    }

    #[test]
    fn test_detect_xychart() {
        assert_eq!(detect("xychart-beta"), Some(DiagramType::XyChart));
    }

    #[test]
    fn test_detect_quadrant() {
        assert_eq!(
            detect("quadrantChart\n    title Test"),
            Some(DiagramType::QuadrantChart)
        );
    }

    #[test]
    fn test_detect_requirement() {
        assert_eq!(
            detect("requirementDiagram\n    requirement test_req"),
            Some(DiagramType::Requirement)
        );
    }

    #[test]
    fn test_detect_radar() {
        assert_eq!(detect("radar-beta"), Some(DiagramType::Radar));
    }

    #[test]
    fn test_detect_treemap() {
        assert_eq!(detect("treemap\n    root"), Some(DiagramType::Treemap));
    }

    #[test]
    fn test_detect_architecture() {
        assert_eq!(
            detect("architecture-beta"),
            Some(DiagramType::Architecture)
        );
    }

    #[test]
    fn test_detect_info() {
        assert_eq!(detect("info"), Some(DiagramType::Info));
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect("unknown diagram type"), None);
        assert_eq!(detect(""), None);
    }
}
