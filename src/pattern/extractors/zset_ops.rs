// pattern/extractors/zset_ops.rs
use sqlparser::ast::{BinaryOperator, Expr, ObjectNamePart, SelectItem, SetExpr, Statement, TableFactor, Value};
use crate::pattern::combinators::Pattern;
use crate::pattern::matchers;
use super::common::extract_key_from_condition;

/// Information extracted for a Redis ZRANGEBYSCORE command (all elements)
#[derive(Debug, Clone)]
pub struct ZSetGetAllInfo {
    pub key: String,
}

/// Information extracted for a Redis ZRANGEBYSCORE command with score range
#[derive(Debug, Clone)]
pub struct ZSetScoreRangeInfo {
    pub key: String,
    pub min: String,
    pub max: String,
}

/// Information extracted for a Redis ZREVRANGEBYSCORE command
#[derive(Debug, Clone)]
pub struct ZSetGetReversedInfo {
    pub key: String,
}

/// Extract score ranges from complex expressions
pub fn extract_score_range(expr: &Expr) -> Option<(String, String)> {
    match expr {
        // Simple comparisons
        Expr::BinaryOp { left, op, right } => {
            if let Expr::Identifier(ident) = &**left {
                if ident.value.to_lowercase() == "score" {
                    match op {
                        BinaryOperator::Gt => {
                            if let Expr::Value(val) = &**right {
                                if let Value::Number(n, _) = &val.value {
                                    return Some((format!("({}",n), "+inf".to_string()));
                                }
                            }
                        }
                        BinaryOperator::GtEq => {
                            if let Expr::Value(val) = &**right {
                                if let Value::Number(n, _) = &val.value {
                                    return Some((n.clone(), "+inf".to_string()));
                                }
                            }
                        }
                        BinaryOperator::Lt => {
                            if let Expr::Value(val) = &**right {
                                if let Value::Number(n, _) = &val.value {
                                    return Some(("-inf".to_string(), format!("({}",n)));
                                }
                            }
                        }
                        BinaryOperator::LtEq => {
                            if let Expr::Value(val) = &**right {
                                if let Value::Number(n, _) = &val.value {
                                    return Some(("-inf".to_string(), n.clone()));
                                }
                            }
                        }
                        // BinaryOperator::Between => {
                        //     // Handle BETWEEN syntax
                        //     if let Expr::Between { expr, low, high } = &**right {
                        //         if let (Expr::Value(low_val), Expr::Value(high_val)) = (&**low, &**high) {
                        //             if let (Value::Number(min, _), Value::Number(max, _)) = 
                        //                 (&low_val.value, &high_val.value) {
                        //                 return Some((min.clone(), max.clone()));
                        //             }
                        //         }
                        //     }
                        // }
                        _ => {}
                    }
                }
            }
            
            // Check for AND conditions combining score ranges
            if *op == BinaryOperator::And {
                let left_range = extract_score_range(left);
                let right_range = extract_score_range(right);
                
                match (left_range, right_range) {
                    (Some((min1, _)), Some((_, max2))) => Some((min1, max2)),
                    (Some(range), None) => Some(range),
                    (None, Some(range)) => Some(range),
                    _ => None
                }
            } else {
                None
            }
        },
        _ => None
    }
}

/// Extract data for a Redis ZRANGEBYSCORE command (all elements)
pub fn extract_zset_getall(stmt: &Statement) -> Option<ZSetGetAllInfo> {
    // First match a wildcard select
    let select = match matchers::common::wildcard_select().match_pattern(stmt) {
        Ok(select) => select,
        Err(_) => return None,
    };
    
    // Check for a zset table
    if select.from.is_empty() {
        return None;
    }
    
    let is_zset_table = match &select.from[0].relation {
        table => matchers::common::zset_table().match_pattern(table).is_ok(),
    };
    
    if !is_zset_table {
        return None;
    }
    
    // Check for a key equals condition
    if let Some(where_clause) = &select.selection {
        // Make sure there's no score condition
        if let Ok(_) = matchers::common::score_range().match_pattern(where_clause) {
            return None;
        }
        
        match matchers::common::key_equals().match_pattern(where_clause) {
            Ok(key) => Some(ZSetGetAllInfo { key }),
            Err(_) => None,
        }
    } else {
        None
    }
}

/// Helper to extract score range from condition
fn extract_score_range_from_condition(expr: &Expr) -> Option<(String, String)> {
    extract_score_range(expr)
}

/// Extract data for a Redis ZRANGEBYSCORE command with range
pub fn extract_zset_get_score_range(stmt: &Statement) -> Option<ZSetScoreRangeInfo> {
    if let Statement::Query(query) = stmt {
        if let SetExpr::Select(select) = &*query.body {
            // Check if we're querying a zset table
            for table in &select.from {
                if let TableFactor::Table { name, .. } = &table.relation {
                    if !name.0.is_empty() {
                        match &name.0[0] {
                            ObjectNamePart::Identifier(ident) => {
                                let table_name = &ident.value;
                                if table_name.ends_with("__zset") {
                                    // Get key from WHERE clause
                                    if let Some(expr) = &select.selection {
                                        if let Some(key) = extract_key_from_condition(expr) {
                                            // Look for score range conditions
                                            if let Some((min, max)) = extract_score_range_from_condition(expr) {
                                                return Some(ZSetScoreRangeInfo {
                                                    key,
                                                    min,
                                                    max,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract data for a Redis ZREVRANGEBYSCORE command
pub fn extract_zset_get_reversed(stmt: &Statement) -> Option<ZSetGetReversedInfo> {
    // First check if it's a query
    let query = match stmt {
        Statement::Query(query) => query,
        _ => return None,
    };
    
    // Check for ORDER BY score DESC
    if !matchers::common::order_by_score_desc().match_pattern(query).is_ok() {
        return None;
    }
    
    // Now match a wildcard select
    let select = match &*query.body {
        SetExpr::Select(select) => {
            if select.projection.len() == 1 && 
               matches!(&select.projection[0], SelectItem::Wildcard(_)) {
                select
            } else {
                return None;
            }
        },
        _ => return None,
    };
    
    // Check for a zset table
    if select.from.is_empty() {
        return None;
    }
    
    let is_zset_table = match &select.from[0].relation {
        table => matchers::common::zset_table().match_pattern(table).is_ok(),
    };
    
    if !is_zset_table {
        return None;
    }
    
    // Check for a key equals condition
    if let Some(where_clause) = &select.selection {
        match matchers::common::key_equals().match_pattern(where_clause) {
            Ok(key) => Some(ZSetGetReversedInfo { key }),
            Err(_) => None,
        }
    } else {
        None
    }
}

