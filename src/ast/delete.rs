// ast/delete.rs - Pure functions for DELETE AST node extraction
// These functions don't modify state, just extract information from DELETE statements

use sqlparser::ast::{Expr, FromTable, ObjectNamePart, Statement, TableFactor, Value};
use std::collections::HashMap;

/// Get the table name from a DELETE statement
pub fn get_table_name(stmt: &Statement) -> Option<String> {
    match stmt {
        Statement::Delete(delete) => {
            match &delete.from {
                FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => {
                    if tables.is_empty() {
                        return None;
                    }
                    match &tables[0].relation {
                        TableFactor::Table { name, .. } => {
                            if !name.0.is_empty() {
                                match &name.0[0] {
                                    ObjectNamePart::Identifier(ident) => Some(ident.value.clone())
                                }
                            } else {
                                None
                            }
                        },
                        _ => None
                    }
                }
            }
        },
        _ => None,
    }
}

/// Get key condition from WHERE clause
pub fn get_key_value(stmt: &Statement) -> Option<String> {
    match stmt {
        Statement::Delete(delete) => {
            delete.selection.as_ref().and_then(|expr| {
                match expr {
                    Expr::BinaryOp { left, op, right } => {
                        if *op != sqlparser::ast::BinaryOperator::Eq {
                            return None;
                        }
                        
                        match &**left {
                            Expr::Identifier(ident) if ident.value.to_lowercase() == "key" => {
                                match &**right {
                                    Expr::Value(value_with_span) => match &value_with_span.value {
                                        Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) => Some(s.clone()),
                                        Value::Number(n, _) => Some(n.clone()),
                                        _ => None,
                                    },
                                    _ => None,
                                }
                            }
                            _ => None,
                        }
                    },
                    _ => None,
                }
            })
        },
        _ => None,
    }
}

/// Get field condition from WHERE clause
pub fn get_field_filter(stmt: &Statement, field_name: &str) -> Option<String> {
    match stmt {
        Statement::Delete(delete) => {
            delete.selection.as_ref().and_then(|expr| {
                extract_field_condition(expr, field_name)
            })
        },
        _ => None,
    }
}

/// Extract a field=value condition from a WHERE clause expression
pub fn extract_field_condition(expr: &Expr, field_name: &str) -> Option<String> {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            match op {
                sqlparser::ast::BinaryOperator::Eq => {
                    match &**left {
                        Expr::Identifier(ident) if ident.value.to_lowercase() == field_name.to_lowercase() => {
                            match &**right {
                                Expr::Value(value_with_span) => match &value_with_span.value {
                                    Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) => Some(s.clone()),
                                    Value::Number(n, _) => Some(n.clone()),
                                    _ => None,
                                },
                                _ => None,
                            }
                        }
                        _ => None,
                    }
                },
                sqlparser::ast::BinaryOperator::And => {
                    // Try left side first, then right side
                    extract_field_condition(left, field_name).or_else(|| extract_field_condition(right, field_name))
                },
                _ => None,
            }
        },
        _ => None,
    }
}

/// Extract all conditions from a WHERE clause
pub fn extract_conditions(expr: &Option<Expr>) -> HashMap<String, String> {
    let mut conditions = HashMap::new();
    
    if let Some(expr) = expr.as_ref() {
        extract_conditions_recursive(expr, &mut conditions);
    }
    
    conditions
}

/// Recursively extract conditions from a complex WHERE clause
fn extract_conditions_recursive(expr: &Expr, conditions: &mut HashMap<String, String>) {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            match op {
                sqlparser::ast::BinaryOperator::And => {
                    // Process both sides of AND
                    extract_conditions_recursive(left, conditions);
                    extract_conditions_recursive(right, conditions);
                },
                sqlparser::ast::BinaryOperator::Eq => {
                    // Process equals condition
                    if let Expr::Identifier(ident) = &**left {
                        if let Expr::Value(value_with_span) = &**right {
                            match &value_with_span.value {
                                Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) => {
                                    conditions.insert(ident.value.clone(), s.clone());
                                },
                                Value::Number(n, _) => {
                                    conditions.insert(ident.value.clone(), n.clone());
                                },
                                _ => {}
                            }
                        }
                    }
                },
                _ => {}
            }
        },
        _ => {}
    }
}