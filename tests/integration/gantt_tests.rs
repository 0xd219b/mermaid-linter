//! Integration tests for Gantt charts.

use mermaid_linter::{parse, DiagramType};

#[test]
fn test_simple_gantt() {
    let code = r#"gantt
    title A Gantt Diagram
    dateFormat YYYY-MM-DD
    section Section
    A task :a1, 2024-01-01, 30d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse simple Gantt: {:?}", result.diagnostics);
    assert_eq!(result.diagram_type, Some(DiagramType::Gantt));
    assert!(result.ast.is_some());
}

#[test]
fn test_gantt_multiple_sections() {
    let code = r#"gantt
    dateFormat YYYY-MM-DD
    section Design
    Design task :a1, 2024-01-01, 10d
    section Development
    Dev task :a2, after a1, 20d
    section Testing
    Test task :a3, after a2, 10d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse multiple sections: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_task_dependencies() {
    let code = r#"gantt
    dateFormat YYYY-MM-DD
    Task 1 :a1, 2024-01-01, 30d
    Task 2 :a2, after a1, 20d
    Task 3 :a3, after a2, 15d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse task dependencies: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_task_modifiers() {
    let code = r#"gantt
    dateFormat YYYY-MM-DD
    Done task :done, a1, 2024-01-01, 10d
    Critical task :crit, a2, after a1, 15d
    Active task :active, a3, after a2, 20d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse task modifiers: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_milestones() {
    let code = r#"gantt
    dateFormat YYYY-MM-DD
    section Milestones
    Release v1.0 :milestone, m1, 2024-02-01, 0d
    Release v2.0 :milestone, m2, 2024-06-01, 0d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse milestones: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_excludes() {
    let code = r#"gantt
    dateFormat YYYY-MM-DD
    excludes weekends
    section Tasks
    Task 1 :a1, 2024-01-01, 30d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse excludes: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_date_format() {
    let code = r#"gantt
    dateFormat DD-MM-YYYY
    axisFormat %d/%m
    section Tasks
    Task 1 :a1, 01-01-2024, 30d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse date format: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_tick_interval() {
    let code = r#"gantt
    dateFormat YYYY-MM-DD
    tickInterval 1week
    section Tasks
    Task 1 :a1, 2024-01-01, 30d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse tick interval: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_today_marker() {
    let code = r#"gantt
    dateFormat YYYY-MM-DD
    todayMarker off
    section Tasks
    Task 1 :a1, 2024-01-01, 30d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse today marker: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_combined_modifiers() {
    let code = r#"gantt
    dateFormat YYYY-MM-DD
    section Critical Path
    Critical done :done, crit, a1, 2024-01-01, 10d
    Critical active :active, crit, a2, after a1, 15d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse combined modifiers: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_with_title() {
    let code = r#"gantt
    title Project Schedule 2024
    dateFormat YYYY-MM-DD
    section Planning
    Requirements :a1, 2024-01-01, 14d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse with title: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_axis_format() {
    let code = r#"gantt
    dateFormat YYYY-MM-DD
    axisFormat %Y-%m-%d
    section Tasks
    Task 1 :a1, 2024-01-01, 30d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse axis format: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_duration_formats() {
    let code = r#"gantt
    dateFormat YYYY-MM-DD
    section Tasks
    Task hours :a1, 2024-01-01, 8h
    Task days :a2, after a1, 5d
    Task weeks :a3, after a2, 2w
    Task months :a4, after a3, 1M"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse duration formats: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_case_insensitive() {
    let code = r#"GANTT
    TITLE A Gantt Chart
    DATEFORMAT YYYY-MM-DD
    SECTION Section
    A task :a1, 2024-01-01, 30d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse case-insensitive: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_complex_example() {
    let code = r#"gantt
    title Software Development Lifecycle
    dateFormat YYYY-MM-DD
    excludes weekends

    section Planning
    Requirements gathering :done, a1, 2024-01-01, 14d
    System design :done, a2, after a1, 14d

    section Development
    Backend development :active, crit, a3, after a2, 30d
    Frontend development :crit, a4, after a2, 25d
    Integration :a5, after a3 a4, 10d

    section Testing
    Unit testing :a6, after a5, 7d
    Integration testing :a7, after a6, 7d
    UAT :a8, after a7, 14d

    section Deployment
    Release :milestone, m1, after a8, 0d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse complex example: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_accessibility() {
    let code = r#"gantt
    accTitle: Project Timeline
    accDescr: This chart shows the project timeline
    dateFormat YYYY-MM-DD
    section Tasks
    Task 1 :a1, 2024-01-01, 30d"#;

    let result = parse(code, None);
    assert!(result.ok, "Failed to parse accessibility: {:?}", result.diagnostics);
}

#[test]
fn test_gantt_invalid() {
    let code = "not a gantt chart";

    let result = parse(code, None);
    assert!(result.diagram_type != Some(DiagramType::Gantt) || !result.ok);
}
