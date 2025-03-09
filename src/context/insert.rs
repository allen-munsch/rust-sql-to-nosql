// context/insert.rs - Context builders for INSERT statement transformations
// Extracts template variables from INSERT AST nodes

use std::collections::HashMap;
use sqlparser::ast::Statement;
use crate::ast;
use crate::context::TemplateContext;
use crate::context::ContextBuilder;

// --------------------------------
// String Command Context Builders
// --------------------------------

/// Builder for string SET commands
/// <string-set> ::= "INSERT" "INTO" <table> "(key, value)" "VALUES" "(" <key> "," <value> ")"
pub struct StringSetContextBuilder;
impl ContextBuilder for StringSetContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::ins_get_column_value(stmt, "key")?;
        let value = ast::ins_get_column_value(stmt, "value")?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("value".to_string(), value);
        Some(context)
    }
}

// --------------------------------
// Hash Command Context Builders
// --------------------------------

/// Builder for hash HSET commands
/// <hash-set> ::= "INSERT" "INTO" <table> "__hash" "(key, field1 [, field2]...)" "VALUES" "(" <key> "," <value1> [, <value2>]... ")"
pub struct HashSetContextBuilder;
impl ContextBuilder for HashSetContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let value_maps = ast::ins_get_values_as_maps(stmt)?;
        if value_maps.is_empty() {
            return None;
        }
        
        let row = &value_maps[0];
        let key = row.get("key")?.clone();
        
        // Create a clone without the key
        let mut field_values = row.clone();
        field_values.remove("key");
        
        if field_values.is_empty() {
            return None;
        }
        
        // Format field-value pairs for template
        let fields_formatted: Vec<String> = field_values.iter()
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

/// Builder for list RPUSH commands
/// <list-push> ::= "INSERT" "INTO" <table> "__list" "(key, value)" "VALUES" "(" <key> "," <value> ")"
pub struct ListPushContextBuilder;
impl ContextBuilder for ListPushContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::ins_get_column_value(stmt, "key")?;
        let value = ast::ins_get_column_value(stmt, "value")?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("value".to_string(), value);
        Some(context)
    }
}

// --------------------------------
// Set Command Context Builders
// --------------------------------

/// Builder for set SADD commands
/// <set-add> ::= "INSERT" "INTO" <table> "__set" "(key, member)" "VALUES" "(" <key> "," <member> ")"
pub struct SetAddContextBuilder;
impl ContextBuilder for SetAddContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::ins_get_column_value(stmt, "key")?;
        
        // Get all member values that share the same key
        let values = ast::ins_get_values_as_maps(stmt)?;
        let first_key = values[0].get("key")?.clone();
        
        let members: Vec<String> = values.iter()
            .filter(|row| row.get("key").map_or(false, |k| k == &first_key))
            .filter_map(|row| row.get("member").cloned())
            .collect();
        
        if members.is_empty() {
            return None;
        }
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("members".to_string(), members.join(" "));
        Some(context)
    }
}

// --------------------------------
// Sorted Set Command Context Builders
// --------------------------------

/// Builder for sorted set ZADD commands
/// <zset-add> ::= "INSERT" "INTO" <table> "__zset" "(key, member, score)" "VALUES" "(" <key> "," <member> "," <score> ")"
pub struct ZSetAddContextBuilder;
impl ContextBuilder for ZSetAddContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::ins_get_column_value(stmt, "key")?;
        let member = ast::ins_get_column_value(stmt, "member")?;
        let score = ast::ins_get_column_value(stmt, "score")?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("member".to_string(), member);
        context.insert("score".to_string(), score);
        Some(context)
    }
}