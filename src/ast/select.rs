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
/// Recursively handles AND expressions
pub fn sel_get_key_value(expr: &Option<Expr>) -> Option<String> {
    expr.as_ref().and_then(sel_extract_key_from_expr)
}

/// Recursively extract key = value from an expression (handles AND)
fn sel_extract_key_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            match op {
                BinaryOperator::Eq => {
                    match &**left {
                        Expr::Identifier(ident) if ident.value.to_lowercase() == "key" => {
                            sel_extract_value(right)
                        }
                        _ => None,
                    }
                }
                BinaryOperator::And => {
                    sel_extract_key_from_expr(left)
                        .or_else(|| sel_extract_key_from_expr(right))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Extract a string value from an expression
fn sel_extract_value(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Value(value_with_span) => match &value_with_span.value {
            Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) => Some(s.clone()),
            Value::Number(n, _) => Some(n.clone()),
            _ => None,
        },
        _ => None,
    }
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

/// Check if the SELECT is a COUNT(*) aggregation
pub fn sel_is_count_star(select: &Select) -> bool {
    if select.projection.len() != 1 {
        return false;
    }
    match &select.projection[0] {
        SelectItem::UnnamedExpr(Expr::Function(func)) => {
            let name = func.name.to_string().to_uppercase();
            if name != "COUNT" {
                return false;
            }
            // Check that the argument is * (wildcard)
            match &func.args {
                sqlparser::ast::FunctionArguments::List(list) => {
                    if list.args.len() != 1 {
                        return false;
                    }
                    matches!(&list.args[0],
                        sqlparser::ast::FunctionArg::Unnamed(
                            sqlparser::ast::FunctionArgExpr::Wildcard
                        )
                    )
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Aggregate function info extracted from a SELECT projection
#[derive(Debug, Clone)]
pub struct AggregateInfo {
    pub function: String,       // AVG, SUM, MIN, MAX, STDDEV_POP, etc.
    pub field: Option<String>,  // The field being aggregated (None for COUNT(*))
    pub is_wildcard: bool,      // True for COUNT(*)
}

/// Try to extract an aggregate function from the SELECT projection.
/// Returns None if the projection is not a simple aggregate function call.
pub fn sel_get_aggregate(select: &Select) -> Option<AggregateInfo> {
    if select.projection.len() != 1 {
        return None;
    }
    match &select.projection[0] {
        SelectItem::UnnamedExpr(Expr::Function(func)) => {
            let name = func.name.to_string().to_uppercase();
            match &func.args {
                sqlparser::ast::FunctionArguments::List(list) => {
                    if list.args.len() != 1 {
                        return None;
                    }
                    match &list.args[0] {
                        sqlparser::ast::FunctionArg::Unnamed(
                            sqlparser::ast::FunctionArgExpr::Wildcard
                        ) => {
                            if name == "COUNT" {
                                Some(AggregateInfo { function: name, field: None, is_wildcard: true })
                            } else {
                                None // AVG(*), SUM(*) etc don't make sense
                            }
                        }
                        sqlparser::ast::FunctionArg::Unnamed(
                            sqlparser::ast::FunctionArgExpr::Expr(ref expr)
                        ) => {
                            let field = match expr {
                                Expr::Identifier(ident) => Some(ident.value.clone()),
                                _ => None,
                            };
                            // Only support known aggregate functions
                            match name.as_str() {
                                "AVG" | "SUM" | "MIN" | "MAX" | "STDDEV_POP" | "STDDEV_SAMP" | "VARIANCE" => {
                                    Some(AggregateInfo { function: name, field, is_wildcard: false })
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Check if the WHERE clause has a BETWEEN condition for score
pub fn sel_get_score_between(expr: &Option<Expr>) -> Option<(String, String)> {
    expr.as_ref().and_then(sel_extract_score_between_from_expr)
}

fn sel_extract_score_between_from_expr(expr: &Expr) -> Option<(String, String)> {
    match expr {
        Expr::Between { expr: inner, negated, low, high } => {
            if *negated {
                return None;
            }
            match &**inner {
                Expr::Identifier(ident) if ident.value.to_lowercase() == "score" => {
                    let min = sel_extract_value(low)?;
                    let max = sel_extract_value(high)?;
                    Some((min, max))
                }
                _ => None,
            }
        }
        Expr::BinaryOp { left, op: BinaryOperator::And, right } => {
            sel_extract_score_between_from_expr(left)
                .or_else(|| sel_extract_score_between_from_expr(right))
        }
        _ => None,
    }
}

/// Check if the WHERE clause has an index < value condition (for list range)
pub fn sel_get_index_lt(expr: &Option<Expr>) -> Option<String> {
    expr.as_ref().and_then(sel_extract_index_lt_from_expr)
}

fn sel_extract_index_lt_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            match op {
                BinaryOperator::Lt | BinaryOperator::LtEq => {
                    match &**left {
                        Expr::Identifier(ident) if ident.value.to_lowercase() == "index" => {
                            sel_extract_value(right)
                        }
                        _ => None,
                    }
                }
                BinaryOperator::And => {
                    sel_extract_index_lt_from_expr(left)
                        .or_else(|| sel_extract_index_lt_from_expr(right))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Get a field value from a binary expression with "field = value"
/// Recursively handles AND expressions
pub fn sel_get_field_filter(expr: &Option<Expr>, field_name: &str) -> Option<String> {
    expr.as_ref().and_then(|e| sel_extract_field_from_expr(e, field_name))
}

/// Recursively extract field = value from an expression (handles AND)
fn sel_extract_field_from_expr(expr: &Expr, field_name: &str) -> Option<String> {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            match op {
                BinaryOperator::Eq => {
                    match &**left {
                        Expr::Identifier(ident) if ident.value.to_lowercase() == field_name.to_lowercase() => {
                            sel_extract_value(right)
                        }
                        _ => None,
                    }
                }
                BinaryOperator::And => {
                    sel_extract_field_from_expr(left, field_name)
                        .or_else(|| sel_extract_field_from_expr(right, field_name))
                }
                _ => None,
            }
        }
        _ => None,
    }
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
/// Recursively handles AND expressions (common with key = value pairs)
pub fn sel_get_score_range(expr: &Option<Expr>) -> Option<(String, String)> {
    expr.as_ref().and_then(sel_extract_score_range_from_expr)
}

fn sel_extract_score_range_from_expr(expr: &Expr) -> Option<(String, String)> {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            // Check for AND: the score condition might be on either side
            if *op == BinaryOperator::And {
                return sel_extract_score_range_from_expr(left)
                    .or_else(|| sel_extract_score_range_from_expr(right));
            }

            match &**left {
                Expr::Identifier(ident) if ident.value.to_lowercase() == "score" => {
                    match op {
                        BinaryOperator::Gt => {
                            if let Expr::Value(value_with_span) = &**right {
                                if let Value::Number(n, _) = &value_with_span.value {
                                    return Some((format!("({}", n), "+inf".to_string()));
                                }
                            }
                            None
                        }
                        BinaryOperator::GtEq => {
                            if let Expr::Value(value_with_span) = &**right {
                                if let Value::Number(n, _) = &value_with_span.value {
                                    return Some((n.clone(), "+inf".to_string()));
                                }
                            }
                            None
                        }
                        BinaryOperator::Lt => {
                            if let Expr::Value(value_with_span) = &**right {
                                if let Value::Number(n, _) = &value_with_span.value {
                                    return Some(("-inf".to_string(), format!("({}", n)));
                                }
                            }
                            None
                        }
                        BinaryOperator::LtEq => {
                            if let Expr::Value(value_with_span) = &**right {
                                if let Value::Number(n, _) = &value_with_span.value {
                                    return Some(("-inf".to_string(), n.clone()));
                                }
                            }
                            None
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        // Could add BETWEEN handling here
        _ => None,
    }
}

/// Get key values from an IN expression: key IN ('a', 'b', 'c')
/// Recursively handles AND expressions
pub fn sel_get_key_in_values(expr: &Option<Expr>) -> Option<Vec<String>> {
    expr.as_ref().and_then(sel_extract_key_in_from_expr)
}

fn sel_extract_key_in_from_expr(expr: &Expr) -> Option<Vec<String>> {
    match expr {
        Expr::InList { expr: left, list, negated, .. } => {
            if *negated {
                return None; // NOT IN not supported yet
            }
            match &**left {
                Expr::Identifier(ident) if ident.value.to_lowercase() == "key" => {
                    let values: Vec<String> = list.iter()
                        .filter_map(sel_extract_value)
                        .collect();
                    if values.is_empty() { None } else { Some(values) }
                }
                _ => None,
            }
        }
        Expr::BinaryOp { left, op: BinaryOperator::And, right } => {
            sel_extract_key_in_from_expr(left)
                .or_else(|| sel_extract_key_in_from_expr(right))
        }
        _ => None,
    }
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