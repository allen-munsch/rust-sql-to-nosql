// pattern/extractors/set_ops.rs
use sqlparser::ast::{Statement, Expr, SetExpr, BinaryOperator, Value, ObjectNamePart, TableFactor};
use crate::pattern::combinators::Pattern;
use crate::pattern::matchers;
use super::common::extract_key_from_condition;

/// Information extracted for a Redis SMEMBERS command
#[derive(Debug, Clone)]
pub struct SetGetAllInfo {
    pub key: String,
}

/// Information extracted for a Redis SISMEMBER command
#[derive(Debug, Clone)]
pub struct SetMemberInfo {
    pub key: String,
    pub member: String,
}

/// Extract set member from expressions
pub fn extract_set_member(expr: &Expr) -> Option<String> {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            if let Expr::Identifier(ident) = &**left {
                // Check for member = 'value'
                if ident.value.to_lowercase() == "member" && *op == BinaryOperator::Eq {
                    if let Expr::Value(val) = &**right {
                        if let Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) = &val.value {
                            return Some(s.clone());
                        }
                    }
                }
            }
            
            // Check for AND conditions
            if *op == BinaryOperator::And {
                return extract_set_member(left)
                    .or_else(|| extract_set_member(right));
            }
            
            None
        },
        _ => None
    }
}

/// Extract data for a Redis SMEMBERS command
pub fn extract_set_getall(stmt: &Statement) -> Option<SetGetAllInfo> {
    // First match a wildcard select
    let select = match matchers::common::wildcard_select().match_pattern(stmt) {
        Ok(select) => select,
        Err(_) => return None,
    };
    
    // Check for a set table
    if select.from.is_empty() {
        return None;
    }
    
    let is_set_table = match &select.from[0].relation {
        table => matchers::common::set_table().match_pattern(table).is_ok(),
    };
    
    if !is_set_table {
        return None;
    }
    
    // Check for a key equals condition
    if let Some(where_clause) = &select.selection {
        // Make sure there's no member condition
        if let Ok(_) = matchers::common::field_equals("member").match_pattern(where_clause) {
            return None;
        }
        
        match matchers::common::key_equals().match_pattern(where_clause) {
            Ok(key) => Some(SetGetAllInfo { key }),
            Err(_) => None,
        }
    } else {
        None
    }
}

/// Helper to extract member from condition
fn extract_member_from_condition(expr: &Expr) -> Option<String> {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            if *op == BinaryOperator::Eq && 
               matches!(&**left, Expr::Identifier(ident) if ident.value.to_lowercase() == "member") {
                match &**right {
                    Expr::Value(value_with_span) => {
                        match &value_with_span.value {
                            Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) => Some(s.clone()),
                            _ => None,
                        }
                    },
                    _ => None,
                }
            } else if *op == BinaryOperator::And {
                // Check both sides
                extract_member_from_condition(left).or_else(|| extract_member_from_condition(right))
            } else {
                None
            }
        },
        _ => None,
    }
}

/// Extract data for a Redis SISMEMBER command
pub fn extract_set_ismember(stmt: &Statement) -> Option<SetMemberInfo> {
    if let Statement::Query(query) = stmt {
        if let SetExpr::Select(select) = &*query.body {
            // Check if we're querying a set table
            for table in &select.from {
                if let TableFactor::Table { name, .. } = &table.relation {
                    if !name.0.is_empty() {
                        match &name.0[0] {
                            ObjectNamePart::Identifier(ident) => {
                                let table_name = &ident.value;
                                if table_name.ends_with("__set") {
                                    // Get key and member from WHERE clause
                                    if let Some(expr) = &select.selection {
                                        if let Some(key) = extract_key_from_condition(expr) {
                                            if let Some(member) = extract_member_from_condition(expr) {
                                                return Some(SetMemberInfo {
                                                    key,
                                                    member,
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