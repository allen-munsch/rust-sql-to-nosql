// pattern/extractors/hash_ops.rs
use sqlparser::ast::{Statement};
use crate::pattern::combinators::Pattern;
use crate::pattern::matchers;

/// Information extracted for a Redis HGETALL command
#[derive(Debug, Clone)]
pub struct HashGetAllInfo {
    pub key: String,
}

/// Extract data for a Redis HGETALL command
pub fn extract_hash_getall(stmt: &Statement) -> Option<HashGetAllInfo> {
    matchers::common::hash_getall()
        .match_pattern(stmt)
        .map(|key| HashGetAllInfo { key })
        .ok()
}

/// Information extracted for a Redis HGET command
#[derive(Debug, Clone)]
pub struct HashGetInfo {
    pub key: String,
    pub field: String,
}

/// Extract data for a Redis HGET command
pub fn extract_hash_get(stmt: &Statement) -> Option<HashGetInfo> {
    // Match a single field select
    let (select, field) = match matchers::common::single_field_select().match_pattern(stmt) {
        Ok(result) => result,
        Err(_) => return None,
    };
    
    // Check for a hash table
    if select.from.is_empty() {
        return None;
    }
    
    let is_hash_table = match &select.from[0].relation {
        table => matchers::common::hash_table().match_pattern(table).is_ok(),
    };
    
    if !is_hash_table {
        return None;
    }
    
    // Check for a key equals condition
    if let Some(where_clause) = &select.selection {
        match matchers::common::key_equals().match_pattern(where_clause) {
            Ok(key) => Some(HashGetInfo { key, field }),
            Err(_) => None,
        }
    } else {
        None
    }
}

/// Information extracted for a Redis HMGET command
#[derive(Debug, Clone)]
pub struct HashMultiGetInfo {
    pub key: String,
    pub fields: Vec<String>,
}

/// Extract data for a Redis HMGET command
pub fn extract_hash_multi_get(stmt: &Statement) -> Option<HashMultiGetInfo> {
    // Match a multi-field select
    let (select, fields) = match matchers::common::multi_field_select().match_pattern(stmt) {
        Ok(result) => result,
        Err(_) => return None,
    };
    
    // Check for a hash table
    if select.from.is_empty() {
        return None;
    }
    
    let is_hash_table = match &select.from[0].relation {
        table => matchers::common::hash_table().match_pattern(table).is_ok(),
    };
    
    if !is_hash_table {
        return None;
    }
    
    // Check for a key equals condition
    if let Some(where_clause) = &select.selection {
        match matchers::common::key_equals().match_pattern(where_clause) {
            Ok(key) => Some(HashMultiGetInfo { key, fields }),
            Err(_) => None,
        }
    } else {
        None
    }
}