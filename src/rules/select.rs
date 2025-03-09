// Update rules/select.rs with enhanced metadata

use crate::pattern::matchers::select;
use crate::context;
use crate::rules::Rule;
use crate::rules::GenericRule;

/// Create all rules for SELECT statement transformations with rich metadata
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
        )
        .with_matcher_name("is_string_get")
        .with_sql_pattern("SELECT * FROM table WHERE key = 'value'")
        .with_redis_pattern("GET value")),

        Box::new(GenericRule::new(
            select::is_string_get_value,
            Box::new(context::StringGetValueContextBuilder), 
            "string_get"  // Reuse the same template as string_get
        )
        .with_matcher_name("is_string_get_value")
        .with_sql_pattern("SELECT value FROM table WHERE key = 'value'")
        .with_redis_pattern("GET value")),        

        // --------------------------------
        // Hash operations
        // --------------------------------
        
        // <hash-getall> ::= "SELECT" "*" "FROM" <table> "__hash" "WHERE" "key" "=" <value> => "HGETALL" <value>
        Box::new(GenericRule::new(
            select::is_hash_getall,
            Box::new(context::HashGetAllContextBuilder), 
            "hash_getall"
        )
        .with_matcher_name("is_hash_getall")
        .with_sql_pattern("SELECT * FROM table__hash WHERE key = 'value'")
        .with_redis_pattern("HGETALL value")),
        
        // <hash-get> ::= "SELECT" <field> "FROM" <table> "__hash" "WHERE" "key" "=" <value> => "HGET" <value> <field>
        Box::new(GenericRule::new(
            select::is_hash_get,
            Box::new(context::HashGetContextBuilder), 
            "hash_get"
        )
        .with_matcher_name("is_hash_get")
        .with_sql_pattern("SELECT field FROM table__hash WHERE key = 'value'")
        .with_redis_pattern("HGET value field")),
        
        // <hash-hmget> ::= "SELECT" <field1> ["," <field2>]... "FROM" <table> "__hash" "WHERE" "key" "=" <value>
        // => "HMGET" <value> <field1> [<field2>]...
        Box::new(GenericRule::new(
            select::is_hash_hmget,
            Box::new(context::HashMultiGetContextBuilder), 
            "hash_hmget"
        )
        .with_matcher_name("is_hash_hmget")
        .with_sql_pattern("SELECT field1, field2 FROM table__hash WHERE key = 'value'")
        .with_redis_pattern("HMGET value field1 field2...")),
        
        // --------------------------------
        // List operations
        // --------------------------------
        
        // <list-get> ::= "SELECT" "*" "FROM" <table> "__list" "WHERE" "key" "=" <value> => "LRANGE" <value> "0" "-1"
        Box::new(GenericRule::new(
            select::is_list_getall,
            Box::new(context::ListGetAllContextBuilder), 
            "list_getall"
        )
        .with_matcher_name("is_list_getall")
        .with_sql_pattern("SELECT * FROM table__list WHERE key = 'value'")
        .with_redis_pattern("LRANGE value 0 -1")),
        
        // <list-get-index> ::= "SELECT" "*" "FROM" <table> "__list" "WHERE" "key" "=" <value> "AND" "index" "=" <index>
        // => "LINDEX" <value> <index>
        Box::new(GenericRule::new(
            select::is_list_get_index,
            Box::new(context::ListGetIndexContextBuilder), 
            "list_get_index"
        )
        .with_matcher_name("is_list_get_index")
        .with_sql_pattern("SELECT * FROM table__list WHERE key = 'value' AND index = n")
        .with_redis_pattern("LINDEX value n")),
        
        // <list-get-range> ::= "SELECT" "*" "FROM" <table> "__list" "WHERE" "key" "=" <value> "LIMIT" <limit>
        // => "LRANGE" <value> "0" <limit-1>
        Box::new(GenericRule::new(
            select::is_list_get_range,
            Box::new(context::ListGetRangeContextBuilder), 
            "list_get_range"
        )
        .with_matcher_name("is_list_get_range")
        .with_sql_pattern("SELECT * FROM table__list WHERE key = 'value' LIMIT n")
        .with_redis_pattern("LRANGE value 0 n-1")),
        
        // --------------------------------
        // Set operations
        // --------------------------------
        
        // <set-getall> ::= "SELECT" "*" "FROM" <table> "__set" "WHERE" "key" "=" <value> => "SMEMBERS" <value>
        Box::new(GenericRule::new(
            select::is_set_getall,
            Box::new(context::SetGetAllContextBuilder), 
            "set_getall"
        )
        .with_matcher_name("is_set_getall")
        .with_sql_pattern("SELECT * FROM table__set WHERE key = 'value'")
        .with_redis_pattern("SMEMBERS value")),
        
        // <set-ismember> ::= "SELECT" "*" "FROM" <table> "__set" "WHERE" "key" "=" <value> "AND" "member" "=" <member>
        // => "SISMEMBER" <value> <member>
        Box::new(GenericRule::new(
            select::is_set_ismember,
            Box::new(context::SetIsMemberContextBuilder), 
            "set_ismember"
        )
        .with_matcher_name("is_set_ismember")
        .with_sql_pattern("SELECT * FROM table__set WHERE key = 'value' AND member = 'member'")
        .with_redis_pattern("SISMEMBER value member")),
        
        // --------------------------------
        // Sorted Set operations
        // --------------------------------
        
        // <zset-get> ::= "SELECT" "*" "FROM" <table> "__zset" "WHERE" "key" "=" <value>
        // => "ZRANGEBYSCORE" <value> "-inf" "+inf"
        Box::new(GenericRule::new(
            select::is_zset_getall,
            Box::new(context::ZSetGetAllContextBuilder), 
            "zset_getall"
        )
        .with_matcher_name("is_zset_getall")
        .with_sql_pattern("SELECT * FROM table__zset WHERE key = 'value'")
        .with_redis_pattern("ZRANGEBYSCORE value -inf +inf")),
        
        // <zset-get-score-range> ::= "SELECT" "*" "FROM" <table> "__zset" "WHERE" "key" "=" <value> "AND" "score" <comparison> <score>
        // => "ZRANGEBYSCORE" <value> <min> <max>
        Box::new(GenericRule::new(
            select::is_zset_get_score_range,
            Box::new(context::ZSetGetScoreRangeContextBuilder), 
            "zset_get_score_range"
        )
        .with_matcher_name("is_zset_get_score_range")
        .with_sql_pattern("SELECT * FROM table__zset WHERE key = 'value' AND score > n")
        .with_redis_pattern("ZRANGEBYSCORE value (n +inf")),
        
        // <zset-get-reversed> ::= "SELECT" "*" "FROM" <table> "__zset" "WHERE" "key" "=" <value> "ORDER BY" "score" "DESC"
        // => "ZREVRANGEBYSCORE" <value> "+inf" "-inf"
        Box::new(GenericRule::new(
            select::is_zset_get_reversed,
            Box::new(context::ZSetGetReversedContextBuilder), 
            "zset_get_reversed"
        )
        .with_matcher_name("is_zset_get_reversed")
        .with_sql_pattern("SELECT * FROM table__zset WHERE key = 'value' ORDER BY score DESC")
        .with_redis_pattern("ZREVRANGEBYSCORE value +inf -inf")),
    ]
}