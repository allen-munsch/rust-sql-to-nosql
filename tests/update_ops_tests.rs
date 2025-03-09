// tests/update_ops_tests.rs
use sql_redis::SqlToRedisTransformer;

#[test]
fn test_update_operations() {
    let transformer = SqlToRedisTransformer::new().unwrap();
    
    // Test string update
    let result = transformer.transform("UPDATE users SET value = 'Jane Doe' WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "SET user:1001 Jane Doe");
    
    // Test hash update (single field)
    let result = transformer.transform("UPDATE users__hash SET name = 'Jane Doe' WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "HSET user:1001 name Jane Doe");
    
    // Test hash update (multiple fields)
    let result = transformer.transform("UPDATE users__hash SET name = 'Jane Doe', status = 'active' WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "HSET user:1001 name Jane Doe status active");
    
    // Test list update
    let result = transformer.transform("UPDATE posts__list SET value = 'Updated post' WHERE key = 'user:1001:posts' AND index = 0").unwrap();
    assert_eq!(result, "LSET user:1001:posts 0 Updated post");
    
    // Test sorted set update
    let result = transformer.transform("UPDATE leaderboard__zset SET score = '3000' WHERE key = 'game:global' AND member = 'user:1001'").unwrap();
    assert_eq!(result, "ZADD game:global 3000 user:1001");
}