// pattern/extractors/string_ops.rs
use sqlparser::ast::Statement;
use crate::pattern::combinators::Pattern;
use crate::pattern::matchers;

/// Information extracted for a Redis GET command from a string table
#[derive(Debug, Clone)]
pub struct StringGetInfo {
    pub key: String,
}

/// Extract data for a Redis GET command
pub fn extract_string_get(stmt: &Statement) -> Option<StringGetInfo> {
    matchers::common::string_get()
        .match_pattern(stmt)
        .map(|key| StringGetInfo { key })
        .ok()
}

/// Information extracted for a Redis SET command
#[derive(Debug, Clone)]
pub struct StringSetInfo {
    pub key: String,
    pub value: String,
}