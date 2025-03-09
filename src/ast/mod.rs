// ast/mod.rs - Module for SQL AST node extraction and examination
// Pure functions that extract data from SQL AST nodes

pub mod select;
pub mod insert;
pub mod update;
pub mod delete;

// Re-export AST functions for convenience
pub use select::*;
pub use insert::*;
pub use update::*;
pub use delete::*;