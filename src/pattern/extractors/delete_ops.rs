// pattern/extractors/delete_ops.rs
use sqlparser::ast::{ Expr, ObjectNamePart, Statement};
use super::common::extract_key_from_condition;
use super::list_ops::extract_list_index;
use super::set_ops::extract_set_member;

/// Info struct for delete commands
#[derive(Debug, Clone)]
pub struct DeleteCommandInfo {
    pub table: String,
    pub key: String,
    pub member: Option<String>,
    pub index: Option<String>,
}

/// Extract delete command information
pub fn extract_delete_command(stmt: &Statement) -> Option<DeleteCommandInfo> {
    if let Statement::Delete(delete) = stmt {
        // Get table name
        let table = if delete.tables.is_empty() || delete.tables[0].0.is_empty() {
            return None;
        } else {
            match &delete.tables[0].0[0] {
                ObjectNamePart::Identifier(ident) => ident.value.clone()
            }
        };
        
        // Extract key and/or member
        if let Some(expr) = &delete.selection {
            // Key is required
            if let Some(key) = extract_key_from_expr(expr) {
                // For set/zset operations, also check for member
                let member = extract_set_member(expr);
                // For list operations, check for index
                let index = extract_list_index(expr);
                
                return Some(DeleteCommandInfo {
                    table: table.clone(),
                    key,
                    member,
                    index,
                });
            }
        }
    }
    
    None
}

/// Helper to extract key from expression
fn extract_key_from_expr(expr: &Expr) -> Option<String> {
    extract_key_from_condition(expr)
}