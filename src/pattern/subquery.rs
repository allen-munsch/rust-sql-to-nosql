// pattern/subquery.rs - Subquery pattern matchers
// Declarative patterns for matching SQL subqueries

use sqlparser::ast::{
    Expr, Query, Statement, TableFactor
};

use super::combinators::{Pattern, extract, or};

/// Subquery types based on context
#[derive(Debug, Clone, PartialEq)]
pub enum SubqueryContext {
    /// Subquery in a FROM clause (derived table)
    FromClause { alias: Option<String> },
    /// Subquery in a WHERE clause
    WhereClause,
    /// Subquery in a SELECT clause (scalar subquery)
    SelectClause,
    /// Subquery in an EXISTS predicate
    ExistsPredicate,
    /// Subquery in an IN predicate
    InPredicate { column: Option<String> },
    /// Subquery used with comparison operator and quantifier (ANY, ALL, SOME)
    Quantified { 
        column: Option<String>,
        operator: String,
        quantifier: String,
    },
}

/// Information about a subquery
#[derive(Debug, Clone)]
pub struct SubqueryInfo {
    /// The subquery
    pub query: Box<Query>,
    /// The context of the subquery
    pub context: SubqueryContext,
    pub negated: bool,
}

/// Pattern that matches a scalar subquery
pub fn scalar_subquery() -> impl Pattern<Expr, SubqueryInfo> {
    extract(|expr: &Expr| {
        match expr {
            Expr::Subquery(query) => {
                Some(SubqueryInfo {
                    query: query.clone(),
                    context: SubqueryContext::SelectClause,
                    negated: false,
                })
            },
            _ => None,
        }
    })
}

/// Pattern that matches an EXISTS subquery
pub fn exists_subquery() -> impl Pattern<Expr, SubqueryInfo> {
    extract(|expr: &Expr| {
        match expr {
            Expr::Exists { subquery, negated } => {
                Some(SubqueryInfo {
                    query: subquery.clone(),
                    context: SubqueryContext::ExistsPredicate,
                    negated: *negated,
                })
            },
            _ => None,
        }
    })
}

/// Pattern that matches an IN subquery
pub fn in_subquery() -> impl Pattern<Expr, SubqueryInfo> {
    extract(|expr: &Expr| {
        match expr {
            Expr::InSubquery { expr, subquery, negated, .. } => {
                let column = match &**expr {
                    Expr::Identifier(ident) => Some(ident.value.clone()),
                    _ => None,
                };
                Some(SubqueryInfo {
                    query: subquery.clone(),
                    context: SubqueryContext::InPredicate { column },
                    negated: *negated
                })
            },
            _ => None,
        }
    })
}

/// Pattern that matches a derived table (subquery in FROM)
pub fn derived_table() -> impl Pattern<TableFactor, SubqueryInfo> {
    extract(|table: &TableFactor| {
        match table {
            TableFactor::Derived { subquery, alias, .. } => {
                let alias_name = alias.as_ref().map(|a| a.name.value.clone());
                
                Some(SubqueryInfo {
                    query: subquery.clone(),
                    context: SubqueryContext::FromClause { alias: alias_name },
                    negated: false,
                })
            },
            _ => None,
        }
    })
}

/// Pattern that matches any type of subquery in an expression
pub fn any_subquery_expr() -> impl Pattern<Expr, SubqueryInfo> {
    or(
        scalar_subquery(),
        or(
            exists_subquery(),
            in_subquery()
        )
    )
}

/// Pattern that matches a quantified comparison subquery (ANY, ALL, SOME)
pub fn quantified_subquery() -> impl Pattern<Expr, SubqueryInfo> {
    extract(|expr: &Expr| {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                // Check if right side is a subquery with quantifier
                if let Expr::Subquery(subquery) = &**right {
                    // Extract column from left side
                    let column = match &**left {
                        Expr::Identifier(ident) => Some(ident.value.clone()),
                        _ => None,
                    };
                    
                    // Determine operator and quantifier (placeholder - in actual implementation
                    // we would need to check for ANY, ALL, SOME quantifiers)
                    let operator = format!("{:?}", op);
                    let quantifier = "ANY".to_string(); // Placeholder
                    
                    return Some(SubqueryInfo {
                        query: subquery.clone(),
                        context: SubqueryContext::Quantified { 
                            column,
                            operator,
                            quantifier,
                        },
                        negated: false,
                    });
                }
                None
            },
            _ => None,
        }
    })
}

/// Extract all subqueries from an expression tree
pub fn extract_subqueries_from_expr(expr: &Expr) -> Vec<SubqueryInfo> {
    let mut subqueries = Vec::new();
    
    // Check for direct subqueries at this node
    if let Ok(info) = any_subquery_expr().match_pattern(expr) {
        subqueries.push(info);
    }
    
    // Recursively check child nodes
    match expr {
        Expr::BinaryOp { left, right, .. } => {
            subqueries.extend(extract_subqueries_from_expr(left));
            subqueries.extend(extract_subqueries_from_expr(right));
        },
        Expr::UnaryOp { expr, .. } => {
            subqueries.extend(extract_subqueries_from_expr(expr));
        },
        Expr::Cast { expr, .. } => {
            subqueries.extend(extract_subqueries_from_expr(expr));
        },
        Expr::IsNull(expr) | Expr::IsNotNull(expr) => {
            subqueries.extend(extract_subqueries_from_expr(expr));
        },
        Expr::Between { expr, low, high, .. } => {
            subqueries.extend(extract_subqueries_from_expr(expr));
            subqueries.extend(extract_subqueries_from_expr(low));
            subqueries.extend(extract_subqueries_from_expr(high));
        },
        Expr::Case { operand, conditions, else_result } => {
            if let Some(op) = operand {
                subqueries.extend(extract_subqueries_from_expr(op));
            }
            for case_when in conditions {
                subqueries.extend(extract_subqueries_from_expr(&case_when.condition));
                subqueries.extend(extract_subqueries_from_expr(&case_when.result));
            }
            if let Some(else_res) = else_result {
                subqueries.extend(extract_subqueries_from_expr(else_res));
            }
        },
        // Already checked direct subqueries above
        _ => {},
    }
    
    subqueries
}

/// Extract all subqueries from a statement
pub fn extract_all_subqueries(stmt: &Statement) -> Vec<SubqueryInfo> {
    let mut subqueries = Vec::new();
    match stmt {
        Statement::Query(query) => {
            // Check FROM clause for derived tables
            if let sqlparser::ast::SetExpr::Select(select) = &*query.body {
                for table in &select.from {
                    // Check base table
                    if let Ok(info) = derived_table().match_pattern(&table.relation) {
                        subqueries.push(info);
                    }
                    // Check JOINed tables
                    for join in &table.joins {
                        if let Ok(info) = derived_table().match_pattern(&join.relation) {
                            subqueries.push(info);
                        }
                    }
                }
                // Check WHERE clause
                if let Some(where_expr) = &select.selection {
                    subqueries.extend(extract_subqueries_from_expr(where_expr));
                }
                // Check SELECT list
                for item in &select.projection {
                    match item {
                        sqlparser::ast::SelectItem::UnnamedExpr(expr) |
                        sqlparser::ast::SelectItem::ExprWithAlias { expr, .. } => {
                            subqueries.extend(extract_subqueries_from_expr(expr));
                        },
                        _ => {},
                    }
                }
                // Check HAVING clause
                if let Some(having) = &select.having {
                    subqueries.extend(extract_subqueries_from_expr(having));
                }
            }
            // Check for nested subqueries in subqueries
            for subquery in subqueries.clone() {
                if let Statement::Query(q) = Statement::Query(subquery.query) {
                    let nested = extract_all_subqueries(&Statement::Query(q));
                    subqueries.extend(nested);
                }
            }
        },
        _ => {},
    }
    subqueries
}

