//! Gantt chart parser.
//!
//! Parses Gantt charts with tasks, sections, milestones, and dependencies.
//!
//! # Syntax
//!
//! ```text
//! gantt
//!     title A Gantt Diagram
//!     dateFormat YYYY-MM-DD
//!     section Section
//!         A task           :a1, 2024-01-01, 30d
//!         Another task     :after a1, 20d
//! ```

pub mod lexer;
pub mod parser;

pub use parser::GanttParser;

/// Task status in a Gantt chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is active
    Active,
    /// Task is done/completed
    Done,
    /// Task is critical
    Critical,
    /// Normal task (no special status)
    Normal,
}

impl TaskStatus {
    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "active" => Some(TaskStatus::Active),
            "done" => Some(TaskStatus::Done),
            "crit" | "critical" => Some(TaskStatus::Critical),
            _ => None,
        }
    }

    /// Get string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Active => "active",
            TaskStatus::Done => "done",
            TaskStatus::Critical => "crit",
            TaskStatus::Normal => "normal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_from_str() {
        assert_eq!(TaskStatus::from_str("active"), Some(TaskStatus::Active));
        assert_eq!(TaskStatus::from_str("done"), Some(TaskStatus::Done));
        assert_eq!(TaskStatus::from_str("crit"), Some(TaskStatus::Critical));
        assert_eq!(TaskStatus::from_str("unknown"), None);
    }
}
