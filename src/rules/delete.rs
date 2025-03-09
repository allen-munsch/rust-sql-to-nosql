// Update rules/delete.rs with enhanced metadata

use crate::pattern::matchers::delete;
use crate::context;
use crate::rules::Rule;
use crate::rules::GenericRule;

/// Create all rules for DELETE statement transformations with rich metadata
pub fn create_delete_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // --------------------------------
        // String operations
        // --------------------------------
        
        // <string-delete> ::= "DELETE" "FROM" <table> "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            delete::is_string_delete,
            Box::new(context::StringDeleteContextBuilder), 
            "del"
        )
        .with_matcher_name("is_string_delete")
        .with_sql_pattern("DELETE FROM table WHERE key = 'key'")
        .with_redis_pattern("DEL key")),
        
        // --------------------------------
        // Hash operations
        // --------------------------------
        
        // <hash-delete> ::= "DELETE" "FROM" <table> "__hash" "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            delete::is_hash_delete,
            Box::new(context::HashDeleteContextBuilder), 
            "hash_delete"
        )
        .with_matcher_name("is_hash_delete")
        .with_sql_pattern("DELETE FROM table__hash WHERE key = 'key'")
        .with_redis_pattern("DEL key")),
        
        // <hash-delete-field> ::= "DELETE" "FROM" <table> "__hash" "WHERE" "key" "=" <key> "AND" "field" "=" <field>
        Box::new(GenericRule::new(
            delete::is_hash_delete_field,
            Box::new(context::HashDeleteFieldContextBuilder), 
            "hash_delete_field"
        )
        .with_matcher_name("is_hash_delete_field")
        .with_sql_pattern("DELETE FROM table__hash WHERE key = 'key' AND field = 'field'")
        .with_redis_pattern("HDEL key field")),
        
        // --------------------------------
        // List operations
        // --------------------------------
        
        // <list-delete> ::= "DELETE" "FROM" <table> "__list" "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            delete::is_list_delete,
            Box::new(context::ListDeleteContextBuilder), 
            "list_delete"
        )
        .with_matcher_name("is_list_delete")
        .with_sql_pattern("DELETE FROM table__list WHERE key = 'key'")
        .with_redis_pattern("DEL key")),
        
        // <list-delete-value> ::= "DELETE" "FROM" <table> "__list" "WHERE" "key" "=" <key> "AND" "value" "=" <value>
        Box::new(GenericRule::new(
            delete::is_list_delete_value,
            Box::new(context::ListDeleteValueContextBuilder), 
            "list_delete_value"
        )
        .with_matcher_name("is_list_delete_value")
        .with_sql_pattern("DELETE FROM table__list WHERE key = 'key' AND value = 'value'")
        .with_redis_pattern("LREM key 0 value")),
        
        // --------------------------------
        // Set operations
        // --------------------------------
        
        // <set-delete> ::= "DELETE" "FROM" <table> "__set" "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            delete::is_set_delete,
            Box::new(context::SetDeleteContextBuilder), 
            "set_delete"
        )
        .with_matcher_name("is_set_delete")
        .with_sql_pattern("DELETE FROM table__set WHERE key = 'key'")
        .with_redis_pattern("DEL key")),
        
        // <set-delete-member> ::= "DELETE" "FROM" <table> "__set" "WHERE" "key" "=" <key> "AND" "member" "=" <member>
        Box::new(GenericRule::new(
            delete::is_set_delete_member,
            Box::new(context::SetDeleteMemberContextBuilder), 
            "set_delete_member"
        )
        .with_matcher_name("is_set_delete_member")
        .with_sql_pattern("DELETE FROM table__set WHERE key = 'key' AND member = 'member'")
        .with_redis_pattern("SREM key member")),
        
        // --------------------------------
        // Sorted Set operations
        // --------------------------------
        
        // <zset-delete> ::= "DELETE" "FROM" <table> "__zset" "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            delete::is_zset_delete,
            Box::new(context::ZSetDeleteContextBuilder), 
            "zset_delete"
        )
        .with_matcher_name("is_zset_delete")
        .with_sql_pattern("DELETE FROM table__zset WHERE key = 'key'")
        .with_redis_pattern("DEL key")),
        
        // <zset-delete-member> ::= "DELETE" "FROM" <table> "__zset" "WHERE" "key" "=" <key> "AND" "member" "=" <member>
        Box::new(GenericRule::new(
            delete::is_zset_delete_member,
            Box::new(context::ZSetDeleteMemberContextBuilder), 
            "zset_delete_member"
        )
        .with_matcher_name("is_zset_delete_member")
        .with_sql_pattern("DELETE FROM table__zset WHERE key = 'key' AND member = 'member'")
        .with_redis_pattern("ZREM key member")),
    ]
}
