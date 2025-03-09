// Update rules/insert.rs with enhanced metadata

use crate::pattern::matchers::insert::{is_string_set, is_hash_set, is_list_push, is_set_add, is_zset_add};
use crate::context;
use crate::rules::Rule;
use crate::rules::GenericRule;

/// Create all rules for INSERT statement transformations with rich metadata
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
        )
        .with_matcher_name("is_string_set")
        .with_sql_pattern("INSERT INTO table (key, value) VALUES ('key', 'value')")
        .with_redis_pattern("SET key value")),
        
        // --------------------------------
        // Hash operations
        // --------------------------------
        
        // <hash-set> ::= "INSERT" "INTO" <table> "__hash" "(key, field1 [, field2]...)" "VALUES" "(" <key> "," <value1> [, <value2>]... ")"
        Box::new(GenericRule::new(
            is_hash_set,
            Box::new(context::HashSetContextBuilder), 
            "hash_set"
        )
        .with_matcher_name("is_hash_set")
        .with_sql_pattern("INSERT INTO table__hash (key, field1, field2) VALUES ('key', 'value1', 'value2')")
        .with_redis_pattern("HSET key field1 value1 field2 value2")),
        
        // --------------------------------
        // List operations
        // --------------------------------
        
        // <list-push> ::= "INSERT" "INTO" <table> "__list" "(key, value)" "VALUES" "(" <key> "," <value> ")"
        Box::new(GenericRule::new(
            is_list_push,
            Box::new(context::ListPushContextBuilder), 
            "list_push"
        )
        .with_matcher_name("is_list_push")
        .with_sql_pattern("INSERT INTO table__list (key, value) VALUES ('key', 'value')")
        .with_redis_pattern("RPUSH key value")),
        
        // --------------------------------
        // Set operations
        // --------------------------------
        
        // <set-add> ::= "INSERT" "INTO" <table> "__set" "(key, member)" "VALUES" "(" <key> "," <member> ")"
        Box::new(GenericRule::new(
            is_set_add,
            Box::new(context::SetAddContextBuilder), 
            "set_add"
        )
        .with_matcher_name("is_set_add")
        .with_sql_pattern("INSERT INTO table__set (key, member) VALUES ('key', 'member')")
        .with_redis_pattern("SADD key member")),
        
        // --------------------------------
        // Sorted Set operations
        // --------------------------------
        
        // <zset-add> ::= "INSERT" "INTO" <table> "__zset" "(key, member, score)" "VALUES" "(" <key> "," <member> "," <score> ")"
        Box::new(GenericRule::new(
            is_zset_add,
            Box::new(context::ZSetAddContextBuilder), 
            "zset_add"
        )
        .with_matcher_name("is_zset_add")
        .with_sql_pattern("INSERT INTO table__zset (key, member, score) VALUES ('key', 'member', 'score')")
        .with_redis_pattern("ZADD key score member")),
    ]
}