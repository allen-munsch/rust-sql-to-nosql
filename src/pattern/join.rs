// pattern/join.rs - JOIN pattern matchers
// Declarative patterns for matching SQL JOIN operations

use sqlparser::ast::{
    Expr, JoinConstraint, JoinOperator, ObjectNamePart, Select, TableFactor, TableWithJoins
};

use super::combinators::{Pattern, extract};

/// Types of JOIN operations
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    LeftOuter,
    RightOuter,
    FullOuter,
    Cross,
    Natural,
}

/// JOIN condition
#[derive(Debug, Clone)]
pub enum JoinCondition {
    /// ON clause (expression)
    On(Box<Expr>),
    /// USING clause (columns)
    Using(Vec<String>),
    Natural,
    /// No condition (CROSS JOIN)
    None,
}

/// Information about a table
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// Table name or alias
    pub name: String,
    /// Table alias if specified
    pub alias: Option<String>,
    /// Is this a derived table (subquery)
    pub is_derived: bool,
}

/// Information about a JOIN relationship
#[derive(Debug, Clone)]
pub struct JoinInfo {
    /// Type of join
    pub join_type: JoinType,
    /// Left table
    pub left: TableInfo,
    /// Right table
    pub right: TableInfo,
    /// JOIN condition
    pub condition: JoinCondition,
}

fn table_info_from_factor(table: &TableFactor) -> TableInfo {
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
        TableFactor::NestedJoin { alias, .. } => {
            let alias_name = alias.as_ref()
                .map(|a| a.name.value.clone())
                .unwrap_or_else(|| "nested".to_string());
            TableInfo {
                name: alias_name.clone(),
                alias: Some(alias_name),
                is_derived: false,
            }
        },
        _ => TableInfo {
            name: "unknown".to_string(),
            alias: None,
            is_derived: false,
        },
    }
}


/// Pattern that extracts table info from a table factor
pub fn table_info() -> impl Pattern<TableFactor, TableInfo> {
    extract(|table: &TableFactor| {
        Some(table_info_from_factor(table))
    })
}

/// Pattern that matches a join operator and extracts its type and condition
pub fn join_operator() -> impl Pattern<JoinOperator, (JoinType, JoinCondition)> {
    extract(|op: &JoinOperator| {
        match op {
            JoinOperator::Inner(constraint) => {
                Some((JoinType::Inner, extract_constraint(constraint)))
            },
            JoinOperator::LeftOuter(constraint) => {
                Some((JoinType::LeftOuter, extract_constraint(constraint)))
            },
            JoinOperator::RightOuter(constraint) => {
                Some((JoinType::RightOuter, extract_constraint(constraint)))
            },
            JoinOperator::FullOuter(constraint) => {
                Some((JoinType::FullOuter, extract_constraint(constraint)))
            },
            JoinOperator::CrossJoin => {
                Some((JoinType::Cross, JoinCondition::None))
            },
            JoinOperator::CrossApply => {
                Some((JoinType::Cross, JoinCondition::None))
            },
            JoinOperator::OuterApply => {
                Some((JoinType::LeftOuter, JoinCondition::None))
            },
            _ => {
                Some((JoinType::Natural, JoinCondition::None))
            },
        }
    })
}

/// Extract JOIN condition from a constraint
fn extract_constraint(constraint: &JoinConstraint) -> JoinCondition {
    match constraint {
        JoinConstraint::On(expr) => {
            JoinCondition::On(Box::new(expr.clone()))
        },
        JoinConstraint::Using(columns) => {
            // Convert ObjectName to String
            JoinCondition::Using(columns.iter().map(|obj_name| obj_name.to_string()).collect())
        },
        JoinConstraint::Natural => JoinCondition::Natural, // v0.55.0 does not implement Natural
        JoinConstraint::None => JoinCondition::None,
    }
}

/// Pattern that extracts a JOIN relationship from a TableWithJoins and its JOINs
pub fn join_info<'a>() -> impl Pattern<(&'a TableWithJoins, &'a sqlparser::ast::Join), JoinInfo> + 'a {
    extract(|(table, join): &(&'a TableWithJoins, &'a sqlparser::ast::Join)| {
        // Get left and right tables
        let left = table_info_from_factor(&table.relation);
        let right = table_info_from_factor(&join.relation);
        // Extract join type and condition
        let (join_type, condition) = join_operator()
            .match_pattern(&join.join_operator)
            .unwrap_or((JoinType::Inner, JoinCondition::None));
        Some(JoinInfo {
            join_type,
            left,
            right,
            condition,
        })
    })
}
/// Extract all JOIN relationships from a SELECT statement
pub fn extract_all_joins(select: &Select) -> Vec<JoinInfo> {
    let mut joins = Vec::new();
    
    for table in &select.from {
        let left_table = table_info_from_factor(&table.relation);
        
        for join in &table.joins {
            let right_table = table_info_from_factor(&join.relation);
            
            let (join_type, condition) = join_operator()
                .match_pattern(&join.join_operator)
                .unwrap_or((JoinType::Inner, JoinCondition::None));
            
            joins.push(JoinInfo {
                join_type,
                left: left_table.clone(),
                right: right_table,
                condition,
            });
        }
    }
    
    joins
}

/// Pattern that matches an equi-join condition (ON a.col = b.col)
pub fn equi_join_condition() -> impl Pattern<Expr, (String, String, String, String)> {
    extract(|expr: &Expr| {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                if *op != sqlparser::ast::BinaryOperator::Eq {
                    return None;
                }
                
                let (left_table, left_col) = match &**left {
                    Expr::CompoundIdentifier(idents) if idents.len() == 2 => {
                        (idents[0].value.clone(), idents[1].value.clone())
                    },
                    _ => return None,
                };
                
                let (right_table, right_col) = match &**right {
                    Expr::CompoundIdentifier(idents) if idents.len() == 2 => {
                        (idents[0].value.clone(), idents[1].value.clone())
                    },
                    _ => return None,
                };
                
                Some((left_table, left_col, right_table, right_col))
            },
            _ => None,
        }
    })
}

/// Pattern that matches a natural join (no ON clause, implied equi-join on same-named columns)
pub fn natural_join() -> impl Pattern<JoinOperator, ()> {
    extract(|op: &JoinOperator| {
        match op {
            JoinOperator::Inner(_) |
            JoinOperator::Left(_) |
            JoinOperator::Right(_) |
            JoinOperator::FullOuter(_) => Some(()),
            _ => None,
        }
    })
}

/// Check if a join is an equi-join (ON a.col = b.col)
pub fn is_equi_join(join_info: &JoinInfo) -> bool {
    match &join_info.condition {
        JoinCondition::On(expr) => {
            equi_join_condition().match_pattern(expr).is_ok()
        },
        JoinCondition::Using(_) => true, // USING clause is always an equi-join
        _ => false,
    }
}