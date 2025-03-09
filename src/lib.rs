// lib.rs - Main implementation of SQL to Redis transformer

use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use std::error::Error;
use std::fmt;

use crate::rules::{Rule, create_rules};
use crate::templates::TemplateEngine;
use crate::commands::generate_command;

// Core error type for SQL-Redis transformation
#[derive(Debug)]
pub enum SqlRedisError {
    SqlParseError(String),
    NoMatchingPattern(String),
    TemplateError(String),
    InitializationError(String),
}

impl fmt::Display for SqlRedisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlRedisError::SqlParseError(msg) => write!(f, "SQL parse error: {}", msg),
            SqlRedisError::NoMatchingPattern(sql) => write!(f, "No matching pattern for: {}", sql),
            SqlRedisError::TemplateError(msg) => write!(f, "Template error: {}", msg),
            SqlRedisError::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
        }
    }
}

impl Error for SqlRedisError {}

// Main transformer that connects SQL pattern matching with Redis command templates
pub struct SqlToRedisTransformer {
    rules: Vec<Box<dyn Rule>>,
    template_engine: TemplateEngine,
}

impl SqlToRedisTransformer {
    pub fn new() -> Result<Self, SqlRedisError> {
        // Create template engine
        let template_engine = match TemplateEngine::new() {
            Ok(engine) => engine,
            Err(e) => return Err(SqlRedisError::InitializationError(format!("Template engine error: {}", e))),
        };
        
        // Create rules
        let rules = create_rules();
        
        Ok(Self { rules, template_engine })
    }
    
    pub fn transform(&self, sql: &str) -> Result<String, SqlRedisError> {
        // Parse SQL into AST
        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, sql)
            .map_err(|e| SqlRedisError::SqlParseError(e.to_string()))?;
        
        if ast.is_empty() {
            return Err(SqlRedisError::SqlParseError("Empty SQL statement".to_string()));
        }
        
        let stmt = &ast[0];
        
        // First strategy: Rule-based matching with templates
        for rule in &self.rules {
            if rule.matches(stmt) {
                // Get context from the rule for the matched statement
                if let Some(context) = rule.get_context(stmt) {
                    // Get template name from the rule
                    let template_name = rule.get_template_name();
                    
                    // Render template with context
                    return self.template_engine.render(template_name, &context)
                        .map_err(|e| SqlRedisError::TemplateError(e.to_string()));
                }
            }
        }
        
        // Second strategy: Direct command generation
        if let Some(command) = generate_command(stmt) {
            return Ok(command.to_string());
        }
        
        // If both strategies fail, return error
        Err(SqlRedisError::NoMatchingPattern(sql.to_string()))
    }

    pub fn list_supported_patterns(&self) -> Vec<String> {
        self.rules
            .iter()
            .map(|rule| rule.get_template_name().to_string())
            .collect()
    }
    
    /// Returns detailed information about all patterns
    pub fn get_pattern_details(&self) -> Vec<PatternInfo> {
        self.rules
            .iter()
            .map(|rule| PatternInfo {
                name: rule.get_template_name().to_string(),
                matcher: rule.get_matcher_name().unwrap_or("unknown").to_string(),
                sql_pattern: rule.get_sql_pattern().unwrap_or("").to_string(),
                redis_pattern: rule.get_redis_pattern().unwrap_or("").to_string(),
            })
            .collect()
    }
}

/// Detailed information about a pattern
#[derive(Debug, Clone)]
pub struct PatternInfo {
    pub name: String,
    pub matcher: String,
    pub sql_pattern: String,
    pub redis_pattern: String,
}

// Modules
pub mod ast;
pub mod pattern;
pub mod context;
pub mod rules;
pub mod templates;
pub mod commands;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformer_initialization() {
        let transformer = SqlToRedisTransformer::new();
        assert!(transformer.is_ok(), "Failed to initialize transformer");
    }
    
    #[test]
    fn test_get_pattern_details() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let details = transformer.get_pattern_details();
        assert!(!details.is_empty(), "No patterns were registered");
    }
    
    #[test]
    fn test_list_supported_patterns() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let patterns = transformer.list_supported_patterns();
        assert!(!patterns.is_empty(), "No patterns were supported");
    }
    
    #[test]
    fn test_error_handling() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        
        // Test with invalid SQL
        let result = transformer.transform("NOT A VALID SQL");
        assert!(result.is_err(), "Should fail with invalid SQL");
        
        // Test with unsupported SQL pattern
        let result = transformer.transform("SELECT * FROM non_redis_table");
        assert!(result.is_err(), "Should fail with unsupported pattern");
    }
}