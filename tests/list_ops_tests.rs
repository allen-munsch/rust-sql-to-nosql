// tests/list_ops_tests.rs
use sql_redis::SqlToRedisTransformer;

#[test]
fn test_list_operations() {
    let transformer = SqlToRedisTransformer::new().unwrap();
    
    // Test list LRANGE (all elements)
    let result = transformer.transform("SELECT * FROM posts__list WHERE key = 'user:1001:posts'").unwrap();
    assert_eq!(result, "LRANGE user:1001:posts 0 -1");
    
    // Test list LRANGE with limit
    let result = transformer.transform("SELECT * FROM posts__list WHERE key = 'user:1001:posts' LIMIT 10").unwrap();
    assert_eq!(result, "LRANGE user:1001:posts 0 9");
    
    // Test list LINDEX
    let result = transformer.transform("SELECT * FROM posts__list WHERE key = 'user:1001:posts' AND index = 0").unwrap();
    assert_eq!(result, "LINDEX user:1001:posts 0");
    
    // Test list RPUSH
    let result = transformer.transform("INSERT INTO posts__list (key, value) VALUES ('user:1001:posts', 'New post')").unwrap();
    assert_eq!(result, "RPUSH user:1001:posts New post");
    
    // Test list update
    let result = transformer.transform("UPDATE posts__list SET value = 'Updated post' WHERE key = 'user:1001:posts' AND index = 0").unwrap();
    assert_eq!(result, "LSET user:1001:posts 0 Updated post");
    
    // Test list delete
    let result = transformer.transform("DELETE FROM posts__list WHERE key = 'user:1001:posts'").unwrap();
    assert_eq!(result, "DEL user:1001:posts");
    
    // Test list value delete
    let result = transformer.transform("DELETE FROM posts__list WHERE key = 'user:1001:posts' AND value = 'spam'").unwrap();
    assert_eq!(result, "LREM user:1001:posts 0 spam");
}

