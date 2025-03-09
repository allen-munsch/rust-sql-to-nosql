// context/update.rs - Context builders for UPDATE statement transformations
// Extracts template variables from UPDATE AST nodes

use std::collections::HashMap;
use sqlparser::ast::Statement;
use crate::ast;
use crate::context::TemplateContext;
use crate::context::ContextBuilder;

// --------------------------------
// String Command Context Builders
// --------------------------------

/// Builder for string SET commands (update)
/// <string-update> ::= "UPDATE" <table> "SET" "value" "=" <new-value> "WHERE" "key" "=" <key>
pub struct StringUpdateContextBuilder;
impl ContextBuilder for StringUpdateContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::upd_get_key_value(stmt)?;
        let assignments = ast::upd_get_assignments(stmt)?;
        let value = assignments.get("value")?.clone();
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("value".to_string(), value);
        Some(context)
    }
}

// --------------------------------
// Hash Command Context Builders
// --------------------------------

/// Builder for hash HSET commands (update)
/// <hash-update> ::= "UPDATE" <table> "__hash" "SET" <field> "=" <value> ["," <field2> "=" <value2>]... "WHERE" "key" "=" <key>
pub struct HashUpdateContextBuilder;
impl ContextBuilder for HashUpdateContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::upd_get_key_value(stmt)?;
        let assignments = ast::upd_get_assignments(stmt)?;
        
        if assignments.is_empty() {
            return None;
        }
        
        // Format field-value pairs for template
        let fields_formatted: Vec<String> = assignments.iter()
            .map(|(field, value)| format!("{} {}", field, value))
            .collect();
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("field_values".to_string(), fields_formatted.join(" "));
        Some(context)
    }
}

// --------------------------------
// List Command Context Builders
// --------------------------------

/// Builder for list LSET commands
/// <list-update> ::= "UPDATE" <table> "__list" "SET" "value" "=" <new-value> "WHERE" "key" "=" <key> "AND" "index" "=" <index>
pub struct ListUpdateContextBuilder;
impl ContextBuilder for ListUpdateContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::upd_get_key_value(stmt)?;
        let index = ast::upd_get_field_filter(stmt, "index")?;
        
        let assignments = ast::upd_get_assignments(stmt)?;
        let value = assignments.get("value")?.clone();
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("index".to_string(), index);
        context.insert("value".to_string(), value);
        Some(context)
    }
}

// --------------------------------
// Sorted Set Command Context Builders
// --------------------------------

/// Builder for sorted set ZADD commands (update score)
/// <zset-update> ::= "UPDATE" <table> "__zset" "SET" "score" "=" <new-score> "WHERE" "key" "=" <key> "AND" "member" "=" <member>
pub struct ZSetUpdateContextBuilder;
impl ContextBuilder for ZSetUpdateContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::upd_get_key_value(stmt)?;
        let member = ast::upd_get_field_filter(stmt, "member")?;
        
        let assignments = ast::upd_get_assignments(stmt)?;
        let score = assignments.get("score")?.clone();
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("member".to_string(), member);
        context.insert("score".to_string(), score);
        Some(context)
    }
}