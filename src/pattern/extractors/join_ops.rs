// pattern/extractors/join_ops.rs
use sqlparser::ast::{JoinConstraint, JoinOperator, ObjectName, ObjectNamePart, SetExpr, Statement, TableFactor};
use crate::pattern::join::{JoinInfo, TableInfo, JoinType, JoinCondition};

/// Extract join information from a query
pub fn extract_join_info(stmt: &Statement) -> Option<Vec<JoinInfo>> {
    if let Statement::Query(query) = stmt {
        if let SetExpr::Select(select) = &*query.body {
            let mut joins = Vec::new();
            
            if select.from.is_empty() {
                return None;
            }
            
            let base_table = extract_table_info(&select.from[0].relation);
            
            for table_with_joins in &select.from {
                for join in &table_with_joins.joins {
                    let join_type = match &join.join_operator {
                        JoinOperator::Inner(_) => JoinType::Inner,
                        JoinOperator::Left(_) => JoinType::LeftOuter,
                        JoinOperator::Right(_) => JoinType::RightOuter,
                        JoinOperator::FullOuter(_) => JoinType::FullOuter,
                        _ => JoinType::Inner,
                    };
                    
                    let right_table = extract_table_info(&join.relation);
                    
                    // Get join condition
                    let condition = match &join.join_operator {
                        JoinOperator::Inner(constraint) | 
                        JoinOperator::Left(constraint) |
                        JoinOperator::Right(constraint) |
                        JoinOperator::FullOuter(constraint) => {
                            extract_join_condition(constraint)
                        },
                        _ => JoinCondition::None,
                    };
                    
                    joins.push(JoinInfo {
                        join_type,
                        left: base_table.clone(),
                        right: right_table,
                        condition,
                    });
                }
            }
            
            if !joins.is_empty() {
                return Some(joins);
            }
        }
    }
    
    None
}

/// Helper to extract table info from table factor
fn extract_table_info(table: &TableFactor) -> TableInfo {
    match table {
        TableFactor::Table { name, alias, .. } => {
            let table_name = if name.0.is_empty() {
                "unknown".to_string()
            } else {
                match &name.0[0] {
                    ObjectNamePart::Identifier(ident) => ident.value.clone()
                }
            };
            
            TableInfo {
                name: table_name,
                alias: alias.as_ref().map(|a| a.name.value.clone()),
                is_derived: false,
            }
        },
        TableFactor::Derived { alias, .. } => {
            let alias_name = alias.as_ref()
                .map(|a| a.name.value.clone())
                .unwrap_or_else(|| "derived".to_string());
                
            TableInfo {
                name: alias_name.clone(),
                alias: Some(alias_name),
                is_derived: true,
            }
        },
        _ => TableInfo {
            name: "unknown".to_string(),
            alias: None,
            is_derived: false,
        }
    }
}

/// Extract join condition
fn extract_join_condition(constraint: &JoinConstraint) -> JoinCondition {
    match constraint {
        JoinConstraint::On(expr) => {
            JoinCondition::On(Box::new(expr.clone()))
        },
        JoinConstraint::Using(columns) => {
            JoinCondition::Using(columns.iter()
                .map(|col| {
                    let ObjectName(parts) = col;
                    if !parts.is_empty() {
                        let ObjectNamePart::Identifier(ident) = &parts[0];
                        return ident.value.clone();
                    };
                    String::new()
                })
                .collect())
        },
        JoinConstraint::Natural => JoinCondition::Natural,
        _ => JoinCondition::None,
    }
}

