// tests/integration_test.rs
use sql_redis::SqlToRedisTransformer;


fn normalize_hset(s: &str) -> String {
    let mut parts: Vec<&str> = s.split_whitespace().collect();
    parts.sort(); // Sort lexicographically
    parts.join(" ")
}

#[test]
fn test_full_sql_redis_workflow() {
    let transformer = SqlToRedisTransformer::new().unwrap();
    
    // Create a new user
    let result = transformer.transform("INSERT INTO users__hash (key, name, email, age) VALUES ('user:1001', 'John Doe', 'john@example.com', '30')").unwrap();
    assert_eq!(
        normalize_hset(&result),
        normalize_hset(&"HSET user:1001 name John Doe email john@example.com age 30")
    );
    
    // Get user information
    let result = transformer.transform("SELECT * FROM users__hash WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "HGETALL user:1001");
    
    // Update user information
    let result = transformer.transform("UPDATE users__hash SET age = '31' WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "HSET user:1001 age 31");
    
    // Add posts to user's posts list
    let result = transformer.transform("INSERT INTO posts__list (key, value) VALUES ('user:1001:posts', 'First post')").unwrap();
    assert_eq!(result, "RPUSH user:1001:posts First post");
    
    // Get user's posts
    let result = transformer.transform("SELECT * FROM posts__list WHERE key = 'user:1001:posts'").unwrap();
    assert_eq!(result, "LRANGE user:1001:posts 0 -1");
    
    // Add follower to user's followers set
    let result = transformer.transform("INSERT INTO followers__set (key, member) VALUES ('user:1001:followers', 'user:1002')").unwrap();
    assert_eq!(result, "SADD user:1001:followers user:1002");
    
    // Check if someone is following the user
    let result = transformer.transform("SELECT * FROM followers__set WHERE key = 'user:1001:followers' AND member = 'user:1002'").unwrap();
    assert_eq!(result, "SISMEMBER user:1001:followers user:1002");
    
    // Add user to a leaderboard with score
    let result = transformer.transform("INSERT INTO leaderboard__zset (key, member, score) VALUES ('game:global', 'user:1001', '2500')").unwrap();
    assert_eq!(result, "ZADD game:global 2500 user:1001");
    
    // Get top players
    let result = transformer.transform("SELECT * FROM leaderboard__zset WHERE key = 'game:global' ORDER BY score DESC LIMIT 10").unwrap();
    assert_eq!(result, "ZRANGEBYSCORE game:global -inf +inf");
    
    // Delete user
    let result = transformer.transform("DELETE FROM users__hash WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "DEL user:1001");
}