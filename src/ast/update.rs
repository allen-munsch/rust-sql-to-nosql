// ast/update.rs - Pure functions for UPDATE AST node extraction
// These functions don't modify state, just extract information from UPDATE statements

use sqlparser::ast::{AssignmentTarget, Expr, ObjectNamePart, Statement, TableFactor, Value};
use std::collections::HashMap;

/// Extract a string value from an SQL expression
pub fn upd_extract_value(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Value(value_with_span) => match &value_with_span.value {
            Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) => Some(s.clone()),
            Value::Number(n, _) => Some(n.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// Get key condition from WHERE clause
pub fn upd_get_key_value(stmt: &Statement) -> Option<String> {
    match stmt {
        Statement::Update { selection, .. } => {
            selection.as_ref().and_then(|expr| {
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

/// Get a specific field condition from WHERE clause
pub fn upd_get_field_filter(stmt: &Statement, field_name: &str) -> Option<String> {
    match stmt {
        Statement::Update { selection, .. } => {
            selection.as_ref().and_then(|expr| {
                match expr {
                    Expr::BinaryOp { left, op, right } => {
                        if *op != sqlparser::ast::BinaryOperator::Eq {
                            return None;
                        }
                        
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
                    _ => None,
                }
            })
        },
        _ => None,
    }
}


/// Extract all SET assignments as field-value pairs
pub fn upd_get_assignments(stmt: &Statement) -> Option<HashMap<String, String>> {
    match stmt {
        Statement::Update { assignments, .. } => {
            let mut field_values = HashMap::new();
            
            for assignment in assignments {
                if let AssignmentTarget::ColumnName(object_name) = &assignment.target {
                    // Assuming the first part of the object name is the column name
                    if !object_name.0.is_empty() {
                        let ObjectNamePart::Identifier(ident) = &object_name.0[0];
                        if let Some(value) = upd_extract_value(&assignment.value) {
                                field_values.insert(ident.value.clone(), value);
                            }
                    }
                }
            }
            
            if field_values.is_empty() {
                None
            } else {
                Some(field_values)
            }
        },
        _ => None,
    }
}

/// Get the table name from an UPDATE statement
pub fn upd_get_table_name(stmt: &Statement) -> Option<String> {
    match stmt {
        Statement::Update { table, .. } => {
            match &table.relation {
                TableFactor::Table { name, .. } => {
                    if !name.0.is_empty() {
                        match &name.0[0] {
                            ObjectNamePart::Identifier(ident) => Some(ident.value.clone())
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