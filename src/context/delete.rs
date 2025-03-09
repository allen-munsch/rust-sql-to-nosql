// context/delete.rs - Context builders for DELETE statement transformations
// Extracts template variables from DELETE AST nodes

use std::collections::HashMap;
use sqlparser::ast::Statement;
use crate::ast;
use crate::context::TemplateContext;
use crate::context::ContextBuilder;

// --------------------------------
// String Command Context Builders
// --------------------------------

/// Builder for string DEL commands
/// <string-delete> ::= "DELETE" "FROM" <table> "WHERE" "key" "=" <key>
pub struct StringDeleteContextBuilder;
impl ContextBuilder for StringDeleteContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::delete::get_key_value(stmt)?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        Some(context)
    }
}

// --------------------------------
// Hash Command Context Builders
// --------------------------------

/// Builder for hash DEL commands
/// <hash-delete> ::= "DELETE" "FROM" <table> "__hash" "WHERE" "key" "=" <key>
pub struct HashDeleteContextBuilder;
impl ContextBuilder for HashDeleteContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::delete::get_key_value(stmt)?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        Some(context)
    }
}

/// Builder for hash HDEL commands
/// <hash-delete-field> ::= "DELETE" "FROM" <table> "__hash" "WHERE" "key" "=" <key> "AND" "field" "=" <field>
pub struct HashDeleteFieldContextBuilder;
impl ContextBuilder for HashDeleteFieldContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::delete::get_key_value(stmt)?;
        let field = ast::delete::get_field_filter(stmt, "field")?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("field".to_string(), field);
        Some(context)
    }
}

// --------------------------------
// List Command Context Builders
// --------------------------------

/// Builder for list DEL commands
/// <list-delete> ::= "DELETE" "FROM" <table> "__list" "WHERE" "key" "=" <key>
pub struct ListDeleteContextBuilder;
impl ContextBuilder for ListDeleteContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::delete::get_key_value(stmt)?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        Some(context)
    }
}

/// Builder for list LREM commands
/// <list-delete-value> ::= "DELETE" "FROM" <table> "__list" "WHERE" "key" "=" <key> "AND" "value" "=" <value>
pub struct ListDeleteValueContextBuilder;
impl ContextBuilder for ListDeleteValueContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::delete::get_key_value(stmt)?;
        let value = ast::delete::get_field_filter(stmt, "value")?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("value".to_string(), value);
        Some(context)
    }
}

// --------------------------------
// Set Command Context Builders
// --------------------------------

/// Builder for set DEL commands
/// <set-delete> ::= "DELETE" "FROM" <table> "__set" "WHERE" "key" "=" <key>
pub struct SetDeleteContextBuilder;
impl ContextBuilder for SetDeleteContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::delete::get_key_value(stmt)?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        Some(context)
    }
}

/// Builder for set SREM commands
/// <set-delete-member> ::= "DELETE" "FROM" <table> "__set" "WHERE" "key" "=" <key> "AND" "member" "=" <member>
pub struct SetDeleteMemberContextBuilder;
impl ContextBuilder for SetDeleteMemberContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::delete::get_key_value(stmt)?;
        let member = ast::delete::get_field_filter(stmt, "member")?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("member".to_string(), member);
        Some(context)
    }
}

// --------------------------------
// Sorted Set Command Context Builders
// --------------------------------

/// Builder for sorted set DEL commands
/// <zset-delete> ::= "DELETE" "FROM" <table> "__zset" "WHERE" "key" "=" <key>
pub struct ZSetDeleteContextBuilder;
impl ContextBuilder for ZSetDeleteContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::delete::get_key_value(stmt)?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        Some(context)
    }
}

/// Builder for sorted set ZREM commands
/// <zset-delete-member> ::= "DELETE" "FROM" <table> "__zset" "WHERE" "key" "=" <key> "AND" "member" "=" <member>
pub struct ZSetDeleteMemberContextBuilder;
impl ContextBuilder for ZSetDeleteMemberContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::delete::get_key_value(stmt)?;
        let member = ast::delete::get_field_filter(stmt, "member")?;
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("member".to_string(), member);
        Some(context)
    }
}