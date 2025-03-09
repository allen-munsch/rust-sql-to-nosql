// src/pattern/extractors/query_extractors.rs

use sqlparser::ast::{Query, SetExpr, SelectItem, Expr, TableFactor, ObjectNamePart, OrderByKind};
use super::common::extract_key_from_condition;
use super::common::determine_table_type;

pub fn extract_table_and_key(query: &Query) -> Option<(&'static str, String)> {
    if let SetExpr::Select(select) = &*query.body {
        if select.from.is_empty() {
            return None;
        }
        
        let table = &select.from[0].relation;
        let table_type = match table {
            TableFactor::Table { name, .. } => {
                if !name.0.is_empty() {
                    let ObjectNamePart::Identifier(ident) = &name.0[0];
                    determine_table_type(&ident.value)
                } else {
                    return None;
                }
            },
            _ => return None,
        };
        
        // Get key
        if let Some(expr) = &select.selection {
            if let Some(key) = extract_key_from_condition(expr) {
                return Some((table_type, key));
            }
        }
    }
    None
}

pub fn extract_selected_fields(query: &Query) -> Option<Vec<String>> {
    if let SetExpr::Select(select) = &*query.body {
        let mut fields = Vec::new();
        
        for item in &select.projection {
            match item {
                SelectItem::UnnamedExpr(Expr::Identifier(ident)) => {
                    fields.push(ident.value.clone());
                },
                SelectItem::ExprWithAlias { expr: Expr::Identifier(ident), .. } => {
                    fields.push(ident.value.clone());
                },
                SelectItem::Wildcard(_) => {
                    return Some(vec!["*".to_string()]);
                },
                _ => {},
            }
        }
        
        return Some(fields);
    }
    None
}

pub fn is_ordered_desc(query: &Query) -> bool {
    if let Some(order_by) = &query.order_by {
        if let OrderByKind::Expressions(exprs) = &order_by.kind {
            if !exprs.is_empty() {
                if let Expr::Identifier(ident) = &exprs[0].expr {
                    return ident.value.to_lowercase() == "score" && 
                        exprs[0].options.asc.is_some() && 
                        !exprs[0].options.asc.unwrap();
                }
            }
        }
    }
    false
}

pub fn get_expr_from_query(query: &Query) -> Option<&Expr> {
    if let SetExpr::Select(select) = &*query.body {
        select.selection.as_ref()
    } else {
        None
    }
}