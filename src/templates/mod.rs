// templates/mod.rs - Template engine for Redis command generation
// Loads and renders Redis command templates based on SQL patterns

use tera::{Context, Tera};
use crate::context::TemplateContext;
use std::io;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum TemplateError {
    IoError(io::Error),
    TeraError(tera::Error),
    InvalidPath(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::IoError(e) => write!(f, "IO error: {}", e),
            TemplateError::TeraError(e) => write!(f, "Template error: {}", e),
            TemplateError::InvalidPath(p) => write!(f, "Invalid path: {}", p),
        }
    }
}

impl Error for TemplateError {}

impl From<io::Error> for TemplateError {
    fn from(error: io::Error) -> Self {
        TemplateError::IoError(error)
    }
}

impl From<tera::Error> for TemplateError {
    fn from(error: tera::Error) -> Self {
        TemplateError::TeraError(error)
    }
}

pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    pub fn new() -> Result<Self, TemplateError> {
        let mut tera = Tera::default();
        
        // Register all command templates
        Self::register_all_templates(&mut tera)?;
        
        Ok(Self { tera })
    }
    
    /// Register all command templates as raw strings
    fn register_all_templates(tera: &mut Tera) -> Result<(), TemplateError> {
        // Common templates
        tera.add_raw_template("del", "DEL {{ key }}")?;
        
        // String operations
        tera.add_raw_template("string_get", "GET {{ key }}")?;
        tera.add_raw_template("string_set", "SET {{ key }} {{ value }}")?;
        tera.add_raw_template("string_update", "SET {{ key }} {{ value }}")?;
        
        // Hash operations
        tera.add_raw_template("hash_getall", "HGETALL {{ key }}")?;
        tera.add_raw_template("hash_get", "HGET {{ key }} {{ field }}")?;
        tera.add_raw_template("hash_hmget", "HMGET {{ key }} {{ fields }}")?;
        tera.add_raw_template("hash_set", "HSET {{ key }} {{ field_values }}")?;
        tera.add_raw_template("hash_update", "HSET {{ key }} {{ field_values }}")?;
        tera.add_raw_template("hash_delete", "DEL {{ key }}")?;
        tera.add_raw_template("hash_delete_field", "HDEL {{ key }} {{ field }}")?;
        
        // List operations
        tera.add_raw_template("list_getall", "LRANGE {{ key }} 0 -1")?;
        tera.add_raw_template("list_get_index", "LINDEX {{ key }} {{ index }}")?;
        tera.add_raw_template("list_get_range", "LRANGE {{ key }} {{ start }} {{ stop }}")?;
        tera.add_raw_template("list_push", "RPUSH {{ key }} {{ value }}")?;
        tera.add_raw_template("list_update", "LSET {{ key }} {{ index }} {{ value }}")?;
        tera.add_raw_template("list_delete", "DEL {{ key }}")?;
        tera.add_raw_template("list_delete_value", "LREM {{ key }} 0 {{ value }}")?;
        
        // Set operations
        tera.add_raw_template("set_getall", "SMEMBERS {{ key }}")?;
        tera.add_raw_template("set_ismember", "SISMEMBER {{ key }} {{ member }}")?;
        tera.add_raw_template("set_add", "SADD {{ key }} {{ members }}")?;
        tera.add_raw_template("set_delete", "DEL {{ key }}")?;
        tera.add_raw_template("set_delete_member", "SREM {{ key }} {{ member }}")?;
        
        // Sorted Set operations
        tera.add_raw_template("zset_getall", "ZRANGEBYSCORE {{ key }} -inf +inf")?;
        tera.add_raw_template("zset_get_score_range", "ZRANGEBYSCORE {{ key }} {{ min }} {{ max }}")?;
        tera.add_raw_template("zset_get_reversed", "ZREVRANGEBYSCORE {{ key }} {{ max }} {{ min }}")?;
        tera.add_raw_template("zset_add", "ZADD {{ key }} {{ score }} {{ member }}")?;
        tera.add_raw_template("zset_update", "ZADD {{ key }} {{ score }} {{ member }}")?;
        tera.add_raw_template("zset_delete", "DEL {{ key }}")?;
        tera.add_raw_template("zset_delete_member", "ZREM {{ key }} {{ member }}")?;
        
        Ok(())
    }
    
    /// Render a template with the given context
    pub fn render(&self, template_name: &str, context: &TemplateContext) -> Result<String, tera::Error> {
        let mut tera_context = Context::new();
        for (key, value) in context {
            tera_context.insert(key, value);
        }
        self.tera.render(template_name, &tera_context)
    }
}