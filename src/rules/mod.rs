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

/// A Rule defines how a SQL pattern maps to a Redis command via a template
pub trait Rule {
    fn matches(&self, stmt: &Statement) -> bool;
    fn get_context(&self, stmt: &Statement) -> Option<TemplateContext>;
    fn get_template_name(&self) -> &str;
}

/// A generic rule implementation that combines a matcher function with a context builder
pub struct GenericRule<F> {
    matcher: F,
    context_builder: Box<dyn ContextBuilder>,
    template_name: String,
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
        }
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