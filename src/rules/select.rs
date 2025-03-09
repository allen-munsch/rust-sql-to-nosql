// rules/select.rs - Rules for SELECT statement transformations
// Maps SELECT statement patterns to Redis commands

use crate::pattern::matchers::select;
use crate::context;
use crate::rules::Rule;
use crate::rules::GenericRule;

/// Create all rules for SELECT statement transformations
pub fn create_select_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // --------------------------------
        // String operations
        // --------------------------------
        
        // <string-get> ::= "SELECT" "*" "FROM" <table> "WHERE" "key" "=" <value> => "GET" <value>
        Box::new(GenericRule::new(
            select::is_string_get,
            Box::new(context::StringGetContextBuilder), 
            "string_get"
        )),
        
        // --------------------------------
        // Hash operations
        // --------------------------------
        
        // <hash-getall> ::= "SELECT" "*" "FROM" <table> "__hash" "WHERE" "key" "=" <value> => "HGETALL" <value>
        Box::new(GenericRule::new(
            select::is_hash_getall,
            Box::new(context::HashGetAllContextBuilder), 
            "hash_getall"
        )),
        
        // <hash-get> ::= "SELECT" <field> "FROM" <table> "__hash" "WHERE" "key" "=" <value> => "HGET" <value> <field>
        Box::new(GenericRule::new(
            select::is_hash_get,
            Box::new(context::HashGetContextBuilder), 
            "hash_get"
        )),
        
        // <hash-hmget> ::= "SELECT" <field1> ["," <field2>]... "FROM" <table> "__hash" "WHERE" "key" "=" <value>
        // => "HMGET" <value> <field1> [<field2>]...
        Box::new(GenericRule::new(
            select::is_hash_hmget,
            Box::new(context::HashMultiGetContextBuilder), 
            "hash_hmget"
        )),
        
        // --------------------------------
        // List operations
        // --------------------------------
        
        // <list-get> ::= "SELECT" "*" "FROM" <table> "__list" "WHERE" "key" "=" <value> => "LRANGE" <value> "0" "-1"
        Box::new(GenericRule::new(
            select::is_list_getall,
            Box::new(context::ListGetAllContextBuilder), 
            "list_getall"
        )),
        
        // <list-get-index> ::= "SELECT" "*" "FROM" <table> "__list" "WHERE" "key" "=" <value> "AND" "index" "=" <index>
        // => "LINDEX" <value> <index>
        Box::new(GenericRule::new(
            select::is_list_get_index,
            Box::new(context::ListGetIndexContextBuilder), 
            "list_get_index"
        )),
        
        // <list-get-range> ::= "SELECT" "*" "FROM" <table> "__list" "WHERE" "key" "=" <value> "LIMIT" <limit>
        // => "LRANGE" <value> "0" <limit-1>
        Box::new(GenericRule::new(
            select::is_list_get_range,
            Box::new(context::ListGetRangeContextBuilder), 
            "list_getall"
        )),
        
        // --------------------------------
        // Set operations
        // --------------------------------
        
        // <set-getall> ::= "SELECT" "*" "FROM" <table> "__set" "WHERE" "key" "=" <value> => "SMEMBERS" <value>
        Box::new(GenericRule::new(
            select::is_set_getall,
            Box::new(context::SetGetAllContextBuilder), 
            "set_getall"
        )),
        
        // <set-ismember> ::= "SELECT" "*" "FROM" <table> "__set" "WHERE" "key" "=" <value> "AND" "member" "=" <member>
        // => "SISMEMBER" <value> <member>
        Box::new(GenericRule::new(
            select::is_set_ismember,
            Box::new(context::SetIsMemberContextBuilder), 
            "set_ismember"
        )),
        
        // --------------------------------
        // Sorted Set operations
        // --------------------------------
        
        // <zset-get> ::= "SELECT" "*" "FROM" <table> "__zset" "WHERE" "key" "=" <value>
        // => "ZRANGEBYSCORE" <value> "-inf" "+inf"
        Box::new(GenericRule::new(
            select::is_zset_getall,
            Box::new(context::ZSetGetAllContextBuilder), 
            "zset_getall"
        )),
        
        // <zset-get-score-range> ::= "SELECT" "*" "FROM" <table> "__zset" "WHERE" "key" "=" <value> "AND" "score" <comparison> <score>
        // => "ZRANGEBYSCORE" <value> <min> <max>
        Box::new(GenericRule::new(
            select::is_zset_get_score_range,
            Box::new(context::ZSetGetScoreRangeContextBuilder), 
            "zset_get_score_range"
        )),
        
        // <zset-get-reversed> ::= "SELECT" "*" "FROM" <table> "__zset" "WHERE" "key" "=" <value> "ORDER BY" "score" "DESC"
        // => "ZREVRANGEBYSCORE" <value> "+inf" "-inf"
        Box::new(GenericRule::new(
            select::is_zset_get_reversed,
            Box::new(context::ZSetGetReversedContextBuilder), 
            "zset_get_reversed"
        )),
    ]
}