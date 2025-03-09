// pattern/extractors/common.rs
use sqlparser::ast::{BinaryOperator, Expr, Value, ObjectNamePart, TableFactor};
use std::collections::HashMap;

// Condition value enum
pub enum ConditionValue {
    String(String),
    Number(String),
    Comparison(BinaryOperator, String),
    OrCondition(Box<HashMap<String, ConditionValue>>, Box<HashMap<String, ConditionValue>>),
    Unknown,
}

// Helper to extract key from condition
pub fn extract_key_from_condition(expr: &Expr) -> Option<String> {
    match expr {
        Expr::BinaryOp { left, op, right } => {
            if *op == BinaryOperator::Eq && 
               matches!(&**left, Expr::Identifier(ident) if ident.value.to_lowercase() == "key") {
                match &**right {
                    Expr::Value(value_with_span) => {
                        match &value_with_span.value {
                            Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) => Some(s.clone()),
                            Value::Number(n, _) => Some(n.clone()),
                            _ => None,
                        }
                    },
                    _ => None,
                }
            } else if *op == BinaryOperator::And {
                // Check both sides
                extract_key_from_condition(left).or_else(|| extract_key_from_condition(right))
            } else {
                None
            }
        },
        _ => None,
    }
}

// Process complex conditions with nested AND/OR 
pub fn extract_complex_conditions(expr: &Expr) -> HashMap<String, ConditionValue> {
    let mut conditions = HashMap::new();
    
    match expr {
        // Simple equality
        Expr::BinaryOp { left, op, right } if *op == BinaryOperator::Eq => {
            if let Expr::Identifier(ident) = &**left {
                if let Expr::Value(val) = &**right {
                    conditions.insert(
                        ident.value.clone(),
                        match &val.value {
                            Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) => 
                                ConditionValue::String(s.clone()),
                            Value::Number(n, _) => ConditionValue::Number(n.clone()),
                            _ => ConditionValue::Unknown,
                        }
                    );
                }
            }
        },
        
        // Comparison operators
        Expr::BinaryOp { left, op, right } if [
            BinaryOperator::Gt, 
            BinaryOperator::Lt, 
            BinaryOperator::GtEq, 
            BinaryOperator::LtEq
        ].contains(op) => {
            if let Expr::Identifier(ident) = &**left {
                if let Expr::Value(val) = &**right {
                    if let Value::Number(n, _) = &val.value {
                        conditions.insert(
                            ident.value.clone(),
                            ConditionValue::Comparison(op.clone(), n.clone())
                        );
                    }
                }
            }
        },
        
        // AND conditions - merge both sides
        Expr::BinaryOp { left, op: BinaryOperator::And, right } => {
            let mut left_conds = extract_complex_conditions(left);
            let right_conds = extract_complex_conditions(right);
            
            // Merge results
            for (k, v) in right_conds {
                left_conds.insert(k, v);
            }
            
            conditions = left_conds;
        },
        
        // OR conditions - create a composite condition
        Expr::BinaryOp { left, op: BinaryOperator::Or, right } => {
            let left_conds = extract_complex_conditions(left);
            let right_conds = extract_complex_conditions(right);
            
            conditions.insert(
                "OR_CONDITION".to_string(), 
                ConditionValue::OrCondition(
                    Box::new(left_conds),
                    Box::new(right_conds)
                )
            );
        },
        
        // Parenthesized expressions
        Expr::Nested(nested) => {
            conditions = extract_complex_conditions(nested);
        },
        
        _ => {}
    }
    
    conditions
}

// Helper to determine Redis data type from table name
pub fn determine_table_type(table: &str) -> &'static str {
    if table.ends_with("__hash") {
        "hash"
    } else if table.ends_with("__list") {
        "list"
    } else if table.ends_with("__set") {
        "set"
    } else if table.ends_with("__zset") {
        "zset"
    } else {
        "string"
    }
}

// Helper to check if a table is of specific type
pub fn is_table_type(table: &TableFactor, suffix: &str) -> bool {
    use sqlparser::ast::TableFactor;
    if let TableFactor::Table { name, .. } = table {
        if !name.0.is_empty() {
            let ObjectNamePart::Identifier(ident) = &name.0[0];
            return ident.value.ends_with(suffix);
        }
        false
    } else {
        false
    }
}