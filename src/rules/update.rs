// Update rules/update.rs with enhanced metadata

use crate::pattern::matchers::update::{is_string_update, is_hash_update, is_list_update, is_zset_update};
use crate::context;
use crate::rules::Rule;
use crate::rules::GenericRule;

/// Create all rules for UPDATE statement transformations with rich metadata
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
        )
        .with_matcher_name("is_string_update")
        .with_sql_pattern("UPDATE table SET value = 'new-value' WHERE key = 'key'")
        .with_redis_pattern("SET key new-value")),
        
        // --------------------------------
        // Hash operations
        // --------------------------------
        
        // <hash-update> ::= "UPDATE" <table> "__hash" "SET" <field> "=" <value> ["," <field2> "=" <value2>]... "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            is_hash_update,
            Box::new(context::HashUpdateContextBuilder), 
            "hash_update"
        )
        .with_matcher_name("is_hash_update")
        .with_sql_pattern("UPDATE table__hash SET field1 = 'value1', field2 = 'value2' WHERE key = 'key'")
        .with_redis_pattern("HSET key field1 value1 field2 value2")),
        
        // --------------------------------
        // List operations
        // --------------------------------
        
        // <list-update> ::= "UPDATE" <table> "__list" "SET" "value" "=" <new-value> "WHERE" "key" "=" <key> "AND" "index" "=" <index>
        Box::new(GenericRule::new(
            is_list_update,
            Box::new(context::ListUpdateContextBuilder), 
            "list_update"
        )
        .with_matcher_name("is_list_update")
        .with_sql_pattern("UPDATE table__list SET value = 'new-value' WHERE key = 'key' AND index = 0")
        .with_redis_pattern("LSET key 0 new-value")),
        
        // --------------------------------
        // Sorted Set operations
        // --------------------------------
        
        // <zset-update> ::= "UPDATE" <table> "__zset" "SET" "score" "=" <new-score> "WHERE" "key" "=" <key> "AND" "member" "=" <member>
        Box::new(GenericRule::new(
            is_zset_update,
            Box::new(context::ZSetUpdateContextBuilder), 
            "zset_update"
        )
        .with_matcher_name("is_zset_update")
        .with_sql_pattern("UPDATE table__zset SET score = 'new-score' WHERE key = 'key' AND member = 'member'")
        .with_redis_pattern("ZADD key new-score member")),
    ]
}