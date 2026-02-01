// crates/chitin-cli/src/output.rs
//
// Output formatting utilities for the Chitin CLI.
// Supports table and JSON output modes.

use serde::Serialize;
use tabled::{Table, Tabled};

/// Output format for CLI commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    /// Pretty-printed table output (default).
    Table,
    /// JSON output for machine consumption.
    Json,
}

/// Format a slice of Tabled items as a table string.
pub fn format_table<T: Tabled>(data: &[T]) -> String {
    Table::new(data).to_string()
}

/// Format a serializable value as a pretty-printed JSON string.
pub fn format_json<T: Serialize>(data: &T) -> String {
    serde_json::to_string_pretty(data).unwrap_or_else(|e| format!("JSON serialization error: {}", e))
}
