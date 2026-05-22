// ast/update.rs - Pure functions for UPDATE AST node extraction
// These functions don't modify state, just extract information from UPDATE statements

use sqlparser::ast::{AssignmentTarget, Expr, ObjectNamePart, Statement, TableFactor, Value};

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

/// Get key condition from WHERE clause (recursively searches AND expressions)
pub fn upd_get_key_value(stmt: &Statement) -> Option<String> {
    match stmt {
        Statement::Update { selection, .. } => {
            selection.as_ref().and_then(upd_extract_key_from_expr)
        },
        _ => None,
    }
}

/// Recursively extract key = value from an expression (handles AND)
fn upd_extract_key_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            match op {
                sqlparser::ast::BinaryOperator::Eq => {
                    match &**left {
                        Expr::Identifier(ident) if ident.value.to_lowercase() == "key" => {
                            upd_extract_value(right)
                        }
                        _ => None,
                    }
                }
                sqlparser::ast::BinaryOperator::And => {
                    upd_extract_key_from_expr(left)
                        .or_else(|| upd_extract_key_from_expr(right))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Get a specific field condition from WHERE clause (recursively searches AND expressions)
pub fn upd_get_field_filter(stmt: &Statement, field_name: &str) -> Option<String> {
    match stmt {
        Statement::Update { selection, .. } => {
            selection.as_ref().and_then(|expr| upd_extract_field_from_expr(expr, field_name))
        },
        _ => None,
    }
}

/// Recursively extract field = value from an expression (handles AND)
fn upd_extract_field_from_expr(expr: &Expr, field_name: &str) -> Option<String> {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            match op {
                sqlparser::ast::BinaryOperator::Eq => {
                    match &**left {
                        Expr::Identifier(ident) if ident.value.to_lowercase() == field_name.to_lowercase() => {
                            upd_extract_value(right)
                        }
                        _ => None,
                    }
                }
                sqlparser::ast::BinaryOperator::And => {
                    upd_extract_field_from_expr(left, field_name)
                        .or_else(|| upd_extract_field_from_expr(right, field_name))
                }
                _ => None,
            }
        }
        _ => None,
    }
}


/// Extract all SET assignments as ordered field-value pairs (preserves SQL order)
pub fn upd_get_assignments(stmt: &Statement) -> Option<Vec<(String, String)>> {
    match stmt {
        Statement::Update { assignments, .. } => {
            let mut field_values = Vec::new();
            
            for assignment in assignments {
                if let AssignmentTarget::ColumnName(object_name) = &assignment.target {
                    if !object_name.0.is_empty() {
                        let ObjectNamePart::Identifier(ident) = &object_name.0[0];
                        if let Some(value) = upd_extract_value(&assignment.value) {
                            field_values.push((ident.value.clone(), value));
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