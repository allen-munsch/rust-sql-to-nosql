// rules/insert.rs - Rules for INSERT statement transformations
// Maps INSERT statement patterns to Redis commands

use crate::pattern::matchers::insert::{is_string_set, is_hash_set, is_list_push, is_set_add, is_zset_add};
use crate::context;
use crate::rules::Rule;
use crate::rules::GenericRule;

/// Create all rules for INSERT statement transformations
pub fn create_insert_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // --------------------------------
        // String operations
        // --------------------------------
        
        // <string-set> ::= "INSERT" "INTO" <table> "(key, value)" "VALUES" "(" <key> "," <value> ")"
        Box::new(GenericRule::new(
            is_string_set,
            Box::new(context::StringSetContextBuilder), 
            "string_set"
        )),
        
        // --------------------------------
        // Hash operations
        // --------------------------------
        
        // <hash-set> ::= "INSERT" "INTO" <table> "__hash" "(key, field1 [, field2]...)" "VALUES" "(" <key> "," <value1> [, <value2>]... ")"
        Box::new(GenericRule::new(
            is_hash_set,
            Box::new(context::HashSetContextBuilder), 
            "hash_set"
        )),
        
        // --------------------------------
        // List operations
        // --------------------------------
        
        // <list-push> ::= "INSERT" "INTO" <table> "__list" "(key, value)" "VALUES" "(" <key> "," <value> ")"
        Box::new(GenericRule::new(
            is_list_push,
            Box::new(context::ListPushContextBuilder), 
            "list_push"
        )),
        
        // --------------------------------
        // Set operations
        // --------------------------------
        
        // <set-add> ::= "INSERT" "INTO" <table> "__set" "(key, member)" "VALUES" "(" <key> "," <member> ")"
        Box::new(GenericRule::new(
            is_set_add,
            Box::new(context::SetAddContextBuilder), 
            "set_add"
        )),
        
        // --------------------------------
        // Sorted Set operations
        // --------------------------------
        
        // <zset-add> ::= "INSERT" "INTO" <table> "__zset" "(key, member, score)" "VALUES" "(" <key> "," <member> "," <score> ")"
        Box::new(GenericRule::new(
            is_zset_add,
            Box::new(context::ZSetAddContextBuilder), 
            "zset_add"
        )),
    ]
}