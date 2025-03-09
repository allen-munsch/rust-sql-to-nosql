// pattern/extractors/list_ops.rs
use sqlparser::ast::{Statement, Expr, SetExpr, BinaryOperator, Value, ObjectNamePart, TableFactor};
use crate::pattern::combinators::Pattern;
use crate::pattern::matchers;
use super::common::extract_key_from_condition;

/// Information extracted for a Redis LRANGE command (all elements)
#[derive(Debug, Clone)]
pub struct ListGetAllInfo {
    pub key: String,
}

/// Information extracted for a Redis LINDEX command
#[derive(Debug, Clone)]
pub struct ListIndexInfo {
    pub key: String,
    pub index: String,
}

/// Information extracted for a Redis LRANGE command with limit
#[derive(Debug, Clone)]
pub struct ListGetRangeInfo {
    pub key: String,
    pub limit: u64,
}

/// Extract list index from expressions
pub fn extract_list_index(expr: &Expr) -> Option<String> {
    match expr {
        // Handle index = N
        Expr::BinaryOp { left, op, right } => {
            if let Expr::Identifier(ident) = &**left {
                if ident.value.to_lowercase() == "index" && *op == BinaryOperator::Eq {
                    if let Expr::Value(val) = &**right {
                        if let Value::Number(n, _) = &val.value {
                            return Some(n.clone());
                        }
                    }
                }
            }
            
            // Check for AND conditions where index might be in either side
            if *op == BinaryOperator::And {
                return extract_list_index(left)
                    .or_else(|| extract_list_index(right));
            }
            
            None
        },
        _ => None
    }
}

/// Extract data for a Redis LRANGE command (all elements)
pub fn extract_list_getall(stmt: &Statement) -> Option<ListGetAllInfo> {
    // First match a wildcard select
    let select = match matchers::common::wildcard_select().match_pattern(stmt) {
        Ok(select) => select,
        Err(_) => return None,
    };
    
    // Check for a list table
    if select.from.is_empty() {
        return None;
    }
    
    let is_list_table = match &select.from[0].relation {
        table => matchers::common::list_table().match_pattern(table).is_ok(),
    };
    
    if !is_list_table {
        return None;
    }
    
    // Check for a key equals condition
    if let Some(where_clause) = &select.selection {
        // Make sure there's no index condition
        if let Ok(_) = matchers::common::field_equals("index").match_pattern(where_clause) {
            return None;
        }
        
        match matchers::common::key_equals().match_pattern(where_clause) {
            Ok(key) => Some(ListGetAllInfo { key }),
            Err(_) => None,
        }
    } else {
        None
    }
}

/// Extract data for a Redis LRANGE command with limit
pub fn extract_list_get_range(stmt: &Statement) -> Option<ListGetRangeInfo> {
    // Match a wildcard select
    let select = match matchers::common::wildcard_select().match_pattern(stmt) {
        Ok(select) => select,
        Err(_) => return None,
    };
    
    // Check for a list table
    if select.from.is_empty() {
        return None;
    }
    
    let is_list_table = match &select.from[0].relation {
        table => matchers::common::list_table().match_pattern(table).is_ok(),
    };
    
    if !is_list_table {
        return None;
    }
    
    // Extract key from WHERE clause
    let key = if let Some(where_clause) = &select.selection {
        match matchers::common::key_equals().match_pattern(where_clause) {
            Ok(key) => key,
            Err(_) => return None,
        }
    } else {
        return None;
    };
    
    // Extract limit from LIMIT clause
    if let Statement::Query(query) = stmt {
        if let Some(limit_value) = crate::pattern::matchers::select::query_has_limit(query) {
            return Some(ListGetRangeInfo { key, limit: limit_value });
        }
        None
    } else {
        None
    }
}

/// Extract data for LINDEX command
pub fn extract_list_get_index(stmt: &Statement) -> Option<ListIndexInfo> {
    if let Statement::Query(query) = stmt {
        if let SetExpr::Select(select) = &*query.body {
            // Check if we're querying a list table
            for table in &select.from {
                if let TableFactor::Table { name, .. } = &table.relation {
                    if !name.0.is_empty() {
                        match &name.0[0] {
                            ObjectNamePart::Identifier(ident) => {
                                let table_name = &ident.value;
                                if table_name.ends_with("__list") {
                                    // Get key from WHERE clause
                                    if let Some(expr) = &select.selection {
                                        if let Some(key) = extract_key_from_condition(expr) {
                                            // Look for index = X condition
                                            if let Some(index) = extract_list_index(expr) {
                                                return Some(ListIndexInfo {
                                                    key,
                                                    index,
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