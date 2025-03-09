// lib.rs - SQL to Redis Command Transformer
// A declarative, functional approach to transforming SQL queries to Redis commands

pub mod ast;
pub mod pattern;
pub mod context;
pub mod rules;
pub mod templates;


use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use std::error::Error;
use std::fmt;

use crate::rules::{Rule, create_rules};
use crate::templates::TemplateEngine;

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
        let template_engine = TemplateEngine::new()
            .map_err(|e| SqlRedisError::InitializationError(format!("{}", e)))?;
        
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
        
        // Try to match any rule
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
        
        // If no rule matched, fall back to existing pattern-based transform
        Err(SqlRedisError::NoMatchingPattern(sql.to_string()))
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_get() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT * FROM users WHERE key = 'user:1001'").unwrap();
        assert_eq!(result, "GET user:1001");
    }

    #[test]
    fn test_hash_getall() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT * FROM users__hash WHERE key = 'user:1001'").unwrap();
        assert_eq!(result, "HGETALL user:1001");
    }

    #[test]
    fn test_hash_get() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT name FROM users__hash WHERE key = 'user:1001'").unwrap();
        assert_eq!(result, "HGET user:1001 name");
    }
    
    #[test]
    fn test_hash_multi_get() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT name, email, age FROM users__hash WHERE key = 'user:1001'").unwrap();
        assert_eq!(result, "HMGET user:1001 name email age");
    }
    
    #[test]
    fn test_list_getall() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT * FROM tweets__list WHERE key = 'user:1001:tweets'").unwrap();
        assert_eq!(result, "LRANGE user:1001:tweets 0 -1");
    }
    
    #[test]
    fn test_list_get_index() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT * FROM tweets__list WHERE key = 'user:1001:tweets' AND index = 0").unwrap();
        assert_eq!(result, "LINDEX user:1001:tweets 0");
    }
    
    #[test]
    fn test_list_get_range() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT * FROM tweets__list WHERE key = 'user:1001:tweets' LIMIT 10").unwrap();
        assert_eq!(result, "LRANGE user:1001:tweets 0 9");
    }
    
    #[test]
    fn test_set_getall() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT * FROM followers__set WHERE key = 'user:1001:followers'").unwrap();
        assert_eq!(result, "SMEMBERS user:1001:followers");
    }
    
    #[test]
    fn test_set_ismember() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT * FROM followers__set WHERE key = 'user:1001:followers' AND member = 'user:1002'").unwrap();
        assert_eq!(result, "SISMEMBER user:1001:followers user:1002");
    }
    
    #[test]
    fn test_zset_getall() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT * FROM leaderboard__zset WHERE key = 'games:leaderboard'").unwrap();
        assert_eq!(result, "ZRANGEBYSCORE games:leaderboard -inf +inf");
    }
    
    #[test]
    fn test_zset_get_score_range() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT * FROM leaderboard__zset WHERE key = 'games:leaderboard' AND score > 1000").unwrap();
        assert_eq!(result, "ZRANGEBYSCORE games:leaderboard (1000 +inf");
    }
    
    #[test]
    fn test_zset_get_reversed() {
        let transformer = SqlToRedisTransformer::new().unwrap();
        let result = transformer.transform("SELECT * FROM leaderboard__zset WHERE key = 'games:leaderboard' ORDER BY score DESC").unwrap();
        assert_eq!(result, "ZREVRANGEBYSCORE games:leaderboard +inf -inf");
    }
}