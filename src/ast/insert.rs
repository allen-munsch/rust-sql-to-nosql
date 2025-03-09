// ast/insert.rs - Pure functions for INSERT AST node extraction
// These functions don't modify state, just extract information from INSERT statements

use sqlparser::ast::{Expr, ObjectNamePart, SetExpr, Statement, TableObject, Value};
use std::collections::HashMap;

/// Get the table name from an INSERT statement
/// Get the table name from an INSERT statement
pub fn ins_get_table_name(stmt: &Statement) -> Option<String> {
    match stmt {
        Statement::Insert(insert) => {
            match &insert.table {
                TableObject::TableName(table_name) => {
                    if table_name.0.is_empty() {
                        None
                    } else {
                        match &table_name.0[0] {
                            ObjectNamePart::Identifier(ident) => Some(ident.value.clone())
                        }
                    }
                },
                _ => None
            }
        },
        _ => None,
    }
}
/// Get the column names from an INSERT statement
pub fn ins_get_column_names(stmt: &Statement) -> Option<Vec<String>> {
    match stmt {
        Statement::Insert(insert) => {
            let columns: Vec<String> = insert.columns.iter()
                .map(|ident| ident.value.clone())
                .collect();
            
            if columns.is_empty() {
                None
            } else {
                Some(columns)
            }
        },
        _ => None,
    }
}

/// Get a map of column index to column name
pub fn ins_get_column_map(stmt: &Statement) -> Option<HashMap<usize, String>> {
    match stmt {
        Statement::Insert(insert) => {
            let map: HashMap<usize, String> = insert.columns.iter()
                .enumerate()
                .map(|(idx, ident)| (idx, ident.value.clone()))
                .collect();
            
            if map.is_empty() {
                None
            } else {
                Some(map)
            }
        },
        _ => None,
    }
}

/// Extract a string value from an SQL expression
pub fn ins_extract_value(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Value(value_with_span) => match &value_with_span.value {
            Value::SingleQuotedString(s) | Value::DoubleQuotedString(s) => Some(s.clone()),
            Value::Number(n, _) => Some(n.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// Extract all values from an INSERT VALUES clause as rows of strings
pub fn ins_get_values_as_strings(stmt: &Statement) -> Option<Vec<Vec<String>>> {
    match stmt {
        Statement::Insert(insert) => {
            insert.source.as_ref().and_then(|source| {
                match &*source.body {
                    SetExpr::Values(values) => {
                        let rows: Vec<Vec<String>> = values.rows.iter()
                            .map(|row| {
                                row.iter()
                                    .filter_map(ins_extract_value)
                                    .collect()
                            })
                            .collect();
                        
                        if rows.is_empty() || rows[0].is_empty() {
                            None
                        } else {
                            Some(rows)
                        }
                    },
                    _ => None,
                }
            })
        },
        _ => None,
    }
}

/// Extract values from an INSERT statement as maps of column name to value
pub fn ins_get_values_as_maps(stmt: &Statement) -> Option<Vec<HashMap<String, String>>> {
    let columns = ins_get_column_names(stmt)?;
    let value_rows = ins_get_values_as_strings(stmt)?;
    
    let maps = value_rows.iter()
        .filter_map(|row| {
            if row.len() != columns.len() {
                return None;
            }
            
            let mut map = HashMap::new();
            for (idx, value) in row.iter().enumerate() {
                map.insert(columns[idx].clone(), value.clone());
            }
            
            Some(map)
        })
        .collect::<Vec<_>>();
    
    if maps.is_empty() {
        None
    } else {
        Some(maps)
    }
}

/// Get a specific column's value from the first row
pub fn ins_get_column_value(stmt: &Statement, column_name: &str) -> Option<String> {
    let value_maps = ins_get_values_as_maps(stmt)?;
    if value_maps.is_empty() {
        return None;
    }
    
    value_maps[0].get(&column_name.to_string()).cloned()
}

/// Get all rows for a specific column
pub fn ins_get_all_column_values(stmt: &Statement, column_name: &str) -> Option<Vec<String>> {
    let value_maps = ins_get_values_as_maps(stmt)?;
    if value_maps.is_empty() {
        return None;
    }
    
    let values: Vec<String> = value_maps.iter()
        .filter_map(|map| map.get(&column_name.to_string()).cloned())
        .collect();
    
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}