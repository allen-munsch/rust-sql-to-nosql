// matchers/common.rs - Common predicate functions for SQL pattern matching
// Shared utility functions for pattern matching across statement types

use sqlparser::ast::{
    Expr, ObjectNamePart, OrderByKind, Query, Select, SelectItem, SetExpr, Statement, TableFactor
};
use crate::pattern::combinators::{Pattern, and_then, extract};

/// Check if a table name represents a Redis Hash table
pub fn is_hash_table_name(name: &str) -> bool {
    name.ends_with("__hash")
}

/// Check if a table name represents a Redis List table
pub fn is_list_table_name(name: &str) -> bool {
    name.ends_with("__list")
}

/// Check if a table name represents a Redis Set table
pub fn is_set_table_name(name: &str) -> bool {
    name.ends_with("__set")
}

/// Check if a table name represents a Redis Sorted Set table
pub fn is_zset_table_name(name: &str) -> bool {
    name.ends_with("__zset")
}

/// Check if a table name represents a Redis String table (default)
pub fn is_string_table_name(name: &str) -> bool {
    !is_hash_table_name(name) && 
    !is_list_table_name(name) && 
    !is_set_table_name(name) && 
    !is_zset_table_name(name)
}

/// Determine the Redis data type from a table name
#[derive(Debug, Clone, PartialEq)]
pub enum RedisDataType {
    String,
    Hash,
    List,
    Set,
    SortedSet,
}

pub fn get_redis_data_type(table_name: &str) -> RedisDataType {
    if is_hash_table_name(table_name) {
        RedisDataType::Hash
    } else if is_list_table_name(table_name) {
        RedisDataType::List
    } else if is_set_table_name(table_name) {
        RedisDataType::Set
    } else if is_zset_table_name(table_name) {
        RedisDataType::SortedSet
    } else {
        RedisDataType::String
    }
}

/// Pattern that matches a SELECT statement
pub fn select_statement() -> impl Pattern<Statement, Box<Select>> {
    extract(|stmt: &Statement| {
        match stmt {
            Statement::Query(query) => {
                match &*query.body {
                    SetExpr::Select(select) => Some(select.clone()),
                    _ => None,
                }
            },
            _ => None,
        }
    })
}

/// Pattern that matches a query with a wildcard projection
pub fn wildcard_select() -> impl Pattern<Statement, Box<Select>> {
    and_then(
        select_statement(),
        extract(|select: &Box<Select>| {
            if select.projection.len() == 1 {
                match &select.projection[0] {
                    SelectItem::Wildcard(_) => Some(select.clone()),
                    _ => None,
                }
            } else {
                None
            }
        })
    )
}

/// Pattern that matches a single field select
pub fn single_field_select() -> impl Pattern<Statement, (Box<Select>, String)> {
    and_then(
        select_statement(),
        extract(|select: &Box<Select>| {
            if select.projection.len() == 1 {
                match &select.projection[0] {
                    SelectItem::UnnamedExpr(Expr::Identifier(ident)) => {
                        Some((select.clone(), ident.value.clone()))
                    },
                    _ => None,
                }
            } else {
                None
            }
        })
    )
}

/// Pattern that matches a multi-field select
pub fn multi_field_select() -> impl Pattern<Statement, (Box<Select>, Vec<String>)> {
    and_then(
        select_statement(),
        extract(|select: &Box<Select>| {
            if select.projection.len() > 1 {
                let field_names: Vec<String> = select.projection.iter()
                    .filter_map(|item| {
                        match item {
                            SelectItem::UnnamedExpr(Expr::Identifier(ident)) => {
                                Some(ident.value.clone())
                            },
                            _ => None,
                        }
                    })
                    .collect();
                
                if field_names.len() == select.projection.len() {
                    Some((select.clone(), field_names))
                } else {
                    None
                }
            } else {
                None
            }
        })
    )
}

/// Pattern that matches a table with a specific suffix
pub fn table_with_suffix(suffix: &'static str) -> impl Pattern<TableFactor, String> {
    extract(move |table: &TableFactor| {
        match table {
            TableFactor::Table { name, .. } => {
                if !name.0.is_empty() {
                    match &name.0[0] {
                        ObjectNamePart::Identifier(ident) => {
                            let table_name = &ident.value;
                            if table_name.ends_with(suffix) {
                                Some(table_name.clone())
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    None
                }
            },
            _ => None,
        }
    })
}

/// Pattern that matches a hash table
pub fn hash_table() -> impl Pattern<TableFactor, String> {
    table_with_suffix("__hash")
}

/// Pattern that matches a list table
pub fn list_table() -> impl Pattern<TableFactor, String> {
    table_with_suffix("__list")
}

/// Pattern that matches a set table
pub fn set_table() -> impl Pattern<TableFactor, String> {
    table_with_suffix("__set")
}

/// Pattern that matches a sorted set table
pub fn zset_table() -> impl Pattern<TableFactor, String> {
    table_with_suffix("__zset")
}

pub fn string_table() -> impl Pattern<TableFactor, String> {
    extract(|table: &TableFactor| {
        match table {
            TableFactor::Table { name, .. } => {
                if !name.0.is_empty() {
                    match &name.0[0] {
                        ObjectNamePart::Identifier(ident) => {
                            let table_name = &ident.value;
                            if !table_name.ends_with("__hash") &&
                               !table_name.ends_with("__list") &&
                               !table_name.ends_with("__set") &&
                               !table_name.ends_with("__zset") {
                                Some(table_name.clone())
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    None
                }
            },
            _ => None,
        }
    })
 }

pub fn key_equals() -> impl Pattern<Expr, String> {
    extract(|expr: &Expr| {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                if *op == sqlparser::ast::BinaryOperator::Eq {
                    match &**left {
                        Expr::Identifier(ident) if ident.value.to_lowercase() == "key" => {
                            match &**right {
                                Expr::Value(value_with_span) => match &value_with_span.value {
                                    sqlparser::ast::Value::SingleQuotedString(s) |
                                    sqlparser::ast::Value::DoubleQuotedString(s) => {
                                        Some(s.clone())
                                    },
                                    sqlparser::ast::Value::Number(n, _) => {
                                        Some(n.clone())
                                    },
                                    _ => None,
                                },
                                _ => None,
                            }
                        },
                        _ => None,
                    }
                } else {
                    None
                }
            },
            _ => None,
        }
    })
 }


/// Pattern that matches a field equality condition
pub fn field_equals(field_name: &'static str) -> impl Pattern<Expr, String> {
    extract(move |expr: &Expr| {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                if *op == sqlparser::ast::BinaryOperator::Eq {
                    match &**left {
                        Expr::Identifier(ident) if ident.value.to_lowercase() == field_name.to_lowercase() => {
                            match &**right {
                                Expr::Value(value_with_span) => match &value_with_span.value {
                                    sqlparser::ast::Value::SingleQuotedString(s) |
                                    sqlparser::ast::Value::DoubleQuotedString(s) => {
                                        Some(s.clone())
                                    },
                                    sqlparser::ast::Value::Number(n, _) => {
                                        Some(n.clone())
                                    },
                                    _ => None,
                                },
                                _ => None,
                            }
                        },
                        _ => None,
                    }
                } else {
                    None
                }
            },
            _ => None,
        }
    })
}

pub fn score_range() -> impl Pattern<Expr, (String, String)> {
    extract(|expr: &Expr| {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                match &**left {
                    Expr::Identifier(ident) if ident.value.to_lowercase() == "score" => {
                        match op {
                            sqlparser::ast::BinaryOperator::Gt => {
                                if let Expr::Value(value_with_span) = &**right {
                                    if let sqlparser::ast::Value::Number(n, _) = &value_with_span.value {
                                        Some((format!("({}", n), "+inf".to_string()))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            },
                            sqlparser::ast::BinaryOperator::GtEq => {
                                if let Expr::Value(value_with_span) = &**right {
                                    if let sqlparser::ast::Value::Number(n, _) = &value_with_span.value {
                                        Some((n.clone(), "+inf".to_string()))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            },
                            sqlparser::ast::BinaryOperator::Lt => {
                                if let Expr::Value(value_with_span) = &**right {
                                    if let sqlparser::ast::Value::Number(n, _) = &value_with_span.value {
                                        Some(("-inf".to_string(), format!("({}",n)))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            },
                            sqlparser::ast::BinaryOperator::LtEq => {
                                if let Expr::Value(value_with_span) = &**right {
                                    if let sqlparser::ast::Value::Number(n, _) = &value_with_span.value {
                                        Some(("-inf".to_string(), n.clone()))
                                    } else {
                                        None
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
            },
            _ => None,
        }
    })
 }


/// Pattern that matches an ORDER BY score DESC clause
pub fn order_by_score_desc() -> impl Pattern<Query, ()> {
    extract(|query: &Query| {
        if query.order_by.is_none() {
            return None;
        }
        
        let order_by = query.order_by.as_ref().unwrap();
        
        match &order_by.kind {
            OrderByKind::Expressions(exprs) => {
                if exprs.is_empty() {
                    return None;
                }
                
                let order_expr = &exprs[0];
                match &order_expr.expr {
                    Expr::Identifier(ident) => {
                        if ident.value.to_lowercase() == "score" && 
                           order_expr.options.asc.is_some() && 
                           !order_expr.options.asc.unwrap() {
                            Some(())
                        } else {
                            None
                        }
                    },
                    _ => None,
                }
            },
            _ => None,
        }
    })
}

/// Pattern that matches a LIMIT clause
pub fn has_limit() -> impl Pattern<Query, u64> {
    extract(|query: &Query| {
        query.limit.as_ref().and_then(|limit| {
            match limit {
                Expr::Value(value_with_span) => {
                    if let sqlparser::ast::Value::Number(n, _) = &value_with_span.value {
                        n.parse::<u64>().ok()
                    } else {
                        None
                    }
                },
                _ => None,
            }
        })
    })
}

// --------------------------------
// BNF Rule Patterns - Direct mapping to BNF rules
// --------------------------------

/// <string-get> ::= "SELECT" "*" "FROM" <table> "WHERE" "key" "=" <value>
pub fn string_get() -> impl Pattern<Statement, String> {
    extract(|stmt: &Statement| {
        // First match a wildcard select
        let select = match wildcard_select().match_pattern(stmt) {
            Ok(select) => select,
            Err(_) => return None,
        };
        
        // Check for a string table
        if select.from.is_empty() {
            return None;
        }
        
        let is_string_table = match &select.from[0].relation {
            table => string_table().match_pattern(table).is_ok(),
        };
        
        if !is_string_table {
            return None;
        }
        
        // Check for a key equals condition
        if let Some(where_clause) = &select.selection {
            match key_equals().match_pattern(where_clause) {
                Ok(key) => Some(key),
                Err(_) => None,
            }
        } else {
            None
        }
    })
}

/// <hash-getall> ::= "SELECT" "*" "FROM" <table> "__hash" "WHERE" "key" "=" <value>
pub fn hash_getall() -> impl Pattern<Statement, String> {
    extract(|stmt: &Statement| {
        // First match a wildcard select
        let select = match wildcard_select().match_pattern(stmt) {
            Ok(select) => select,
            Err(_) => return None,
        };
        
        // Check for a hash table
        if select.from.is_empty() {
            return None;
        }
        
        let is_hash_table = match &select.from[0].relation {
            table => hash_table().match_pattern(table).is_ok(),
        };
        
        if !is_hash_table {
            return None;
        }
        
        // Check for a key equals condition
        if let Some(where_clause) = &select.selection {
            match key_equals().match_pattern(where_clause) {
                Ok(key) => Some(key),
                Err(_) => None,
            }
        } else {
            None
        }
    })
}