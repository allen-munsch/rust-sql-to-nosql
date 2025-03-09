// rules/update.rs - Rules for UPDATE statement transformations
// Maps UPDATE statement patterns to Redis commands

use crate::pattern::matchers::update::{is_string_update, is_hash_update, is_list_update, is_zset_update};
use crate::context;
use crate::rules::Rule;
use crate::rules::GenericRule;

/// Create all rules for UPDATE statement transformations
pub fn create_update_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // --------------------------------
        // String operations
        // --------------------------------
        
        // <string-update> ::= "UPDATE" <table> "SET" "value" "=" <new-value> "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            is_string_update,
            Box::new(context::StringUpdateContextBuilder), 
            "string_update"
        )),
        
        // --------------------------------
        // Hash operations
        // --------------------------------
        
        // <hash-update> ::= "UPDATE" <table> "__hash" "SET" <field> "=" <value> ["," <field2> "=" <value2>]... "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            is_hash_update,
            Box::new(context::HashUpdateContextBuilder), 
            "hash_update"
        )),
        
        // --------------------------------
        // List operations
        // --------------------------------
        
        // <list-update> ::= "UPDATE" <table> "__list" "SET" "value" "=" <new-value> "WHERE" "key" "=" <key> "AND" "index" "=" <index>
        Box::new(GenericRule::new(
            is_list_update,
            Box::new(context::ListUpdateContextBuilder), 
            "list_update"
        )),
        
        // --------------------------------
        // Sorted Set operations
        // --------------------------------
        
        // <zset-update> ::= "UPDATE" <table> "__zset" "SET" "score" "=" <new-score> "WHERE" "key" "=" <key> "AND" "member" "=" <member>
        Box::new(GenericRule::new(
            is_zset_update,
            Box::new(context::ZSetUpdateContextBuilder), 
            "zset_update"
        )),
    ]
}