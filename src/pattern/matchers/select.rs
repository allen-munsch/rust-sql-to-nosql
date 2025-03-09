// matchers/select.rs - Predicate functions for SELECT statement pattern matching
// Each function tests if a statement matches a particular SELECT pattern from the BNF grammar

use sqlparser::ast::{Expr, SelectItem, Statement};
use crate::ast;

// --------------------------------
// Basic Pattern Matchers - Pure Functions
// --------------------------------

/// Check if the SELECT statement has a wildcard (*) projection
pub fn is_wildcard_select(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .map(|select| select.projection.len() == 1 && ast::sel_is_wildcard(&select.projection[0]))
        .unwrap_or(false)
}

/// Check if the SELECT statement has a single field projection
pub fn is_single_field_select(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .map(|select| select.projection.len() == 1 && ast::sel_get_field_name(&select.projection[0]).is_some())
        .unwrap_or(false)
}

/// Check if the SELECT statement has multiple field projections
pub fn is_multi_field_select(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .map(|select| {
            select.projection.len() > 1 && 
            select.projection.iter().all(|item| ast::sel_get_field_name(item).is_some())
        })
        .unwrap_or(false)
}

// --------------------------------
// Table Type Matchers - Pure Functions
// --------------------------------

/// Check if the table name has a "__hash" suffix, indicating a Redis Hash
pub fn is_hash_table(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .and_then(ast::sel_get_table_name)
        .map(|name| name.ends_with("__hash"))
        .unwrap_or(false)
}

/// Check if the table name has a "__list" suffix, indicating a Redis List
pub fn is_list_table(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .and_then(ast::sel_get_table_name)
        .map(|name| name.ends_with("__list"))
        .unwrap_or(false)
}

/// Check if the table name has a "__set" suffix, indicating a Redis Set
pub fn is_set_table(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .and_then(ast::sel_get_table_name)
        .map(|name| name.ends_with("__set"))
        .unwrap_or(false)
}

/// Check if the table name has a "__zset" suffix, indicating a Redis Sorted Set
pub fn is_zset_table(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .and_then(ast::sel_get_table_name)
        .map(|name| name.ends_with("__zset"))
        .unwrap_or(false)
}

/// Check if the table name has no special suffix, indicating a Redis String
pub fn is_string_table(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .and_then(ast::sel_get_table_name)
        .map(|name| !name.ends_with("__hash") && !name.ends_with("__list") && 
                     !name.ends_with("__set") && !name.ends_with("__zset"))
        .unwrap_or(false)
}

// --------------------------------
// WHERE Clause Matchers - Pure Functions
// --------------------------------

/// Check if the WHERE clause has a "key = value" condition
pub fn has_key_equals(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .map(|select| ast::sel_get_key_value(&select.selection).is_some())
        .unwrap_or(false)
}

/// Check if the WHERE clause has a "field = value" condition for a specific field
pub fn has_field_equals(stmt: &Statement, field_name: &str) -> bool {
    ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .map(|select| ast::sel_get_field_filter(&select.selection, field_name).is_some())
        .unwrap_or(false)
}

/// Check if the WHERE clause has a score comparison for sorted sets
pub fn has_score_range(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .map(|select| ast::sel_get_score_range(&select.selection).is_some())
        .unwrap_or(false)
}

/// Check if the query has ORDER BY score DESC
pub fn has_order_by_score_desc(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .map(|query| ast::sel_is_order_by_score_desc(query))
        .unwrap_or(false)
}

/// Check if the query has a LIMIT clause
pub fn has_limit(stmt: &Statement) -> bool {
    ast::sel_get_query(stmt)
        .map(|query| query.limit.is_some())
        .unwrap_or(false)
}

// --------------------------------
// BNF Rule Matchers - Direct mapping to BNF rules
// --------------------------------

/// <string-get> ::= "SELECT" "*" "FROM" <table> "WHERE" "key" "=" <value>
pub fn is_string_get(stmt: &Statement) -> bool {
    is_wildcard_select(stmt) && is_string_table(stmt) && has_key_equals(stmt)
}

/// <hash-getall> ::= "SELECT" "*" "FROM" <table> "__hash" "WHERE" "key" "=" <value>
pub fn is_hash_getall(stmt: &Statement) -> bool {
    is_wildcard_select(stmt) && is_hash_table(stmt) && has_key_equals(stmt)
}

/// <hash-get> ::= "SELECT" <field> "FROM" <table> "__hash" "WHERE" "key" "=" <value>
pub fn is_hash_get(stmt: &Statement) -> bool {
    is_single_field_select(stmt) && is_hash_table(stmt) && has_key_equals(stmt)
}

/// <hash-hmget> ::= "SELECT" <field1> ["," <field2>]... "FROM" <table> "__hash" "WHERE" "key" "=" <value>
pub fn is_hash_hmget(stmt: &Statement) -> bool {
    is_multi_field_select(stmt) && is_hash_table(stmt) && has_key_equals(stmt)
}

/// <list-get> ::= "SELECT" "*" "FROM" <table> "__list" "WHERE" "key" "=" <value>
pub fn is_list_getall(stmt: &Statement) -> bool {
    is_wildcard_select(stmt) && is_list_table(stmt) && has_key_equals(stmt)
}

/// <list-get-index> ::= "SELECT" "*" "FROM" <table> "__list" "WHERE" "key" "=" <value> "AND" "index" "=" <index>
pub fn is_list_get_index(stmt: &Statement) -> bool {
    // First check if it's a wildcard select
    is_wildcard_select(stmt) && 
    // Check if we're querying a list table
    is_list_table(stmt) && 
    // Check for key condition
    has_key_equals(stmt) && 
    // Check for index condition
    has_field_equals(stmt, "index")
}

/// <list-get-range> ::= "SELECT" "*" "FROM" <table> "__list" "WHERE" "key" "=" <value> "LIMIT" <limit>
pub fn is_list_get_range(stmt: &Statement) -> bool {
    is_wildcard_select(stmt) && is_list_table(stmt) && has_key_equals(stmt) && has_limit(stmt)
}

/// <set-getall> ::= "SELECT" "*" "FROM" <table> "__set" "WHERE" "key" "=" <value>
pub fn is_set_getall(stmt: &Statement) -> bool {
    is_wildcard_select(stmt) && is_set_table(stmt) && has_key_equals(stmt)
}

/// <set-ismember> ::= "SELECT" "*" "FROM" <table> "__set" "WHERE" "key" "=" <value> "AND" "member" "=" <member>
pub fn is_set_ismember(stmt: &Statement) -> bool {
    is_wildcard_select(stmt) && is_set_table(stmt) && 
    has_key_equals(stmt) && has_field_equals(stmt, "member")
}

/// <zset-get> ::= "SELECT" "*" "FROM" <table> "__zset" "WHERE" "key" "=" <value>
pub fn is_zset_getall(stmt: &Statement) -> bool {
    is_wildcard_select(stmt) && is_zset_table(stmt) && has_key_equals(stmt)
}

/// <zset-get-score-range> ::= "SELECT" "*" "FROM" <table> "__zset" "WHERE" "key" "=" <value> "AND" "score" <comparison> <score>
pub fn is_zset_get_score_range(stmt: &Statement) -> bool {
    is_wildcard_select(stmt) && is_zset_table(stmt) && 
    has_key_equals(stmt) && has_score_range(stmt)
}

/// <zset-get-reversed> ::= "SELECT" "*" "FROM" <table> "__zset" "WHERE" "key" "=" <value> "ORDER BY" "score" "DESC"
pub fn is_zset_get_reversed(stmt: &Statement) -> bool {
    is_wildcard_select(stmt) && is_zset_table(stmt) && 
    has_key_equals(stmt) && has_order_by_score_desc(stmt)
}

pub fn query_has_limit(query: &sqlparser::ast::Query) -> Option<u64> {
    query.limit.as_ref().and_then(|limit| {
        match limit {
            sqlparser::ast::Expr::Value(value) => {
                if let sqlparser::ast::Value::Number(n, _) = &value.value {
                    n.parse::<u64>().ok()
                } else {
                    None
                }
            },
            _ => None,
        }
    })
}

pub fn is_string_get_value(stmt: &Statement) -> bool {
    // Check if it's a SELECT statement
    let query = match ast::sel_get_query(stmt) {
        Some(query) => query,
        None => return false,
    };
    
    // Check if we're selecting from a string table
    let select = match ast::sel_get_select(query) {
        Some(select) => select,
        None => return false,
    };
    
    // Use the existing is_string_table helper function
    if !is_string_table(stmt) {
        return false;
    }
    
    // Check if we're selecting only the 'value' field
    if select.projection.len() != 1 {
        return false;
    }
    
    let is_value_field = match &select.projection[0] {
        SelectItem::UnnamedExpr(Expr::Identifier(ident)) => 
            ident.value.to_lowercase() == "value",
        _ => false,
    };
    
    if !is_value_field {
        return false;
    }
    
    // Check for key = 'value' condition
    has_key_equals(stmt)
}

