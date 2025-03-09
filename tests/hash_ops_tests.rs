// tests/hash_ops_tests.rs
use sql_redis::SqlToRedisTransformer;

fn normalize_hset(s: &str) -> String {
    let mut parts: Vec<&str> = s.split_whitespace().collect();
    parts.sort(); // Sort lexicographically
    parts.join(" ")
}


#[test]
fn test_hash_operations() {
    let transformer = SqlToRedisTransformer::new().unwrap();
    
    // Test hash HGETALL
    let result = transformer.transform("SELECT * FROM users__hash WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "HGETALL user:1001");
    
    // Test hash HGET
    let result = transformer.transform("SELECT name FROM users__hash WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "HGET user:1001 name");
    
    // Test hash HMGET
    let result = transformer.transform("SELECT name, email FROM users__hash WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "HMGET user:1001 name email");
    
    // Test hash HSET
    let result = transformer.transform("INSERT INTO users__hash (key, name, email) VALUES ('user:1001', 'John Doe', 'john@example.com')").unwrap();
    assert_eq!(
        normalize_hset(&result),
        normalize_hset(&"HSET user:1001 name John Doe email john@example.com")
    );
    
    // Test hash update
    let result = transformer.transform("UPDATE users__hash SET name = 'Jane Doe' WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "HSET user:1001 name Jane Doe");
    
    // Test hash delete
    let result = transformer.transform("DELETE FROM users__hash WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "DEL user:1001");
    
    // Test hash field delete
    let result = transformer.transform("DELETE FROM users__hash WHERE key = 'user:1001' AND field = 'temporary_token'").unwrap();
    assert_eq!(result, "HDEL user:1001 temporary_token");

}

