// rules/delete.rs - Rules for DELETE statement transformations
// Maps DELETE statement patterns to Redis commands

use crate::pattern::matchers;
use crate::context;
use crate::rules::Rule;
use crate::rules::GenericRule;

/// Create all rules for DELETE statement transformations
pub fn create_delete_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // --------------------------------
        // String operations
        // --------------------------------
        
        // <string-delete> ::= "DELETE" "FROM" <table> "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            matchers::delete::is_string_delete,
            Box::new(context::StringDeleteContextBuilder), 
            "del"
        )),
        
        // --------------------------------
        // Hash operations
        // --------------------------------
        
        // <hash-delete> ::= "DELETE" "FROM" <table> "__hash" "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            matchers::delete::is_hash_delete,
            Box::new(context::HashDeleteContextBuilder), 
            "hash_delete"
        )),
        
        // <hash-delete-field> ::= "DELETE" "FROM" <table> "__hash" "WHERE" "key" "=" <key> "AND" "field" "=" <field>
        Box::new(GenericRule::new(
            matchers::delete::is_hash_delete_field,
            Box::new(context::HashDeleteFieldContextBuilder), 
            "hash_delete_field"
        )),
        
        // --------------------------------
        // List operations
        // --------------------------------
        
        // <list-delete> ::= "DELETE" "FROM" <table> "__list" "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            matchers::delete::is_list_delete,
            Box::new(context::ListDeleteContextBuilder), 
            "list_delete"
        )),
        
        // <list-delete-value> ::= "DELETE" "FROM" <table> "__list" "WHERE" "key" "=" <key> "AND" "value" "=" <value>
        Box::new(GenericRule::new(
            matchers::delete::is_list_delete_value,
            Box::new(context::ListDeleteValueContextBuilder), 
            "list_delete_value"
        )),
        
        // --------------------------------
        // Set operations
        // --------------------------------
        
        // <set-delete> ::= "DELETE" "FROM" <table> "__set" "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            matchers::delete::is_set_delete,
            Box::new(context::SetDeleteContextBuilder), 
            "set_delete"
        )),
        
        // <set-delete-member> ::= "DELETE" "FROM" <table> "__set" "WHERE" "key" "=" <key> "AND" "member" "=" <member>
        Box::new(GenericRule::new(
            matchers::delete::is_set_delete_member,
            Box::new(context::SetDeleteMemberContextBuilder), 
            "set_delete_member"
        )),
        
        // --------------------------------
        // Sorted Set operations
        // --------------------------------
        
        // <zset-delete> ::= "DELETE" "FROM" <table> "__zset" "WHERE" "key" "=" <key>
        Box::new(GenericRule::new(
            matchers::delete::is_zset_delete,
            Box::new(context::ZSetDeleteContextBuilder), 
            "zset_delete"
        )),
        
        // <zset-delete-member> ::= "DELETE" "FROM" <table> "__zset" "WHERE" "key" "=" <key> "AND" "member" "=" <member>
        Box::new(GenericRule::new(
            matchers::delete::is_zset_delete_member,
            Box::new(context::ZSetDeleteMemberContextBuilder), 
            "zset_delete_member"
        )),
    ]
}