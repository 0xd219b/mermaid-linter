//! Golden tests for the mermaid linter.
//!
//! These tests validate fixtures against expected JSON output.
//! When the expected output doesn't exist, it will be created.

use std::fs;
use std::path::PathBuf;

use mermaid_linter::parse;

/// Test all fixture files in a directory
fn test_fixtures_in_dir(dir_name: &str) {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(dir_name);

    let golden_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(dir_name);

    // Create golden dir if it doesn't exist
    fs::create_dir_all(&golden_dir).expect("Failed to create golden directory");

    // Find all .mmd files
    let entries = fs::read_dir(&fixtures_dir).expect("Failed to read fixtures directory");

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "mmd") {
            test_single_fixture(&path, &golden_dir);
        }
    }
}

fn test_single_fixture(fixture_path: &PathBuf, golden_dir: &PathBuf) {
    let fixture_name = fixture_path.file_stem().unwrap().to_str().unwrap();
    let golden_path = golden_dir.join(format!("{}.json", fixture_name));

    // Read fixture
    let content = fs::read_to_string(fixture_path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {:?}", fixture_path));

    // Parse
    let result = parse(&content, None);

    // Create a serializable result
    let output = serde_json::json!({
        "ok": result.ok,
        "diagram_type": result.diagram_type.map(|t| t.as_str()),
        "title": result.title,
        "has_ast": result.ast.is_some(),
        "diagnostic_count": result.diagnostics.len(),
        "diagnostics": result.diagnostics.iter().map(|d| {
            serde_json::json!({
                "code": d.code.as_str(),
                "message": d.message,
                "severity": d.severity.as_str(),
            })
        }).collect::<Vec<_>>(),
    });

    let output_str = serde_json::to_string_pretty(&output).unwrap();

    if golden_path.exists() {
        // Compare with golden file
        let expected = fs::read_to_string(&golden_path)
            .unwrap_or_else(|_| panic!("Failed to read golden file: {:?}", golden_path));

        assert_eq!(
            output_str.trim(),
            expected.trim(),
            "Golden test failed for {:?}\n\nExpected:\n{}\n\nActual:\n{}",
            fixture_path,
            expected,
            output_str
        );
    } else {
        // Create golden file
        fs::write(&golden_path, &output_str)
            .unwrap_or_else(|_| panic!("Failed to write golden file: {:?}", golden_path));
        println!("Created golden file: {:?}", golden_path);
    }
}

#[test]
fn test_flowchart_fixtures() {
    test_fixtures_in_dir("flowchart");
}

#[test]
fn test_sequence_fixtures() {
    test_fixtures_in_dir("sequence");
}

#[test]
fn test_class_fixtures() {
    test_fixtures_in_dir("class");
}

#[test]
fn test_state_fixtures() {
    test_fixtures_in_dir("state");
}

/// Test error cases - diagrams that should fail
mod error_cases {
    use mermaid_linter::parse;

    #[test]
    fn test_unknown_diagram() {
        let result = parse("not a valid diagram", None);
        assert!(!result.ok);
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_empty_input() {
        let result = parse("", None);
        assert!(!result.ok);
    }

    #[test]
    fn test_whitespace_only() {
        let result = parse("   \n\n\t  ", None);
        assert!(!result.ok);
    }

    #[test]
    fn test_incomplete_flowchart() {
        let _result = parse("graph TD\n    A -->", None);
        // May succeed with partial parse or fail
        // Just ensure it doesn't panic
    }

    #[test]
    fn test_incomplete_sequence() {
        let _result = parse("sequenceDiagram\n    Alice->>", None);
        // May succeed with partial parse or fail
        // Just ensure it doesn't panic
    }

    #[test]
    fn test_malformed_frontmatter() {
        let _result = parse("---\ninvalid: yaml: here\n---\ngraph TD\nA-->B", None);
        // Should handle gracefully
    }

    #[test]
    fn test_malformed_directive() {
        let _result = parse("%%{init: {invalid json}}%%\ngraph TD\nA-->B", None);
        // Should handle gracefully - directive may be ignored
    }
}

/// Regression tests for specific issues
mod regression {
    use mermaid_linter::parse;

    #[test]
    fn test_node_names_with_end_substring() {
        // Issue: Node names containing "end" were misinterpreted
        let result = parse("graph TD\n    endpoint --> sender", None);
        assert!(result.ok, "Failed: {:?}", result.diagnostics);
    }

    #[test]
    fn test_node_names_ending_with_keywords() {
        // Issue: Node names ending with keywords like "graph" or "end"
        let result = parse("graph TD\n    blend --> monograph", None);
        assert!(result.ok, "Failed: {:?}", result.diagnostics);
    }

    #[test]
    fn test_default_as_node_name() {
        // Issue: "default" used as node name
        let result = parse("graph TD\n    default --> node", None);
        assert!(result.ok, "Failed: {:?}", result.diagnostics);
    }

    #[test]
    fn test_state_names_with_as() {
        // Issue: State names containing "as"
        let result = parse("stateDiagram-v2\n    assemble --> assemblies", None);
        assert!(result.ok, "Failed: {:?}", result.diagnostics);
    }

    #[test]
    fn test_trailing_whitespace() {
        // Issue: Trailing whitespace after statements
        let result = parse("graph TD;\n\n\n A-->B; \n B-->C;", None);
        assert!(result.ok, "Failed: {:?}", result.diagnostics);
    }

    #[test]
    fn test_sequence_alt_else() {
        // Issue: alt/else blocks in sequence diagrams
        let code = r#"sequenceDiagram
    Alice->>Bob: Request
    alt Success
        Bob-->>Alice: OK
    else Failure
        Bob-->>Alice: Error
    end"#;
        let result = parse(code, None);
        assert!(result.ok, "Failed: {:?}", result.diagnostics);
    }
}
