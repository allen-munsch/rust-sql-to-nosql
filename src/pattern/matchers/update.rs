// matchers/update.rs - Predicate functions for UPDATE statement pattern matching
// Each function tests if a statement matches a particular UPDATE pattern from the BNF grammar

use sqlparser::ast::Statement;
use crate::ast::{
    upd_get_table_name,
    upd_get_key_value,
    upd_get_field_filter,
    upd_get_assignments,
};


// --------------------------------
// Basic Pattern Matchers - Pure Functions
// --------------------------------

/// Check if statement is an UPDATE statement
pub fn is_update(stmt: &Statement) -> bool {
    matches!(stmt, Statement::Update { .. })
}

/// Check if UPDATE statement target is a hash table
pub fn is_hash_table(stmt: &Statement) -> bool {
    match upd_get_table_name(stmt) {
        Some(name) => name.ends_with("__hash"),
        None => false,
    }
}

/// Check if UPDATE statement target is a list table
pub fn is_list_table(stmt: &Statement) -> bool {
    match upd_get_table_name(stmt) {
        Some(name) => name.ends_with("__list"),
        None => false,
    }
}

/// Check if UPDATE statement target is a set table
pub fn is_set_table(stmt: &Statement) -> bool {
    match upd_get_table_name(stmt) {
        Some(name) => name.ends_with("__set"),
        None => false,
    }
}

/// Check if UPDATE statement target is a sorted set table
pub fn is_zset_table(stmt: &Statement) -> bool {
    match upd_get_table_name(stmt) {
        Some(name) => name.ends_with("__zset"),
        None => false,
    }
}

/// Check if UPDATE statement target is a string table (default)
pub fn is_string_table(stmt: &Statement) -> bool {
    match upd_get_table_name(stmt) {
        Some(name) => !name.ends_with("__hash") && !name.ends_with("__list") && 
                     !name.ends_with("__set") && !name.ends_with("__zset"),
        None => false,
    }
}

/// Check if the UPDATE statement has a key condition
pub fn has_key_equals(stmt: &Statement) -> bool {
    upd_get_key_value(stmt).is_some()
}

/// Check if the UPDATE statement has a field condition
pub fn has_field_equals(stmt: &Statement, field_name: &str) -> bool {
    upd_get_field_filter(stmt, field_name).is_some()
}

/// Check if the UPDATE statement has a specific SET assignment
pub fn has_assignment(stmt: &Statement, field_name: &str) -> bool {
    match upd_get_assignments(stmt) {
        Some(assignments) => assignments.contains_key(field_name),
        None => false,
    }
}

// --------------------------------
// BNF Rule Matchers - Direct mapping to BNF rules
// --------------------------------

/// <string-update> ::= "UPDATE" <table> "SET" "value" "=" <new-value> "WHERE" "key" "=" <key>
pub fn is_string_update(stmt: &Statement) -> bool {
    is_update(stmt) && is_string_table(stmt) && has_key_equals(stmt) && has_assignment(stmt, "value")
}

/// <hash-update> ::= "UPDATE" <table> "__hash" "SET" <field> "=" <value> ["," <field2> "=" <value2>]... "WHERE" "key" "=" <key>
pub fn is_hash_update(stmt: &Statement) -> bool {
    is_update(stmt) && is_hash_table(stmt) && has_key_equals(stmt) && 
    upd_get_assignments(stmt).map_or(false, |a| !a.is_empty())
}

/// <list-update> ::= "UPDATE" <table> "__list" "SET" "value" "=" <new-value> "WHERE" "key" "=" <key> "AND" "index" "=" <index>
pub fn is_list_update(stmt: &Statement) -> bool {
    is_update(stmt) && is_list_table(stmt) && has_key_equals(stmt) && 
    has_field_equals(stmt, "index") && has_assignment(stmt, "value")
}

/// <zset-update> ::= "UPDATE" <table> "__zset" "SET" "score" "=" <new-score> "WHERE" "key" "=" <key> "AND" "member" "=" <member>
pub fn is_zset_update(stmt: &Statement) -> bool {
    is_update(stmt) && is_zset_table(stmt) && has_key_equals(stmt) && 
    has_field_equals(stmt, "member") && has_assignment(stmt, "score")
}