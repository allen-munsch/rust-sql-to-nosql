// rules/mod.rs - Rules engine for SQL to Redis mapping
// Each rule connects a pattern matcher with a context builder

use sqlparser::ast::Statement;
use crate::context::{ContextBuilder, TemplateContext};

mod select;
mod insert;
mod update;
mod delete;

pub use select::create_select_rules;
pub use insert::create_insert_rules;
pub use update::create_update_rules;
pub use delete::create_delete_rules;


// Update the Rule trait in src/rules/mod.rs to include a description method

/// A Rule defines how a SQL pattern maps to a Redis command via a template
pub trait Rule {
    fn matches(&self, stmt: &Statement) -> bool;
    fn get_context(&self, stmt: &Statement) -> Option<TemplateContext>;
    fn get_template_name(&self) -> &str;
    
    /// Returns the function name used for matching
    fn get_matcher_name(&self) -> Option<&str> {
        None // Default implementation returns None
    }
    
    /// Returns the SQL pattern description
    fn get_sql_pattern(&self) -> Option<&str> {
        None // Default implementation returns None
    }
    
    /// Returns the Redis command pattern
    fn get_redis_pattern(&self) -> Option<&str> {
        None // Default implementation returns None
    }
}

// Update GenericRule to include more metadata
pub struct GenericRule<F> {
    matcher: F,
    context_builder: Box<dyn ContextBuilder>,
    template_name: String,
    matcher_name: Option<String>,
    sql_pattern: Option<String>,
    redis_pattern: Option<String>,
}

impl<F> GenericRule<F> 
where
    F: Fn(&Statement) -> bool,
{
    pub fn new(matcher: F, context_builder: Box<dyn ContextBuilder>, template_name: &str) -> Self {
        Self {
            matcher,
            context_builder,
            template_name: template_name.to_string(),
            matcher_name: None,
            sql_pattern: None,
            redis_pattern: None,
        }
    }
    
    pub fn with_matcher_name(mut self, name: &str) -> Self {
        self.matcher_name = Some(name.to_string());
        self
    }
    
    pub fn with_sql_pattern(mut self, pattern: &str) -> Self {
        self.sql_pattern = Some(pattern.to_string());
        self
    }
    
    pub fn with_redis_pattern(mut self, pattern: &str) -> Self {
        self.redis_pattern = Some(pattern.to_string());
        self
    }
}

impl<F> Rule for GenericRule<F> 
where
    F: Fn(&Statement) -> bool,
{
    fn matches(&self, stmt: &Statement) -> bool {
        (self.matcher)(stmt)
    }
    
    fn get_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        self.context_builder.build_context(stmt)
    }
    
    fn get_template_name(&self) -> &str {
        &self.template_name
    }
    
    fn get_matcher_name(&self) -> Option<&str> {
        self.matcher_name.as_deref()
    }
    
    fn get_sql_pattern(&self) -> Option<&str> {
        self.sql_pattern.as_deref()
    }
    
    fn get_redis_pattern(&self) -> Option<&str> {
        self.redis_pattern.as_deref()
    }
}


/// Creates all the rules for SQL to Redis transformations from all rule sets
pub fn create_rules() -> Vec<Box<dyn Rule>> {
    let mut rules = Vec::new();
    
    // Add SELECT rules
    rules.extend(create_select_rules());
    
    // Add INSERT rules
    rules.extend(create_insert_rules());
    
    // Add UPDATE rules

    rules.extend(create_update_rules());

    // Add DELETE rules

    rules.extend(create_delete_rules());

    rules
}