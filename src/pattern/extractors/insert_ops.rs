// pattern/extractors/insert_ops.rs
use sqlparser::ast::{Statement, SetExpr, Query, Value, ObjectNamePart, Expr, TableObject};
use std::collections::HashMap;

/// Info struct for insert commands
#[derive(Debug, Clone)]
pub struct InsertCommandInfo {
    pub table: String,
    pub key: String,
    pub fields: HashMap<String, String>,
}

/// Extract insert command information
pub fn extract_insert_command(stmt: &Statement) -> Option<InsertCommandInfo> {
    if let Statement::Insert(insert) = stmt {
        // Get table name
        let table = match &insert.table {
            TableObject::TableName(object_name) => {
                if object_name.0.is_empty() {
                    return None;
                }
                
                match &object_name.0[0] {
                    ObjectNamePart::Identifier(ident) => ident.value.clone()
                }
            },
            TableObject::TableFunction(_) => return None,
        };
        
        // Handle different data types based on table suffix
        let source = insert.source.as_ref()?;
        
        if let Some(values) = extract_values_from_source(source) {
            if insert.columns.len() != values.len() {
                return None;
            }
            
            let mut field_map = HashMap::new();
            for (i, col) in insert.columns.iter().enumerate() {
                field_map.insert(col.value.clone(), values[i].clone());
            }
            
            // Require key for all inserts
            if let Some(key) = field_map.get("key") {
                return Some(InsertCommandInfo {
                    table,
                    key: key.clone(),
                    fields: field_map,
                });
            }
        }
    }
    None
}


// Helper to extract values from VALUES clause
fn extract_values_from_source(source: &Query) -> Option<Vec<String>> {
    if let SetExpr::Values(values) = &*source.body {
        if values.rows.len() == 1 {
            let row = &values.rows[0];
            let mut result = Vec::new();
            
            for value in row {
                if let Expr::Value(val) = value {
                    match &val.value {
                        Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) => {
                            result.push(s.clone());
                        },
                        Value::Number(n, _) => {
                            result.push(n.clone());
                        },
                        _ => return None,
                    }
                } else {
                    return None;
                }
            }
            
            return Some(result);
        }
    }
    None
}