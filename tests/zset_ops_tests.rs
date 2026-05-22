// tests/zset_ops_tests.rs
use sql_redis::SqlToRedisTransformer;

#[test]
fn test_zset_operations() {
    let transformer = SqlToRedisTransformer::new().unwrap();
    
    // Test zset ZRANGEBYSCORE (all elements)
    let result = transformer.transform("SELECT * FROM leaderboard__zset WHERE key = 'game:global'").unwrap();
    assert_eq!(result, "ZRANGEBYSCORE game:global -inf +inf");
    
    // Test zset ZRANGEBYSCORE with score range
    let result = transformer.transform("SELECT * FROM leaderboard__zset WHERE key = 'game:global' AND score > 1000").unwrap();
    assert_eq!(result, "ZRANGEBYSCORE game:global (1000 +inf");
    
    // Test zset ZREVRANGEBYSCORE
    let result = transformer.transform("SELECT * FROM leaderboard__zset WHERE key = 'game:global' ORDER BY score DESC").unwrap();
    assert_eq!(result, "ZREVRANGEBYSCORE game:global +inf -inf");
    
    // Test zset ZADD
    let result = transformer.transform("INSERT INTO leaderboard__zset (key, member, score) VALUES ('game:global', 'user:1001', '2500')").unwrap();
    assert_eq!(result, "ZADD game:global 2500 user:1001");
    
    // Test zset update
    let result = transformer.transform("UPDATE leaderboard__zset SET score = '3000' WHERE key = 'game:global' AND member = 'user:1001'").unwrap();
    assert_eq!(result, "ZADD game:global 3000 user:1001");
    
    // Test zset delete
    let result = transformer.transform("DELETE FROM leaderboard__zset WHERE key = 'game:global'").unwrap();
    assert_eq!(result, "DEL game:global");
    
    // Test zset member delete
    let result = transformer.transform("DELETE FROM leaderboard__zset WHERE key = 'game:global' AND member = 'user:1001'").unwrap();
    assert_eq!(result, "ZREM game:global user:1001");
}

