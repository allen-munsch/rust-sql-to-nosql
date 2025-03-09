// matchers/insert.rs - Predicate functions for INSERT statement pattern matching
// Each function tests if a statement matches a particular INSERT pattern from the BNF grammar

use sqlparser::ast::{Insert, ObjectNamePart, SetExpr, Statement, TableObject};

// --------------------------------
// INSERT Pattern Analysis Functions - Pure
// --------------------------------

/// Check if statement is an INSERT statement
pub fn is_insert(stmt: &Statement) -> bool {
    matches!(stmt, Statement::Insert(_))
}

/// Get the table name from an INSERT statement
pub fn get_table_name(stmt: &Statement) -> Option<String> {
    match stmt {
        Statement::Insert(Insert { table, .. }) => {
            match table {
                TableObject::TableName(name) => {
                    if !name.0.is_empty() {
                        match &name.0[0] {
                            ObjectNamePart::Identifier(ident) => {
                                // In a real implementation, we would check if this table name
                                // references a CTE defined in a WITH clause. Here we're just
                                // extracting the name for demonstration purposes.
                                Some(ident.value.clone())
                            }
                        }
                    } else {
                        None
                    }    
                },
                _ => None,
            }
        },
        _ => None,
    }
}

/// Check if INSERT statement target is a hash table
pub fn is_hash_table(stmt: &Statement) -> bool {
    get_table_name(stmt)
        .map(|name| name.ends_with("__hash"))
        .unwrap_or(false)
}

/// Check if INSERT statement target is a list table
pub fn is_list_table(stmt: &Statement) -> bool {
    get_table_name(stmt)
        .map(|name| name.ends_with("__list"))
        .unwrap_or(false)
}

/// Check if INSERT statement target is a set table
pub fn is_set_table(stmt: &Statement) -> bool {
    get_table_name(stmt)
        .map(|name| name.ends_with("__set"))
        .unwrap_or(false)
}

/// Check if INSERT statement target is a sorted set table
pub fn is_zset_table(stmt: &Statement) -> bool {
    get_table_name(stmt)
        .map(|name| name.ends_with("__zset"))
        .unwrap_or(false)
}

/// Check if INSERT statement target is a string table (default)
pub fn is_string_table(stmt: &Statement) -> bool {
    get_table_name(stmt)
        .map(|name| !name.ends_with("__hash") && !name.ends_with("__list") && 
                     !name.ends_with("__set") && !name.ends_with("__zset"))
        .unwrap_or(false)
}

/// Check if the INSERT statement has the required columns
pub fn has_columns(stmt: &Statement, required_columns: &[&str]) -> bool {
    match stmt {
        Statement::Insert(Insert { columns, .. }) => {
            // Convert all column names to lowercase for case-insensitive comparison
            let columns_lowercase: Vec<String> = columns.iter()
                .map(|ident| ident.value.to_lowercase())
                .collect();
            
            // Check if all required columns exist
            required_columns.iter()
                .map(|col| col.to_lowercase())
                .all(|required| columns_lowercase.contains(&required))
        },
        _ => false,
    }
}

/// Check if the INSERT statement has exactly the specified columns
pub fn has_exact_columns(stmt: &Statement, required_columns: &[&str]) -> bool {
    match stmt {
        Statement::Insert(Insert { columns, .. }) => {
            if columns.len() != required_columns.len() {
                return false;
            }
            
            // Convert all column names to lowercase for case-insensitive comparison
            let columns_lowercase: Vec<String> = columns.iter()
                .map(|ident| ident.value.to_lowercase())
                .collect();
            
            // Check if all required columns exist
            required_columns.iter()
                .map(|col| col.to_lowercase())
                .all(|required| columns_lowercase.contains(&required))
        },
        _ => false,
    }
}

/// Check if the INSERT statement has values
pub fn has_values(stmt: &Statement) -> bool {
    match stmt {
        Statement::Insert(Insert { source, .. }) => {
            source.as_ref().map_or(false, |query| {
                matches!(&*query.body, SetExpr::Values(values) if !values.rows.is_empty())
            })
        },
        _ => false,
    }
}

// --------------------------------
// BNF INSERT Rule Matchers - Direct mapping to BNF rules
// --------------------------------

/// <string-set> ::= "INSERT" "INTO" <table> "(key, value)" "VALUES" "(" <key> "," <value> ")"
pub fn is_string_set(stmt: &Statement) -> bool {
    is_insert(stmt) && is_string_table(stmt) && has_exact_columns(stmt, &["key", "value"]) && has_values(stmt)
}

/// <hash-set> ::= "INSERT" "INTO" <table> "__hash" "(key, field1 [, field2]...)" "VALUES" "(" <key> "," <value1> [, <value2>]... ")"
pub fn is_hash_set(stmt: &Statement) -> bool {
    is_insert(stmt) && is_hash_table(stmt) && has_columns(stmt, &["key"]) && has_values(stmt)
}

/// <list-push> ::= "INSERT" "INTO" <table> "__list" "(key, value)" "VALUES" "(" <key> "," <value> ")"
pub fn is_list_push(stmt: &Statement) -> bool {
    is_insert(stmt) && is_list_table(stmt) && has_exact_columns(stmt, &["key", "value"]) && has_values(stmt)
}

/// <set-add> ::= "INSERT" "INTO" <table> "__set" "(key, member)" "VALUES" "(" <key> "," <member> ")"
pub fn is_set_add(stmt: &Statement) -> bool {
    is_insert(stmt) && is_set_table(stmt) && has_exact_columns(stmt, &["key", "member"]) && has_values(stmt)
}

/// <zset-add> ::= "INSERT" "INTO" <table> "__zset" "(key, member, score)" "VALUES" "(" <key> "," <member> "," <score> ")"
pub fn is_zset_add(stmt: &Statement) -> bool {
    is_insert(stmt) && is_zset_table(stmt) && has_exact_columns(stmt, &["key", "member", "score"]) && has_values(stmt)
}