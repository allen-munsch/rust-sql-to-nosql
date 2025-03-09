// matchers/delete.rs - Predicate functions for DELETE statement pattern matching
// Each function tests if a statement matches a particular DELETE pattern from the BNF grammar
use sqlparser::ast::Statement;
use crate::ast;

// --------------------------------
// Basic Pattern Matchers - Pure Functions
// --------------------------------

/// Check if statement is a DELETE statement
pub fn is_delete(stmt: &Statement) -> bool {
    matches!(stmt, Statement::Delete { .. })
}

/// Check if DELETE statement target is a hash table
pub fn is_hash_table(stmt: &Statement) -> bool {
    match ast::delete::get_table_name(stmt) {
        Some(name) => name.ends_with("__hash"),
        None => false,
    }
}

/// Check if DELETE statement target is a list table
pub fn is_list_table(stmt: &Statement) -> bool {
    match ast::delete::get_table_name(stmt) {
        Some(name) => name.ends_with("__list"),
        None => false,
    }
}

/// Check if DELETE statement target is a set table
pub fn is_set_table(stmt: &Statement) -> bool {
    match ast::delete::get_table_name(stmt) {
        Some(name) => name.ends_with("__set"),
        None => false,
    }
}

/// Check if DELETE statement target is a sorted set table
pub fn is_zset_table(stmt: &Statement) -> bool {
    match ast::delete::get_table_name(stmt) {
        Some(name) => name.ends_with("__zset"),
        None => false,
    }
}

/// Check if DELETE statement target is a string table (default)
pub fn is_string_table(stmt: &Statement) -> bool {
    match ast::delete::get_table_name(stmt) {
        Some(name) => !name.ends_with("__hash") && !name.ends_with("__list") && 
                     !name.ends_with("__set") && !name.ends_with("__zset"),
        None => false,
    }
}

/// Check if the DELETE statement has a key condition
pub fn has_key_equals(stmt: &Statement) -> bool {
    ast::delete::get_key_value(stmt).is_some()
}

/// Check if the DELETE statement has a field condition
pub fn has_field_equals(stmt: &Statement, field_name: &str) -> bool {
    ast::delete::get_field_filter(stmt, field_name).is_some()
}

// --------------------------------
// BNF Rule Matchers - Direct mapping to BNF rules
// --------------------------------

/// <string-delete> ::= "DELETE" "FROM" <table> "WHERE" "key" "=" <key>
pub fn is_string_delete(stmt: &Statement) -> bool {
    is_delete(stmt) && is_string_table(stmt) && has_key_equals(stmt)
}

/// <hash-delete> ::= "DELETE" "FROM" <table> "__hash" "WHERE" "key" "=" <key>
pub fn is_hash_delete(stmt: &Statement) -> bool {
    is_delete(stmt) && is_hash_table(stmt) && has_key_equals(stmt) && !has_field_equals(stmt, "field")
}

/// <hash-delete-field> ::= "DELETE" "FROM" <table> "__hash" "WHERE" "key" "=" <key> "AND" "field" "=" <field>
pub fn is_hash_delete_field(stmt: &Statement) -> bool {
    is_delete(stmt) && is_hash_table(stmt) && has_key_equals(stmt) && has_field_equals(stmt, "field")
}

/// <list-delete> ::= "DELETE" "FROM" <table> "__list" "WHERE" "key" "=" <key>
pub fn is_list_delete(stmt: &Statement) -> bool {
    is_delete(stmt) && is_list_table(stmt) && has_key_equals(stmt) && !has_field_equals(stmt, "value")
}

/// <list-delete-value> ::= "DELETE" "FROM" <table> "__list" "WHERE" "key" "=" <key> "AND" "value" "=" <value>
pub fn is_list_delete_value(stmt: &Statement) -> bool {
    is_delete(stmt) && is_list_table(stmt) && has_key_equals(stmt) && has_field_equals(stmt, "value")
}

/// <set-delete> ::= "DELETE" "FROM" <table> "__set" "WHERE" "key" "=" <key>
pub fn is_set_delete(stmt: &Statement) -> bool {
    is_delete(stmt) && is_set_table(stmt) && has_key_equals(stmt) && !has_field_equals(stmt, "member")
}

/// <set-delete-member> ::= "DELETE" "FROM" <table> "__set" "WHERE" "key" "=" <key> "AND" "member" "=" <member>
pub fn is_set_delete_member(stmt: &Statement) -> bool {
    is_delete(stmt) && is_set_table(stmt) && has_key_equals(stmt) && has_field_equals(stmt, "member")
}

/// <zset-delete> ::= "DELETE" "FROM" <table> "__zset" "WHERE" "key" "=" <key>
pub fn is_zset_delete(stmt: &Statement) -> bool {
    is_delete(stmt) && is_zset_table(stmt) && has_key_equals(stmt) && !has_field_equals(stmt, "member")
}

/// <zset-delete-member> ::= "DELETE" "FROM" <table> "__zset" "WHERE" "key" "=" <key> "AND" "member" "=" <member>
pub fn is_zset_delete_member(stmt: &Statement) -> bool {
    is_delete(stmt) && is_zset_table(stmt) && has_key_equals(stmt) && has_field_equals(stmt, "member")
}
