// rules/select.rs - SELECT statement transformation rules
// More specific patterns come before general ones to avoid shadowing

use sqlparser::ast::Statement;
use crate::pattern::matchers::select;
use crate::context;
use crate::rules::Rule;
use crate::rules::GenericRule;
use crate::ast;
use crate::lua;

/// Helper: build a zset aggregate EVAL command from a statement
fn build_zset_aggregate(stmt: &Statement, script: &str) -> Option<String> {
    let key = ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .and_then(|s| ast::sel_get_key_value(&s.selection))?;
    
    // Try to extract score range; default to -inf/+inf
    let (min, max) = ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .and_then(|s| ast::sel_get_score_range(&s.selection))
        .or_else(|| {
            ast::sel_get_query(stmt)
                .and_then(ast::sel_get_select)
                .and_then(|s| ast::sel_get_score_between(&s.selection))
        })
        .unwrap_or_else(|| ("-inf".to_string(), "+inf".to_string()));
    
    Some(lua::zset_aggregate(script, &key, &min, &max))
}

/// Helper: build a hash field aggregate EVAL command
fn build_hash_field_aggregate(stmt: &Statement, script: &str) -> Option<String> {
    let select = ast::sel_get_query(stmt).and_then(ast::sel_get_select)?;
    let key = ast::sel_get_key_value(&select.selection)?;
    let agg = ast::sel_get_aggregate(&select)?;
    let field = agg.field?;
    Some(lua::hash_aggregate(script, &key, &[field]))
}

/// Helper: build a list aggregate EVAL command
fn build_list_aggregate(stmt: &Statement, script: &str) -> Option<String> {
    let key = ast::sel_get_query(stmt)
        .and_then(ast::sel_get_select)
        .and_then(|s| ast::sel_get_key_value(&s.selection))?;
    Some(lua::list_aggregate(script, &key))
}

/// Create all rules for SELECT statement transformations with rich metadata
pub fn create_select_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // ================================
        // String operations
        // ================================
        
        // <string-get-multi> ::= SELECT * FROM table WHERE key IN (v1, v2) => MGET v1 v2
        Box::new(GenericRule::new(
            select::is_string_get_multi,
            Box::new(context::StringGetMultiContextBuilder),
            "string_mget"
        )
        .with_matcher_name("is_string_get_multi")
        .with_sql_pattern("SELECT * FROM table WHERE key IN ('value1', 'value2')")
        .with_redis_pattern("MGET value1 value2")),
        
        // <string-get> ::= SELECT * FROM table WHERE key = value => GET value
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
            "string_get"
        )
        .with_matcher_name("is_string_get_value")
        .with_sql_pattern("SELECT value FROM table WHERE key = 'value'")
        .with_redis_pattern("GET value")),        

        // ================================
        // Hash operations
        // ================================
        
        // <hash-getall> ::= SELECT * FROM table__hash WHERE key = value => HGETALL value
        Box::new(GenericRule::new(
            select::is_hash_getall,
            Box::new(context::HashGetAllContextBuilder), 
            "hash_getall"
        )
        .with_matcher_name("is_hash_getall")
        .with_sql_pattern("SELECT * FROM table__hash WHERE key = 'value'")
        .with_redis_pattern("HGETALL value")),
        
        // <hash-get> ::= SELECT field FROM table__hash WHERE key = value => HGET value field
        Box::new(GenericRule::new(
            select::is_hash_get,
            Box::new(context::HashGetContextBuilder), 
            "hash_get"
        )
        .with_matcher_name("is_hash_get")
        .with_sql_pattern("SELECT field FROM table__hash WHERE key = 'value'")
        .with_redis_pattern("HGET value field")),
        
        // <hash-hmget> ::= SELECT f1, f2 FROM table__hash WHERE key = value => HMGET value f1 f2
        Box::new(GenericRule::new(
            select::is_hash_hmget,
            Box::new(context::HashMultiGetContextBuilder), 
            "hash_hmget"
        )
        .with_matcher_name("is_hash_hmget")
        .with_sql_pattern("SELECT field1, field2 FROM table__hash WHERE key = 'value'")
        .with_redis_pattern("HMGET value field1 field2...")),
        
        // <hash-count> ::= SELECT COUNT(*) FROM table__hash WHERE key = value => HLEN value
        Box::new(GenericRule::new(
            select::is_hash_count,
            Box::new(context::CountContextBuilder),
            "hash_count"
        )
        .with_matcher_name("is_hash_count")
        .with_sql_pattern("SELECT COUNT(*) FROM table__hash WHERE key = 'value'")
        .with_redis_pattern("HLEN value")),
        
        // <hash-avg> ::= SELECT AVG(field) FROM table__hash WHERE key = value (Lua)
        Box::new(GenericRule::new(
            move |s| select::is_hash_aggregate(s, "AVG"),
            Box::new(context::CountContextBuilder),
            "hash_avg"
        )
        .with_matcher_name("is_hash_avg")
        .with_sql_pattern("SELECT AVG(field) FROM table__hash WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key field")
        .with_direct_command(move |s| build_hash_field_aggregate(s, lua::HASH_FIELD_AVG_SCRIPT))),
        
        // <hash-sum> ::= SELECT SUM(field) FROM table__hash WHERE key = value (Lua)
        Box::new(GenericRule::new(
            move |s| select::is_hash_aggregate(s, "SUM"),
            Box::new(context::CountContextBuilder),
            "hash_sum"
        )
        .with_matcher_name("is_hash_sum")
        .with_sql_pattern("SELECT SUM(field) FROM table__hash WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key field")
        .with_direct_command(move |s| build_hash_field_aggregate(s, lua::HASH_FIELD_SUM_SCRIPT))),
        
        // <hash-min> ::= SELECT MIN(field) FROM table__hash WHERE key = value (Lua)
        Box::new(GenericRule::new(
            move |s| select::is_hash_aggregate(s, "MIN"),
            Box::new(context::CountContextBuilder),
            "hash_min"
        )
        .with_matcher_name("is_hash_min")
        .with_sql_pattern("SELECT MIN(field) FROM table__hash WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key field")
        .with_direct_command(move |s| build_hash_field_aggregate(s, lua::HASH_FIELD_MIN_SCRIPT))),
        
        // <hash-max> ::= SELECT MAX(field) FROM table__hash WHERE key = value (Lua)
        Box::new(GenericRule::new(
            move |s| select::is_hash_aggregate(s, "MAX"),
            Box::new(context::CountContextBuilder),
            "hash_max"
        )
        .with_matcher_name("is_hash_max")
        .with_sql_pattern("SELECT MAX(field) FROM table__hash WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key field")
        .with_direct_command(move |s| build_hash_field_aggregate(s, lua::HASH_FIELD_MAX_SCRIPT))),

        // ================================
        // List operations
        // (aggregates first — they use Lua EVAL scripts)
        // ================================
        
        // <list-avg> ::= SELECT AVG(value) FROM table__list WHERE key = value (Lua)
        Box::new(GenericRule::new(
            move |s| select::is_list_aggregate(s, "AVG"),
            Box::new(context::CountContextBuilder),
            "list_avg"
        )
        .with_matcher_name("is_list_avg")
        .with_sql_pattern("SELECT AVG(value) FROM table__list WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key")
        .with_direct_command(move |s| build_list_aggregate(s, lua::LIST_AVG_SCRIPT))),
        
        // <list-sum> ::= SELECT SUM(value) FROM table__list WHERE key = value (Lua)
        Box::new(GenericRule::new(
            move |s| select::is_list_aggregate(s, "SUM"),
            Box::new(context::CountContextBuilder),
            "list_sum"
        )
        .with_matcher_name("is_list_sum")
        .with_sql_pattern("SELECT SUM(value) FROM table__list WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key")
        .with_direct_command(move |s| build_list_aggregate(s, lua::LIST_SUM_SCRIPT))),
        
        // <list-min> ::= SELECT MIN(value) FROM table__list WHERE key = value (Lua)
        Box::new(GenericRule::new(
            move |s| select::is_list_aggregate(s, "MIN"),
            Box::new(context::CountContextBuilder),
            "list_min"
        )
        .with_matcher_name("is_list_min")
        .with_sql_pattern("SELECT MIN(value) FROM table__list WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key")
        .with_direct_command(move |s| build_list_aggregate(s, lua::LIST_MIN_SCRIPT))),
        
        // <list-max> ::= SELECT MAX(value) FROM table__list WHERE key = value (Lua)
        Box::new(GenericRule::new(
            move |s| select::is_list_aggregate(s, "MAX"),
            Box::new(context::CountContextBuilder),
            "list_max"
        )
        .with_matcher_name("is_list_max")
        .with_sql_pattern("SELECT MAX(value) FROM table__list WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key")
        .with_direct_command(move |s| build_list_aggregate(s, lua::LIST_MAX_SCRIPT))),
        
        // <list-get-index> ::= SELECT * FROM table__list WHERE key = value AND index = n => LINDEX value n
        Box::new(GenericRule::new(
            select::is_list_get_index,
            Box::new(context::ListGetIndexContextBuilder), 
            "list_get_index"
        )
        .with_matcher_name("is_list_get_index")
        .with_sql_pattern("SELECT * FROM table__list WHERE key = 'value' AND index = n")
        .with_redis_pattern("LINDEX value n")),
        
        // <list-get-index-range> ::= SELECT * FROM table__list WHERE key = value AND index < n => LRANGE value 0 n-1
        Box::new(GenericRule::new(
            select::is_list_get_index_range,
            Box::new(context::ListGetIndexRangeContextBuilder),
            "list_get_index_range"
        )
        .with_matcher_name("is_list_get_index_range")
        .with_sql_pattern("SELECT * FROM table__list WHERE key = 'value' AND index < n")
        .with_redis_pattern("LRANGE value 0 n-1")),
        
        // <list-get-range> ::= SELECT * FROM table__list WHERE key = value LIMIT n => LRANGE value 0 n-1
        Box::new(GenericRule::new(
            select::is_list_get_range,
            Box::new(context::ListGetRangeContextBuilder), 
            "list_get_range"
        )
        .with_matcher_name("is_list_get_range")
        .with_sql_pattern("SELECT * FROM table__list WHERE key = 'value' LIMIT n")
        .with_redis_pattern("LRANGE value 0 n-1")),
        
        // <list-getall> ::= SELECT * FROM table__list WHERE key = value => LRANGE value 0 -1
        Box::new(GenericRule::new(
            select::is_list_getall,
            Box::new(context::ListGetAllContextBuilder), 
            "list_getall"
        )
        .with_matcher_name("is_list_getall")
        .with_sql_pattern("SELECT * FROM table__list WHERE key = 'value'")
        .with_redis_pattern("LRANGE value 0 -1")),
        
        // <list-count> ::= SELECT COUNT(*) FROM table__list WHERE key = value => LLEN value
        Box::new(GenericRule::new(
            select::is_list_count,
            Box::new(context::CountContextBuilder),
            "list_count"
        )
        .with_matcher_name("is_list_count")
        .with_sql_pattern("SELECT COUNT(*) FROM table__list WHERE key = 'value'")
        .with_redis_pattern("LLEN value")),

        // ================================
        // Set operations
        // ================================
        
        // <set-ismember> ::= SELECT * FROM table__set WHERE key = value AND member = m => SISMEMBER value m
        Box::new(GenericRule::new(
            select::is_set_ismember,
            Box::new(context::SetIsMemberContextBuilder), 
            "set_ismember"
        )
        .with_matcher_name("is_set_ismember")
        .with_sql_pattern("SELECT * FROM table__set WHERE key = 'value' AND member = 'member'")
        .with_redis_pattern("SISMEMBER value member")),
        
        // <set-getall> ::= SELECT * FROM table__set WHERE key = value => SMEMBERS value
        Box::new(GenericRule::new(
            select::is_set_getall,
            Box::new(context::SetGetAllContextBuilder), 
            "set_getall"
        )
        .with_matcher_name("is_set_getall")
        .with_sql_pattern("SELECT * FROM table__set WHERE key = 'value'")
        .with_redis_pattern("SMEMBERS value")),
        
        // <set-count> ::= SELECT COUNT(*) FROM table__set WHERE key = value => SCARD value
        Box::new(GenericRule::new(
            select::is_set_count,
            Box::new(context::CountContextBuilder),
            "set_count"
        )
        .with_matcher_name("is_set_count")
        .with_sql_pattern("SELECT COUNT(*) FROM table__set WHERE key = 'value'")
        .with_redis_pattern("SCARD value")),

        // ================================
        // Sorted Set operations
        // (aggregates first — they use Lua EVAL scripts)
        // ================================
        
        // <zset-avg> ::= SELECT AVG(score) FROM table__zset WHERE key = value
        Box::new(GenericRule::new(
            move |s| select::is_zset_aggregate(s, "AVG"),
            Box::new(context::CountContextBuilder),
            "zset_avg"
        )
        .with_matcher_name("is_zset_avg")
        .with_sql_pattern("SELECT AVG(score) FROM table__zset WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key -inf +inf")
        .with_direct_command(move |s| build_zset_aggregate(s, lua::ZSET_AVG_SCRIPT))),
        
        // <zset-sum> ::= SELECT SUM(score) FROM table__zset WHERE key = value
        Box::new(GenericRule::new(
            move |s| select::is_zset_aggregate(s, "SUM"),
            Box::new(context::CountContextBuilder),
            "zset_sum"
        )
        .with_matcher_name("is_zset_sum")
        .with_sql_pattern("SELECT SUM(score) FROM table__zset WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key -inf +inf")
        .with_direct_command(move |s| build_zset_aggregate(s, lua::ZSET_SUM_SCRIPT))),
        
        // <zset-min> ::= SELECT MIN(score) FROM table__zset WHERE key = value
        Box::new(GenericRule::new(
            move |s| select::is_zset_aggregate(s, "MIN"),
            Box::new(context::CountContextBuilder),
            "zset_min"
        )
        .with_matcher_name("is_zset_min")
        .with_sql_pattern("SELECT MIN(score) FROM table__zset WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key -inf +inf")
        .with_direct_command(move |s| build_zset_aggregate(s, lua::ZSET_MIN_SCRIPT))),
        
        // <zset-max> ::= SELECT MAX(score) FROM table__zset WHERE key = value
        Box::new(GenericRule::new(
            move |s| select::is_zset_aggregate(s, "MAX"),
            Box::new(context::CountContextBuilder),
            "zset_max"
        )
        .with_matcher_name("is_zset_max")
        .with_sql_pattern("SELECT MAX(score) FROM table__zset WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key -inf +inf")
        .with_direct_command(move |s| build_zset_aggregate(s, lua::ZSET_MAX_SCRIPT))),
        
        // <zset-stddev-pop> ::= SELECT STDDEV_POP(score) FROM table__zset WHERE key = value
        Box::new(GenericRule::new(
            move |s| select::is_zset_aggregate(s, "STDDEV_POP"),
            Box::new(context::CountContextBuilder),
            "zset_stddev_pop"
        )
        .with_matcher_name("is_zset_stddev_pop")
        .with_sql_pattern("SELECT STDDEV_POP(score) FROM table__zset WHERE key = 'value'")
        .with_redis_pattern("EVAL '<lua>' 1 key -inf +inf")
        .with_direct_command(move |s| build_zset_aggregate(s, lua::ZSET_STDDEV_POP_SCRIPT))),
        
        // <zset-get-score-between> ::= SELECT * FROM table__zset WHERE key = value AND score BETWEEN n AND m
        Box::new(GenericRule::new(
            select::is_zset_get_score_between,
            Box::new(context::ZSetGetScoreBetweenContextBuilder),
            "zset_get_score_between"
        )
        .with_matcher_name("is_zset_get_score_between")
        .with_sql_pattern("SELECT * FROM table__zset WHERE key = 'value' AND score BETWEEN n AND m")
        .with_redis_pattern("ZRANGEBYSCORE value n m")),
        
        // <zset-get-score-range> ::= SELECT * FROM table__zset WHERE key = value AND score > n
        Box::new(GenericRule::new(
            select::is_zset_get_score_range,
            Box::new(context::ZSetGetScoreRangeContextBuilder), 
            "zset_get_score_range"
        )
        .with_matcher_name("is_zset_get_score_range")
        .with_sql_pattern("SELECT * FROM table__zset WHERE key = 'value' AND score > n")
        .with_redis_pattern("ZRANGEBYSCORE value (n +inf")),
        
        // <zset-get-reversed> ::= SELECT * FROM table__zset WHERE key = value ORDER BY score DESC
        Box::new(GenericRule::new(
            select::is_zset_get_reversed,
            Box::new(context::ZSetGetReversedContextBuilder), 
            "zset_get_reversed"
        )
        .with_matcher_name("is_zset_get_reversed")
        .with_sql_pattern("SELECT * FROM table__zset WHERE key = 'value' ORDER BY score DESC")
        .with_redis_pattern("ZREVRANGEBYSCORE value +inf -inf")),
        
        // <zset-count-score-range> ::= SELECT COUNT(*) FROM table__zset WHERE key = value AND score BETWEEN n AND m
        Box::new(GenericRule::new(
            select::is_zset_count_score_range,
            Box::new(context::ZSetCountScoreRangeContextBuilder),
            "zset_count_score_range"
        )
        .with_matcher_name("is_zset_count_score_range")
        .with_sql_pattern("SELECT COUNT(*) FROM table__zset WHERE key = 'value' AND score BETWEEN n AND m")
        .with_redis_pattern("ZCOUNT value n m")),
        
        // <zset-getall> ::= SELECT * FROM table__zset WHERE key = value => ZRANGEBYSCORE value -inf +inf
        Box::new(GenericRule::new(
            select::is_zset_getall,
            Box::new(context::ZSetGetAllContextBuilder), 
            "zset_getall"
        )
        .with_matcher_name("is_zset_getall")
        .with_sql_pattern("SELECT * FROM table__zset WHERE key = 'value'")
        .with_redis_pattern("ZRANGEBYSCORE value -inf +inf")),
        
        // <zset-count> ::= SELECT COUNT(*) FROM table__zset WHERE key = value => ZCARD value
        Box::new(GenericRule::new(
            select::is_zset_count,
            Box::new(context::CountContextBuilder),
            "zset_count"
        )
        .with_matcher_name("is_zset_count")
        .with_sql_pattern("SELECT COUNT(*) FROM table__zset WHERE key = 'value'")
        .with_redis_pattern("ZCARD value")),
    ]
}
