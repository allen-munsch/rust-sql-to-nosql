// context/mod.rs - Module for template context building
// Extracts template variables from SQL AST nodes

use std::collections::HashMap;
use sqlparser::ast::Statement;

pub mod select;
pub mod insert;
pub mod update;
pub mod delete;

// A Template Context is just key-value pairs
pub type TemplateContext = HashMap<String, String>;

// Create context for commands from AST nodes
pub trait ContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext>;
}

// Re-export context builders for convenience
pub use select::*;
pub use insert::*;
pub use update::*;
pub use delete::*;