// context/select.rs - Context builders for SELECT statement transformations
// Extracts template variables from SELECT AST nodes

use std::collections::HashMap;
use sqlparser::ast::Statement;
use crate::ast;
use crate::context::TemplateContext;
use crate::context::ContextBuilder;

// --------------------------------
// String Command Context Builders
// --------------------------------

// String GET context builder
pub struct StringGetContextBuilder;
impl ContextBuilder for StringGetContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
            
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        Some(context)
    }
}

// --------------------------------
// Hash Command Context Builders
// --------------------------------

// Hash HGETALL context builder
pub struct HashGetAllContextBuilder;
impl ContextBuilder for HashGetAllContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
            
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        Some(context)
    }
}

// Hash HGET context builder
pub struct HashGetContextBuilder;
impl ContextBuilder for HashGetContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
            
        let field = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_field_name(&select.projection[0]))?;
            
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("field".to_string(), field);
        Some(context)
    }
}

// Hash HMGET context builder for multiple fields
pub struct HashMultiGetContextBuilder;
impl ContextBuilder for HashMultiGetContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        // Extract key
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
        
        // Extract selected fields
        let fields = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .map(|select| ast::sel_get_field_names(&select.projection))?;
        
        if fields.is_empty() {
            return None;
        }
        
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("fields".to_string(), fields.join(" "));
        
        // For Lua script option
        context.insert("fields_array".to_string(), 
            fields.iter()
                .map(|f| format!("'{}'", f))
                .collect::<Vec<_>>()
                .join(", "));
        
        Some(context)
    }
}

// --------------------------------
// List Command Context Builders
// --------------------------------

// List LRANGE (all elements) context builder
pub struct ListGetAllContextBuilder;
impl ContextBuilder for ListGetAllContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
            
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("start".to_string(), "0".to_string());
        context.insert("stop".to_string(), "-1".to_string());
        Some(context)
    }
}

// List LINDEX context builder
pub struct ListGetIndexContextBuilder;
impl ContextBuilder for ListGetIndexContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
            
        let index = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_field_filter(&select.selection, "index"))?;
            
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("index".to_string(), index);
        Some(context)
    }
}

// List LRANGE with limit context builder
pub struct ListGetRangeContextBuilder;
impl ContextBuilder for ListGetRangeContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
            
        let limit = ast::sel_get_query(stmt)
            .and_then(|query| ast::sel_get_limit(query))?;
            
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("start".to_string(), "0".to_string());
        context.insert("stop".to_string(), (limit - 1).to_string());
        Some(context)
    }
}

// --------------------------------
// Set Command Context Builders
// --------------------------------

// Set SMEMBERS context builder
pub struct SetGetAllContextBuilder;
impl ContextBuilder for SetGetAllContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
            
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        Some(context)
    }
}

// Set SISMEMBER context builder
pub struct SetIsMemberContextBuilder;
impl ContextBuilder for SetIsMemberContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
            
        let member = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_field_filter(&select.selection, "member"))?;
            
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("member".to_string(), member);
        Some(context)
    }
}

// --------------------------------
// Sorted Set Command Context Builders
// --------------------------------

// ZSet ZRANGE (all elements) context builder
pub struct ZSetGetAllContextBuilder;
impl ContextBuilder for ZSetGetAllContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
            
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("min".to_string(), "-inf".to_string());
        context.insert("max".to_string(), "+inf".to_string());
        Some(context)
    }
}

// ZSet ZRANGEBYSCORE with score range context builder
pub struct ZSetGetScoreRangeContextBuilder;
impl ContextBuilder for ZSetGetScoreRangeContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
            
        let (min, max) = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_score_range(&select.selection))?;
            
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("min".to_string(), min);
        context.insert("max".to_string(), max);
        Some(context)
    }
}

// ZSet ZREVRANGEBYSCORE context builder
pub struct ZSetGetReversedContextBuilder;
impl ContextBuilder for ZSetGetReversedContextBuilder {
    fn build_context(&self, stmt: &Statement) -> Option<TemplateContext> {
        let key = ast::sel_get_query(stmt)
            .and_then(ast::sel_get_select)
            .and_then(|select| ast::sel_get_key_value(&select.selection))?;
            
        let mut context = HashMap::new();
        context.insert("key".to_string(), key);
        context.insert("max".to_string(), "+inf".to_string());
        context.insert("min".to_string(), "-inf".to_string());
        Some(context)
    }
}