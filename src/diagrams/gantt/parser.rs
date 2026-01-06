//! Parser for Gantt charts.

use crate::ast::{Ast, AstNode, NodeKind, Span};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity};

use super::lexer::{tokenize, GanttToken, Token};

/// Parser for Gantt charts.
pub struct GanttParser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> GanttParser<'a> {
    /// Create a new parser.
    pub fn new(source: &'a str) -> Self {
        Self {
            tokens: tokenize(source),
            pos: 0,
            source,
            diagnostics: Vec::new(),
        }
    }

    /// Parse the Gantt chart.
    pub fn parse(&mut self) -> Result<Ast, Vec<Diagnostic>> {
        let start_span = Span::new(0, self.source.len());
        let mut root = AstNode::new(NodeKind::Root, start_span);

        // Skip any leading whitespace/newlines
        self.skip_newlines();

        // Parse the gantt declaration
        if let Some(decl) = self.parse_declaration() {
            root.add_child(decl);
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ExpectedToken,
                "Expected 'gantt'".to_string(),
                Severity::Error,
                self.current_span(),
            ));
            return Err(self.diagnostics.clone());
        }

        // Parse statements
        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            if let Some(stmt) = self.parse_statement() {
                root.add_child(stmt);
            } else {
                // Skip unknown token
                self.advance();
            }
        }

        if self.diagnostics.iter().any(|d| d.severity == Severity::Error) {
            Err(self.diagnostics.clone())
        } else {
            Ok(Ast::new(root, self.source.to_string()))
        }
    }

    /// Parse the gantt declaration.
    fn parse_declaration(&mut self) -> Option<AstNode> {
        if !self.check(&GanttToken::Gantt) {
            return None;
        }

        let start = self.current_span().start;
        self.advance(); // consume 'gantt'
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::DiagramDeclaration, Span::new(start, end));
        node.text = Some("gantt".to_string());

        Some(node)
    }

    /// Parse a statement.
    fn parse_statement(&mut self) -> Option<AstNode> {
        self.skip_newlines();

        if self.is_at_end() {
            return None;
        }

        // Check for title
        if self.check(&GanttToken::Title) {
            return self.parse_title();
        }

        // Check for dateFormat
        if self.check(&GanttToken::DateFormat) {
            return self.parse_date_format();
        }

        // Check for axisFormat
        if self.check(&GanttToken::AxisFormat) {
            return self.parse_axis_format();
        }

        // Check for tickInterval
        if self.check(&GanttToken::TickInterval) {
            return self.parse_tick_interval();
        }

        // Check for excludes
        if self.check(&GanttToken::Excludes) {
            return self.parse_excludes();
        }

        // Check for includes
        if self.check(&GanttToken::Includes) {
            return self.parse_includes();
        }

        // Check for todayMarker
        if self.check(&GanttToken::TodayMarker) {
            return self.parse_today_marker();
        }

        // Check for weekday
        if self.check(&GanttToken::Weekday) {
            return self.parse_weekday();
        }

        // Check for section
        if self.check(&GanttToken::Section) {
            return self.parse_section();
        }

        // Check for accessibility
        if self.check(&GanttToken::AccTitle) || self.check(&GanttToken::AccDescr) {
            return self.parse_accessibility();
        }

        // Otherwise, try to parse a task
        self.parse_task()
    }

    /// Parse title statement.
    fn parse_title(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'title'

        let title = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "title");
        node.add_property("value", title.trim().to_string());
        Some(node)
    }

    /// Parse dateFormat statement.
    fn parse_date_format(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'dateFormat'

        let format = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "dateFormat");
        node.add_property("value", format.trim().to_string());
        Some(node)
    }

    /// Parse axisFormat statement.
    fn parse_axis_format(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'axisFormat'

        let format = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "axisFormat");
        node.add_property("value", format.trim().to_string());
        Some(node)
    }

    /// Parse tickInterval statement.
    fn parse_tick_interval(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'tickInterval'

        let interval = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "tickInterval");
        node.add_property("value", interval.trim().to_string());
        Some(node)
    }

    /// Parse excludes statement.
    fn parse_excludes(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'excludes'

        let excludes = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "excludes");
        node.add_property("value", excludes.trim().to_string());
        Some(node)
    }

    /// Parse includes statement.
    fn parse_includes(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'includes'

        let includes = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "includes");
        node.add_property("value", includes.trim().to_string());
        Some(node)
    }

    /// Parse todayMarker statement.
    fn parse_today_marker(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'todayMarker'

        let marker = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "todayMarker");
        node.add_property("value", marker.trim().to_string());
        Some(node)
    }

    /// Parse weekday statement.
    fn parse_weekday(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'weekday'

        let day = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", "weekday");
        node.add_property("value", day.trim().to_string());
        Some(node)
    }

    /// Parse section statement.
    fn parse_section(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        self.advance(); // consume 'section'

        let name = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Subgraph, Span::new(start, end));
        node.add_property("type", "section");
        node.add_property("name", name.trim().to_string());
        Some(node)
    }

    /// Parse accessibility statement.
    fn parse_accessibility(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;
        let acc_type = if self.check(&GanttToken::AccTitle) {
            "accTitle"
        } else {
            "accDescr"
        };
        self.advance();

        // Skip colon if present
        if self.check(&GanttToken::Colon) {
            self.advance();
        }

        // Check for multi-line description
        if self.check(&GanttToken::OpenBrace) {
            self.advance();
            let mut content = String::new();
            while !self.check(&GanttToken::CloseBrace) && !self.is_at_end() {
                content.push_str(&self.current_text());
                content.push(' ');
                self.advance();
            }
            if self.check(&GanttToken::CloseBrace) {
                self.advance();
            }
            let end = self.previous_span().end;

            let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
            node.add_property("type", acc_type);
            node.add_property("value", content.trim().to_string());
            return Some(node);
        }

        // Single line
        let value = self.consume_until_newline();
        let end = self.previous_span().end;

        let mut node = AstNode::new(NodeKind::Statement, Span::new(start, end));
        node.add_property("type", acc_type);
        node.add_property("value", value.trim().to_string());
        Some(node)
    }

    /// Parse a task.
    fn parse_task(&mut self) -> Option<AstNode> {
        let start = self.current_span().start;

        // Collect task name (everything before the colon)
        let mut task_name = String::new();
        while !self.check(&GanttToken::Colon) && !self.check(&GanttToken::Newline) && !self.is_at_end() {
            if !task_name.is_empty() && !self.current_text().is_empty() {
                task_name.push(' ');
            }
            task_name.push_str(&self.current_text());
            self.advance();
        }

        if task_name.trim().is_empty() {
            return None;
        }

        let mut node = AstNode::new(NodeKind::Node, Span::new(start, start));
        node.add_property("type", "task");
        node.add_property("name", task_name.trim().to_string());

        // Parse task data after colon
        if self.check(&GanttToken::Colon) {
            self.advance();
            self.parse_task_data(&mut node);
        }

        let end = self.previous_span().end;
        node.span = Span::new(start, end);
        Some(node)
    }

    /// Parse task data (modifiers, ID, dates, duration).
    fn parse_task_data(&mut self, node: &mut AstNode) {
        let mut modifiers = Vec::new();
        let mut task_id = None;
        let mut start_date = None;
        let mut end_date = None;
        let mut duration = None;
        let mut after_ref = None;
        let mut until_ref = None;

        while !self.check(&GanttToken::Newline) && !self.is_at_end() {
            // Skip commas
            if self.check(&GanttToken::Comma) {
                self.advance();
                continue;
            }

            // Check for modifiers
            if self.check(&GanttToken::Done) {
                modifiers.push("done");
                self.advance();
                continue;
            }
            if self.check(&GanttToken::Active) {
                modifiers.push("active");
                self.advance();
                continue;
            }
            if self.check(&GanttToken::Crit) {
                modifiers.push("crit");
                self.advance();
                continue;
            }
            if self.check(&GanttToken::Milestone) {
                modifiers.push("milestone");
                self.advance();
                continue;
            }

            // Check for after dependency
            if self.check(&GanttToken::After) {
                self.advance();
                if self.check(&GanttToken::Identifier) {
                    after_ref = Some(self.current_text().trim().to_string());
                    self.advance();
                }
                continue;
            }

            // Check for until dependency
            if self.check(&GanttToken::Until) {
                self.advance();
                if self.check(&GanttToken::Identifier) {
                    until_ref = Some(self.current_text().trim().to_string());
                    self.advance();
                }
                continue;
            }

            // Check for duration
            if self.check(&GanttToken::Duration) {
                duration = Some(self.current_text());
                self.advance();
                continue;
            }

            // Check for date
            if self.check(&GanttToken::Date) {
                if start_date.is_none() {
                    start_date = Some(self.current_text());
                } else {
                    end_date = Some(self.current_text());
                }
                self.advance();
                continue;
            }

            // Check for identifier (task ID)
            if self.check(&GanttToken::Identifier) {
                if task_id.is_none() {
                    task_id = Some(self.current_text());
                }
                self.advance();
                continue;
            }

            // Skip other tokens
            self.advance();
        }

        // Add properties to node
        if !modifiers.is_empty() {
            node.add_property("modifiers", modifiers.join(","));
        }
        if let Some(id) = task_id {
            node.add_property("id", id);
        }
        if let Some(date) = start_date {
            node.add_property("startDate", date);
        }
        if let Some(date) = end_date {
            node.add_property("endDate", date);
        }
        if let Some(dur) = duration {
            node.add_property("duration", dur);
        }
        if let Some(after) = after_ref {
            node.add_property("after", after);
        }
        if let Some(until) = until_ref {
            node.add_property("until", until);
        }
    }

    /// Consume tokens until newline.
    fn consume_until_newline(&mut self) -> String {
        let mut text = String::new();
        while !self.check(&GanttToken::Newline) && !self.is_at_end() {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(&self.current_text());
            self.advance();
        }
        text
    }

    // Helper methods

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn current_text(&self) -> String {
        self.current().map(|t| t.text.clone()).unwrap_or_default()
    }

    fn current_span(&self) -> Span {
        self.current()
            .map(|t| Span::new(t.span.start, t.span.end))
            .unwrap_or(Span::new(self.source.len(), self.source.len()))
    }

    fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens
                .get(self.pos - 1)
                .map(|t| Span::new(t.span.start, t.span.end))
                .unwrap_or(Span::new(0, 0))
        } else {
            Span::new(0, 0)
        }
    }

    fn check(&self, kind: &GanttToken) -> bool {
        self.current().map(|t| &t.kind == kind).unwrap_or(false)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn skip_newlines(&mut self) {
        while self.check(&GanttToken::Newline) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let code = r#"gantt
    title A Gantt Chart
    dateFormat YYYY-MM-DD
    section Section
    A task :a1, 2024-01-01, 30d"#;

        let mut parser = GanttParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_with_dependencies() {
        let code = r#"gantt
    title Project Timeline
    dateFormat YYYY-MM-DD
    section Development
    Task 1 :a1, 2024-01-01, 30d
    Task 2 :after a1, 20d"#;

        let mut parser = GanttParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_with_modifiers() {
        let code = r#"gantt
    dateFormat YYYY-MM-DD
    section Tasks
    Done task :done, a1, 2024-01-01, 30d
    Critical task :crit, a2, after a1, 20d
    Active task :active, a3, after a2, 15d"#;

        let mut parser = GanttParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_milestone() {
        let code = r#"gantt
    dateFormat YYYY-MM-DD
    section Milestones
    Release v1 :milestone, m1, 2024-02-01, 0d"#;

        let mut parser = GanttParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_excludes() {
        let code = r#"gantt
    dateFormat YYYY-MM-DD
    excludes weekends
    section Tasks
    Task 1 :a1, 2024-01-01, 30d"#;

        let mut parser = GanttParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_axis_format() {
        let code = r#"gantt
    dateFormat YYYY-MM-DD
    axisFormat %m/%d
    section Tasks
    Task 1 :a1, 2024-01-01, 30d"#;

        let mut parser = GanttParser::new(code);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    #[test]
    fn test_parse_invalid() {
        let code = "not a gantt chart";
        let mut parser = GanttParser::new(code);
        let result = parser.parse();
        assert!(result.is_err());
    }
}
