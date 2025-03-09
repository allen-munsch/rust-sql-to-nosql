// ast/select.rs - Pure functions for SELECT AST node extraction
// These functions don't modify state, just extract or test information

use sqlparser::ast::{
    BinaryOperator, Expr, ObjectNamePart, OrderByKind, Query, Select, SelectItem, SetExpr, Statement, TableFactor, Value
};

/// Get the query from a statement, if it exists
pub fn sel_get_query(stmt: &Statement) -> Option<&Query> {
    match stmt {
        Statement::Query(query) => Some(query),
        _ => None,
    }
}

/// Get the select from a query, if it exists
pub fn sel_get_select(query: &Query) -> Option<&Select> {
    match &*query.body {
        SetExpr::Select(select) => Some(select),
        _ => None,
    }
}

/// Get the table name from a select statement, if it exists
pub fn sel_get_table_name(select: &Select) -> Option<String> {
    select.from.get(0).and_then(|table_with_joins| {
        match &table_with_joins.relation {
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
    })
}

/// Get a "key" value from a binary expression where "key = value"
pub fn sel_get_key_value(expr: &Option<Expr>) -> Option<String> {
    expr.as_ref().and_then(|expr| {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                if *op != BinaryOperator::Eq {
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
                    },
                    _ => None,
                }
            },
            _ => None,
        }
    })
}

/// Get a field name from a SelectItem, if it exists
pub fn sel_get_field_name(select_item: &SelectItem) -> Option<String> {
    match select_item {
        SelectItem::UnnamedExpr(Expr::Identifier(ident)) => Some(ident.value.clone()),
        _ => None,
    }
}

/// Get multiple field names from a list of SelectItems
pub fn sel_get_field_names(select_items: &[SelectItem]) -> Vec<String> {
    select_items.iter()
        .filter_map(sel_get_field_name)
        .collect()
}

/// Check if a SelectItem is a wildcard (*) selector
pub fn sel_is_wildcard(select_item: &SelectItem) -> bool {
    matches!(select_item, SelectItem::Wildcard(_))
}

/// Get a field value from a binary expression with "field = value"
pub fn sel_get_field_filter(expr: &Option<Expr>, field_name: &str) -> Option<String> {
    expr.as_ref().and_then(|expr| {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                if *op != BinaryOperator::Eq {
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
            }
            _ => None,
        }
    })
}

/// Get the limit value from a query, if it exists
/// Get the limit value from a query, if it exists
pub fn sel_get_limit(query: &Query) -> Option<u64> {
    query.limit.as_ref().and_then(|limit| {
        match limit {
            Expr::Value(value_with_span) => match &value_with_span.value {
                Value::Number(n, _) => n.parse::<u64>().ok(),
                _ => None,
            },
            _ => None,
        }
    })
}

/// Extract score comparison from a binary expression
pub fn sel_get_score_range(expr: &Option<Expr>) -> Option<(String, String)> {
    expr.as_ref().and_then(|expr| {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                match &**left {
                    Expr::Identifier(ident) if ident.value.to_lowercase() == "score" => {
                        match op {
                            BinaryOperator::Gt => {
                                if let Expr::Value(value_with_span) = &**right {
                                    if let Value::Number(n, _) = &value_with_span.value {
                                        Some((format!("({}",n), "+inf".to_string()))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            },
                            BinaryOperator::GtEq => {
                                if let Expr::Value(value_with_span) = &**right {
                                    if let Value::Number(n, _) = &value_with_span.value {
                                        Some((n.clone(), "+inf".to_string()))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            },
                            BinaryOperator::Lt => {
                                if let Expr::Value(value_with_span) = &**right {
                                    if let Value::Number(n, _) = &value_with_span.value {
                                        Some(("-inf".to_string(), format!("({}",n)))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            },
                            BinaryOperator::LtEq => {
                                if let Expr::Value(value_with_span) = &**right {
                                    if let Value::Number(n, _) = &value_with_span.value {
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
            // Could add BETWEEN handling here
            _ => None,
        }
    })
}

/// Check if a SQL statement represents a query ordering by score in descending order
pub fn sel_is_order_by_score_desc(query: &Query) -> bool {
    match &query.order_by {
        Some(order_by) => {
            match &order_by.kind {
                OrderByKind::Expressions(exprs) => {
                    exprs.iter().any(|order_expr| {
                        match &order_expr.expr {
                            Expr::Identifier(ident) if ident.value.to_lowercase() == "score" => {
                                order_expr.options.asc.map_or(false, |v| !v)
                            },
                            _ => false,
                        }
                    })
                },
                _ => false,
            }
        },
        None => false,
    }
}